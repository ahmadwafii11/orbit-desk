//! Tes data wilayah dan pengurai BMKG, memakai contoh JSON kecil.
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{
      AtomicUsize, 
      Ordering,
    },
};

use crate::models::weather::Place;
use crate::services::{
  bmkg_client, 
  wilayah_index::WilayahIndex,
};

// Folder unik per panggilan: tes berjalan paralel, jadi nama tetap akan saling menghapus folder satu sama lain.
fn temp_dir(name: &str) -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "orbit-weather-{name}-{}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

const PROVINCE: &str = r#"{
  "kode_provinsi": "35",
  "nama_provinsi": "JAWA TIMUR",
  "kabupaten_kota": [
    {
      "kode_kabkota": "35.01",
      "nama_kabkota": "KABUPATEN PACITAN",
      "kecamatan": [
        {
          "kode_kecamatan": "35.01.04",
          "nama_kecamatan": "Pacitan",
          "desa_kelurahan": [
            {"kode_wilayah": "35.01.04.1013", "jenis_wilayah": "Kelurahan", "nama_desakel": "Ploso"},
            {"kode_wilayah": "35.01.04.2001", "jenis_wilayah": "Desa", "nama_desakel": "Sirnoboyo"}
          ]
        },
        {
          "kode_kecamatan": "35.01.05",
          "nama_kecamatan": "Kebonagung",
          "desa_kelurahan": [
            {"kode_wilayah": "35.01.05.2010", "jenis_wilayah": "Desa", "nama_desakel": "Tahunan Baru"},
            {"kode_wilayah": "35.01.05.2011", "jenis_wilayah": "Desa", "nama_desakel": "Ploso"},
            {"kode_wilayah": "35.01.05.2010", "jenis_wilayah": "Desa", "nama_desakel": "Kembaran"},
            {"kode_wilayah": "35.02.01.2001", "jenis_wilayah": "Desa", "nama_desakel": "Nyasar"}
          ]
        }
      ]
    },
    {"kode_kabkota": "35.02", "nama_kabkota": "KABUPATEN PONOROGO", "kecamatan": []},
    {"kode_kabkota": "99.01", "nama_kabkota": "SALAH PROVINSI", "kecamatan": []}
  ]
}"#;

fn load() -> WilayahIndex {
    let dir = temp_dir("index");
    fs::write(dir.join("jatim.json"), PROVINCE).unwrap();
    fs::write(dir.join("rusak.json"), "{ bukan json").unwrap();
    fs::write(dir.join("catatan.txt"), "diabaikan").unwrap();
    WilayahIndex::load_dir(&dir)
}

#[test]
fn index_skips_bad_parts_and_reports_them() {
    let info = load().info();

    assert_eq!(info.provinces, 1);
    assert_eq!(info.desa, 4); // Ploso, Sirnoboyo, Tahunan Baru, Ploso (kode beda). "Kembaran" (kode ganda) & "Nyasar" (induk salah) dilewati.

    let all = info.problems.join("\n");
    assert!(all.contains("rusak.json"), "{all}");
    assert!(all.contains("ganda"), "{all}");
    assert!(all.contains("Nyasar"), "{all}");
    assert!(all.contains("99.01"), "{all}");
    assert!(all.contains("PONOROGO") || all.contains("Ponorogo"), "{all}");
}

#[test]
fn children_walk_down_each_level() {
    let index = load();

    let provinces = index.children("");
    assert_eq!(provinces.len(), 1);
    assert_eq!(provinces[0].name, "Jawa Timur");

    let kab = index.children("35");
    assert_eq!(kab[0].name, "Kabupaten Pacitan");
    assert!(kab[0].has_children);
    assert!(!kab[1].has_children); // Ponorogo kosong

    let kec = index.children("35.01");
    assert_eq!(kec.len(), 2);

    let desa = index.children("35.01.04");
    assert_eq!(desa.len(), 2);
    assert_eq!(desa[0].jenis.as_deref(), Some("Kelurahan"));
    assert!(index.children("35.01.04.1013").is_empty());
    assert!(index.children("12").is_empty());
}

#[test]
fn search_uses_hierarchy_and_prefers_prefix_matches() {
    let index = load();

    let ploso = index.search("ploso", 10);
    assert_eq!(ploso.len(), 2);

    let narrowed = index.search("ploso kebonagung", 10);
    assert_eq!(narrowed.len(), 1);
    assert_eq!(narrowed[0].kecamatan, "Kebonagung");

    let by_kab = index.search("pacitan sirno", 10);
    assert_eq!(by_kab.len(), 1);
    assert_eq!(by_kab[0].desa, "Sirnoboyo");

    assert!(index.search("   ", 10).is_empty());
    assert!(index.search("tidak-ada", 10).is_empty());
}

