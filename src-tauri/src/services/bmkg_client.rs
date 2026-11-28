//! Berguna untuk Klien HTTP BMKG serta mengurai responsnya.
//!
//! Endpoint: `https://api.bmkg.go.id/publik/prakiraan-cuaca?adm4=<kode>`.
//! Free tanpa kunci; batas 60 permintaan/menit/IP. 
//! Pengurai (`parse`) sengaja dipisah dari HTTP agar dapat diuji dengan contoh JSON tanpa jaringan, dan memakai `serde_json::Value` agar tahan terhadap field yang hilang atau berganti tipe (angka vs teks).
use std::time::Duration;
use serde_json::Value;

use crate::core::{
    civil, 
    error::AppError,
};
use crate::models::weather::{
    Place, 
    Slot, 
    SunDay, 
    WeatherSnapshot,
};
use crate::services::weather_calc;

const ENDPOINT: &str = "https://api.bmkg.go.id/publik/prakiraan-cuaca";

pub fn fetch(code: &str) -> Result<String, AppError> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .build()
        .into();

    let url = format!("{ENDPOINT}?adm4={code}");
    let mut response = agent.get(&url).call().map_err(|error| match error {
        ureq::Error::StatusCode(404) => {
            AppError::Network("BMKG tidak punya prakiraan untuk kode wilayah ini".into())
        }
        ureq::Error::StatusCode(429) => {
            AppError::Network("terlalu banyak permintaan ke BMKG, coba lagi sebentar lagi".into())
        }
        ureq::Error::StatusCode(status) => AppError::Network(format!("BMKG menjawab {status}")),
        other => AppError::Network(other.to_string()),
    })?;

    response
        .body_mut()
        .read_to_string()
        .map_err(|e| AppError::Network(e.to_string()))
}

// Angka dari field yang bisa berupa angka atau teks angka.
fn num(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn text(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    }
}

// Memotong "2026-11-25 07:00:00" menjadi "2026-11-25 07:00".
fn minutes(text: &str) -> String {
    text.chars().take(16).collect()
}

pub fn parse(raw: &str, place: &Place, fetched_at: i64) -> Result<WeatherSnapshot, AppError> {
    let root: Value =
        serde_json::from_str(raw).map_err(|e| AppError::WeatherParse(e.to_string()))?;

    let entry = root
        .get("data")
        .and_then(Value::as_array)
        .and_then(|d| d.first())
        .ok_or_else(|| AppError::WeatherParse("tidak ada bagian \"data\"".into()))?;

    let location = root
        .get("lokasi")
        .or_else(|| entry.get("lokasi"))
        .cloned()
        .unwrap_or(Value::Null);
    let lat = num(location.get("lat"));
    let lon = num(location.get("lon"));
    let timezone = text(location.get("timezone")).unwrap_or_default();

    // `cuaca` = daftar hari, masing-masing daftar slot 3 jam. 
    // Ratakan, dan terima juga bentuk yang sudah rata.
    let mut raw_slots: Vec<&Value> = Vec::new();
    if let Some(days) = entry.get("cuaca").and_then(Value::as_array) {
        for day in days {
            match day {
                Value::Array(items) => raw_slots.extend(items.iter()),
                Value::Object(_) => raw_slots.push(day),
                _ => {}
            }
        }
    }

    let mut utc_offset: Option<i64> = None;
    let mut analysis_date = None;
    let mut slots: Vec<Slot> = Vec::new();

    for item in raw_slots {
        let local = text(item.get("local_datetime"));
        let utc = text(item.get("utc_datetime"));

        let local_secs = local.as_deref().and_then(civil::parse_datetime);
        let utc_secs = utc.as_deref().and_then(civil::parse_datetime);

        // Selisih zona dari pasangan waktu lokal/UTC, lebih tepercaya daripada menebak dari nama zona.
        if utc_offset.is_none() {
            if let (Some(l), Some(u)) = (local_secs, utc_secs) {
                utc_offset = Some(l - u);
            }
        }

        if analysis_date.is_none() {
            analysis_date = text(item.get("analysis_date"));
        }

        let offset = utc_offset.unwrap_or_else(|| weather_calc::timezone_offset_seconds(&timezone));
        let Some(epoch) = utc_secs.or_else(|| local_secs.map(|l| l - offset)) else {
            continue; // slot tanpa waktu tidak bisa dipakai
        };
        let local_text = local
            .as_deref()
            .map(minutes)
            .unwrap_or_else(|| local_from_epoch(epoch + offset));

        let desc_id = text(item.get("weather_desc"));
        let desc_en = text(item.get("weather_desc_en"));
        let temp = num(item.get("t"));
        let humidity = num(item.get("hu"));

        slots.push(Slot {
            time: local_text,
            time_ms: epoch * 1000,
            temp_c: temp,
            humidity,
            wind_kmh: num(item.get("ws")),
            wind_from: text(item.get("wd")),
            wind_deg: num(item.get("wd_deg")),
            cloud_pct: num(item.get("tcc")),
            rain_mm: num(item.get("tp")),
            visibility_m: num(item.get("vs")),
            visibility_text: text(item.get("vs_text")),
            kind: weather_calc::classify(
                desc_en.as_deref().unwrap_or(""),
                desc_id.as_deref().unwrap_or(""),
            )
            .to_string(),
            desc_id,
            desc_en,
            feels_like_c: match (temp, humidity) {
                (Some(t), Some(h)) => Some(weather_calc::heat_index_c(t, h)),
                _ => None,
            },
        });
    }

    if slots.is_empty() {
        return Err(AppError::WeatherParse("prakiraan kosong".into()));
    }
    slots.sort_by_key(|s| s.time_ms);

    let offset = utc_offset.unwrap_or_else(|| weather_calc::timezone_offset_seconds(&timezone));
    let sun = sun_days(&slots, lat, lon);

    Ok(WeatherSnapshot {
        code: place.code.clone(),
        place: place.clone(),
        lat,
        lon,
        utc_offset_s: offset,
        fetched_at,
        analysis_date,
        slots,
        sun,
    })
}

fn local_from_epoch(local_secs: i64) -> String {
    let days = local_secs.div_euclid(86_400);
    let rest = local_secs.rem_euclid(86_400);
    let (y, m, d) = civil::civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}",
        rest / 3600,
        rest % 3600 / 60
    )
}

// Terbit atau terbenam untuk tiap tanggal lokal yang ada di prakiraan.
fn sun_days(slots: &[Slot], lat: Option<f64>, lon: Option<f64>) -> Vec<SunDay> {
    let (Some(lat), Some(lon)) = (lat, lon) else {
        return Vec::new();
    };

    let mut dates: Vec<String> = slots
        .iter()
        .map(|s| s.time.chars().take(10).collect::<String>())
        .collect();
    dates.dedup();

    dates
        .into_iter()
        .filter_map(|date| {
            let (y, m, d) = civil::parse_date(&date)?;
            let (rise, set) = weather_calc::sun_times(y, m, d, lat, lon)?;
            Some(SunDay {
                date,
                sunrise_ms: rise * 1000,
                sunset_ms: set * 1000,
            })
        })
        .collect()
}