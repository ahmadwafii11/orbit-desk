use crate::core::civil::{
    civil_from_days, 
    days_from_civil, 
    parse_datetime,
};
use crate::services::weather_calc::{
    classify, 
    heat_index_c, 
    sun_times, 
    timezone_offset_seconds,
};

#[test]
fn tanggal_sipil_bolak_balik_dan_titik_acuan() {
    assert_eq!(days_from_civil(1970, 1, 1), 0);
    assert_eq!(days_from_civil(2000, 1, 1), 10_957);
    assert_eq!(days_from_civil(2026, 11, 25), 20_782);

    for days in [-800, -1, 0, 59, 60, 10_957, 20_782, 25_000] {
        let (y, m, d) = civil_from_days(days);
        assert_eq!(days_from_civil(y, m, d), days);
    }
    assert_eq!(civil_from_days(days_from_civil(2028, 2, 29)), (2028, 2, 29)); // Tahun kabisat.
}

#[test]
fn parse_datetime_menerima_format_bmkg() {
    let a = parse_datetime("2026-11-25 13:00:00").unwrap();
    let b = parse_datetime("2026-11-25T13:00:00").unwrap();
    let c = parse_datetime("2026-11-25T13:00:00Z").unwrap();
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a, 20_782 * 86_400 + 13 * 3_600);

    assert!(parse_datetime("kemarin").is_none());
    assert!(parse_datetime("2026-13-01 00:00:00").is_none());
    assert!(parse_datetime("2026-11-25 25:00:00").is_none());
}

#[test]
fn suhu_terasa_naik_saat_panas_lembap_dan_tidak_di_bawah_ambang() {
    let hi = heat_index_c(32.0, 70.0); // 32 °C dengan kelembapan 70% terasa sekitar 40 °C (tabel NWS: 90 °F/70% = 105 °F).
    assert!((39.0..42.0).contains(&hi), "heat index {hi}");

    assert_eq!(heat_index_c(24.0, 90.0), 24.0); // Di bawah ambang, tidak ada koreksi.
    assert!(heat_index_c(28.0, 10.0) >= 28.0); // Tidak pernah lebih dingin dari suhu aslinya.
}

#[test]
fn terbit_terbenam_jakarta_masuk_akal() {
    let (rise, set) = sun_times(2026, 11, 25, -6.2088, 106.8456).unwrap(); // Jakarta, 25 Nov 2026. Acuan: kalkulator surya NOAA (dihitung terpisah) memberi terbit 05:26 WIB dan terbenam 17:52 WIB. Toleransi 4 menit.
    let day_start_utc = 20_782 * 86_400;
    let wib = |epoch: i64| (epoch - day_start_utc + 7 * 3_600) / 60; // menit sejak 00:00 WIB

    let rise_min = wib(rise);
    let set_min = wib(set);
    assert!(
        (5 * 60 + 22..=5 * 60 + 30).contains(&rise_min),
        "terbit {rise_min}"
    );
    assert!(
        (17 * 60 + 48..=17 * 60 + 56).contains(&set_min),
        "terbenam {set_min}"
    );
    assert!(set > rise);
}

#[test]
fn terbit_terbenam_di_kutub_bisa_tidak_ada() {
    assert!(sun_times(2026, 6, 21, 69.65, 18.96).is_none()); // Tromsø, tengah musim panas: matahari tidak terbenam.
}

#[test]
fn klasifikasi_kondisi_bmkg() {
    assert_eq!(classify("Clear Skies", "Cerah"), "clear");
    assert_eq!(classify("Partly Cloudy", "Cerah Berawan"), "partly");
    assert_eq!(classify("Mostly Cloudy", "Berawan"), "cloudy");
    assert_eq!(classify("Overcast", "Berawan Tebal"), "overcast");
    assert_eq!(classify("Haze", "Udara Kabur"), "haze");
    assert_eq!(classify("Fog", "Kabut"), "fog");
    assert_eq!(classify("Light Rain", "Hujan Ringan"), "drizzle");
    assert_eq!(classify("Rain", "Hujan Sedang"), "rain");
    assert_eq!(classify("Heavy Rain", "Hujan Lebat"), "heavy_rain");
    assert_eq!(classify("Thunderstorm", "Hujan Petir"), "thunder");
    assert_eq!(classify("", "Cerah Berawan"), "partly"); // Hanya teks Indonesia.
    assert_eq!(classify("", "Berawan Tebal"), "overcast");
    assert_eq!(classify("", ""), "unknown");
}

#[test]
fn zona_waktu_indonesia() {
    assert_eq!(timezone_offset_seconds("Asia/Jakarta"), 7 * 3_600);
    assert_eq!(timezone_offset_seconds("Asia/Makassar"), 8 * 3_600);
    assert_eq!(timezone_offset_seconds("Asia/Jayapura"), 9 * 3_600);
    assert_eq!(timezone_offset_seconds("tidak dikenal"), 7 * 3_600);
}