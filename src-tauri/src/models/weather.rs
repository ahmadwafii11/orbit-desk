//! Tipe data modul Weather dan data wilayah. Cermin dari `src/types/weather.ts`.
use serde::{
    Deserialize, 
    Serialize
};

// Satu lokasi (desa/kelurahan) yang disimpan pengguna.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Place {
    pub code: String, // Kode wilayah tingkat IV berdasarkan kep. kemendagri tahun 2022, misalnya "35.01.04.1013".
    pub desa: String,
    pub kecamatan: String,
    pub kotkab: String,
    pub provinsi: String,
}

// Isi dari file `weather-places.json`.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlacesState {
    #[serde(default)]
    pub places: Vec<Place>,
    #[serde(default)]
    pub default_code: Option<String>,
}

// Satu slot prakiraan (Dimana BMKG memberikan satu slot per 3 jam).
// Semua nilai opsional: BMKG bisa saja tidak menyertakan sebuah field.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
    pub time: String, // Waktu lokal lokasi, "YYYY-MM-DD HH:MM".
    pub time_ms: i64, // Waktu slot sebagai epoch sebenarnya (UTC), milidetik. Menampilkan jam lokal dengan menambah `utcOffsetS` lalu diformat sebagai UTC.
    pub temp_c: Option<f64>,
    pub humidity: Option<f64>,
    pub wind_kmh: Option<f64>,
    pub wind_from: Option<String>, /// Arah dari angin berasal, misalnya "SW".
    pub wind_deg: Option<f64>,
    pub cloud_pct: Option<f64>,
    pub rain_mm: Option<f64>,
    pub visibility_m: Option<f64>,
    pub visibility_text: Option<String>,
    pub kind: String, // Golongan untuk ikon: clear, partly, cloudy, overcast, haze, fog, drizzle, rain, heavy_rain, thunder, unknown.
    pub desc_id: Option<String>,
    pub desc_en: Option<String>,
    pub feels_like_c: Option<f64>, // Suhu terasa (estimasinya dihitung di sini secara manual, bukan dari BMKG)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SunDay {
    pub date: String,  // Tanggal lokal (local date), "YYYY-MM-DD".
    pub sunrise_ms: i64,
    pub sunset_ms: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WeatherSnapshot {
    pub code: String,
    pub place: Place,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub utc_offset_s: i64, // Selisih waktu lokal dari UTC, (detik).
    pub fetched_at: i64, // Waktu app dalam mengambil suatu data ini (ms sejak epoch).
    pub analysis_date: Option<String>, // Waktu BMKG yang menghasilkan prakiraan, apa adanya dari respons.
    pub slots: Vec<Slot>,
    pub sun: Vec<SunDay>,
}

// Hasil dari `weather_get` 
// `stale` = data lama dari cache karena pengambilan terbaru gagal (mis. offline); `error` berisi alasannya.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WeatherResult {
    pub snapshot: WeatherSnapshot,
    pub stale: bool,
    pub error: Option<String>,
}

// Isi dari file `weather-cache.json`: prakiraan terakhir per lokasi tersimpan.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct WeatherCache {
    #[serde(default)]
    pub entries: std::collections::HashMap<String, WeatherSnapshot>,
}

// ---- Wilayah (Region) ----

// Satu baris saat menelusuri wilayah per tingkat.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WilayahItem {
    pub code: String,
    pub name: String,
    pub level: String, // "provinsi", "kabkota", "kecamatan", atau "desa".
    pub jenis: Option<String>, /// "Desa" / "Kelurahan" (yang hanya digunakan untuk tingkat desa).
    pub has_children: bool,
}

// Hasil dari pencarian desa: nama lengkap dengan hierarkinya, karena banyak desa bernama sama di tempat berbeda.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WilayahHit {
    pub code: String,
    pub desa: String,
    pub jenis: String,
    pub kecamatan: String,
    pub kotkab: String,
    pub provinsi: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WilayahInfo {
    pub provinces: usize,
    pub desa: usize,
    pub problems: Vec<String>, // Masalah yang ditemukan saat membaca file wilayah (kode ganda, kode  tidak cocok dengan induknya, dst.). Bagian bermasalah dapat dilewati.
    pub dir: String, // Direktori folder dimana tempat file JSON wilayah dibaca.
}