#[test]
fn find_place_returns_full_hierarchy() {
    let place = load().find_place("35.01.04.1013").unwrap();
    assert_eq!(place.desa, "Ploso");
    assert_eq!(place.kecamatan, "Pacitan");
    assert_eq!(place.kotkab, "Kabupaten Pacitan");
    assert_eq!(place.provinsi, "Jawa Timur");
    assert!(load().find_place("00.00.00.0000").is_none());
}

#[test]
fn missing_folder_gives_empty_index_with_a_problem() {
    let info = WilayahIndex::load_dir(&temp_dir("none").join("tidak-ada")).info();
    assert_eq!(info.desa, 0);
    assert_eq!(info.problems.len(), 1);
}

fn place() -> Place {
    Place {
        code: "35.01.04.1013".into(),
        desa: "Ploso".into(),
        kecamatan: "Pacitan".into(),
        kotkab: "Kabupaten Pacitan".into(),
        provinsi: "Jawa Timur".into(),
    }
}

const BMKG: &str = r#"{
  "lokasi": {"lon": 111.1, "lat": -8.2, "timezone": "Asia/Jakarta"},
  "data": [{
    "lokasi": {},
    "cuaca": [
      [
        {"utc_datetime": "2026-11-25 03:00:00", "local_datetime": "2026-11-25 10:00:00",
         "t": 31, "hu": "70", "ws": 7.5, "wd": "SW", "wd_deg": 225, "tcc": 60, "tp": 0.2,
         "vs": 8000, "vs_text": "> 8 km", "weather_desc": "Berawan", "weather_desc_en": "Mostly Cloudy",
         "analysis_date": "2026-11-25T00:00:00"},
        {"utc_datetime": "2026-11-25 00:00:00", "local_datetime": "2026-11-25 07:00:00",
         "t": 27, "hu": 85, "weather_desc": "Hujan Petir", "weather_desc_en": "Thunderstorm"}
      ],
      [
        {"utc_datetime": "2026-11-26 03:00:00", "local_datetime": "2026-11-26 10:00:00",
         "weather_desc_en": "Sunny"},
        {"local_datetime": "tanggal-rusak"}
      ]
    ]
  }]
}"#;

#[test]
fn bmkg_parse_flattens_sorts_and_derives_fields() {
    let snapshot = bmkg_client::parse(BMKG, &place(), 123).unwrap();

    assert_eq!(snapshot.utc_offset_s, 7 * 3600);
    assert_eq!(snapshot.fetched_at, 123);
    assert_eq!(snapshot.analysis_date.as_deref(), Some("2026-11-25T00:00:00"));
    assert_eq!(snapshot.slots.len(), 3); // Slot tanpa waktu valid dilewati; sisanya terurut.
    assert!(snapshot.slots.windows(2).all(|w| w[0].time_ms <= w[1].time_ms));

    let first = &snapshot.slots[0];
    assert_eq!(first.time, "2026-11-25 07:00");
    assert_eq!(first.kind, "thunder");
    assert_eq!(first.humidity, Some(85.0));

    let second = &snapshot.slots[1];
    assert_eq!(second.temp_c, Some(31.0));
    assert_eq!(second.humidity, Some(70.0)); // dari teks "70"
    assert_eq!(second.wind_kmh, Some(7.5));
    assert_eq!(second.wind_from.as_deref(), Some("SW"));
    assert_eq!(second.kind, "cloudy");
    assert!(second.feels_like_c.unwrap() >= 31.0);

    assert_eq!(snapshot.slots[2].kind, "clear");
    assert_eq!(snapshot.slots[2].temp_c, None);
    assert_eq!(snapshot.slots[2].feels_like_c, None);

    assert_eq!(snapshot.sun.len(), 2);
    assert!(snapshot.sun[0].sunrise_ms < snapshot.sun[0].sunset_ms);
}

#[test]
fn bmkg_parse_rejects_unusable_responses() {
    assert!(bmkg_client::parse("bukan json", &place(), 0).is_err());
    assert!(bmkg_client::parse("{}", &place(), 0).is_err());
    assert!(bmkg_client::parse(r#"{"data":[{"cuaca":[]}]}"#, &place(), 0).is_err());
}