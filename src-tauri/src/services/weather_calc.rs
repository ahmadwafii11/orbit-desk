//! Berguna untuk Perhitungan cuaca murni (tanpa jaringan dan tanpa disk): suhu terasa, terbit/terbenam matahari, dan penggolongan kondisi cuaca.
use crate::core::civil::days_from_civil;

// Suhu terasa (heat index) dalam °C, rumus regresi Rothfusz (NWS).
// Di bawah ~27 °C rumusnya tidak berlaku, jadi suhu aslinya akan dikembalikan.
// Ini hanyalah estimasi, bukan merupakan data BMKG.
pub fn heat_index_c(temp_c: f64, humidity: f64) -> f64 {
    let t = temp_c * 9.0 / 5.0 + 32.0;
    if t < 80.0 {
        return temp_c;
    }

    let rh = humidity.clamp(0.0, 100.0);
    let hi = -42.379 + 2.049_015_23 * t + 10.143_331_27 * rh
        - 0.224_755_41 * t * rh
        - 0.006_837_83 * t * t
        - 0.054_817_17 * rh * rh
        + 0.001_228_74 * t * t * rh
        + 0.000_852_82 * t * rh * rh
        - 0.000_001_99 * t * t * rh * rh;

    ((hi - 32.0) * 5.0 / 9.0).max(temp_c)
}

/// Terbit dan terbenam matahari (detik sejak epoch, UTC) untuk tanggal sipil lokal `year-month-day` di koordinat `lat`/`lon` (bujur timur positif).
/// Rumus "sunrise equation"; akurat sekitar satu-dua menit untuk lintang rendah seperti Indonesia. 
/// `None` di kutub saat matahari tidak terbit/terbenam.
pub fn sun_times(year: i64, month: u32, day: u32, lat: f64, lon: f64) -> Option<(i64, i64)> {
    let rad = std::f64::consts::PI / 180.0;

    // Hari ke-n sejak J2000 untuk tanggal ini (siang matahari terdekat bujur lokal).
    let n = (days_from_civil(year, month, day) - 10_957) as f64;
    let mean_noon = n - lon / 360.0;

    let anomaly = (357.5291 + 0.985_600_28 * mean_noon).rem_euclid(360.0);
    let m = anomaly * rad;
    let center = 1.9148 * m.sin() + 0.02 * (2.0 * m).sin() + 0.0003 * (3.0 * m).sin();
    let ecliptic_lon = (anomaly + center + 180.0 + 102.9372).rem_euclid(360.0) * rad;

    let transit = 2_451_545.0 + mean_noon + 0.0053 * m.sin() - 0.0069 * (2.0 * ecliptic_lon).sin();

    let sin_decl = ecliptic_lon.sin() * (23.4397 * rad).sin();
    let cos_decl = (1.0 - sin_decl * sin_decl).sqrt();
    let lat_rad = lat * rad;

    // -0.833° = pembiasan atmosfer + jari-jari piringan matahari.
    let cos_hour_angle =
        ((-0.833 * rad).sin() - lat_rad.sin() * sin_decl) / (lat_rad.cos() * cos_decl);
    if !(-1.0..=1.0).contains(&cos_hour_angle) {
        return None;
    }

    let hour_angle = cos_hour_angle.acos() / rad;
    let to_epoch = |julian: f64| ((julian - 2_440_587.5) * 86_400.0).round() as i64;

    Some((
        to_epoch(transit - hour_angle / 360.0),
        to_epoch(transit + hour_angle / 360.0),
    ))
}

/// Golongan kondisi cuaca untuk memilih ikon. 
/// Dicocokkan dari teks deskripsi BMKG (Inggris dan Indonesia). 
/// Urutan pengecekan penting: "Cerah Berawan" harus dicek sebelum "Berawan", dan seterusnya.
pub fn classify(desc_en: &str, desc_id: &str) -> &'static str {
    let text = format!("{} {}", desc_en, desc_id).to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| text.contains(w));

    if has(&["thunder", "petir"]) {
        "thunder"
    } else if has(&["heavy rain", "hujan lebat", "sangat lebat"]) {
        "heavy_rain"
    } else if has(&["light rain", "hujan ringan", "drizzle", "gerimis"]) {
        "drizzle"
    } else if has(&["rain", "hujan", "shower"]) {
        "rain"
    } else if has(&["fog", "kabut", "mist"]) {
        "fog"
    } else if has(&["haze", "kabur", "asap", "smoke"]) {
        "haze"
    } else if has(&["overcast", "berawan tebal"]) {
        "overcast"
    } else if has(&["partly", "cerah berawan"]) {
        "partly"
    } else if has(&["cloudy", "berawan"]) {
        "cloudy"
    } else if has(&["clear", "cerah", "sunny"]) {
        "clear"
    } else {
        "unknown"
    }
}

// Selisih zona waktu (detik dari UTC) untuk nama zona yang dipakai BMKG.
// Dipakai hanya jika respons tidak menyertakan `utc_datetime`.
pub fn timezone_offset_seconds(timezone: &str) -> i64 {
    let tz = timezone.trim();

    match tz {
        "Asia/Makassar" | "WITA" | "+0800" | "+08:00" => 8 * 3_600,
        "Asia/Jayapura" | "WIT" | "+0900" | "+09:00" => 9 * 3_600,
        _ => 7 * 3_600,
    }
}