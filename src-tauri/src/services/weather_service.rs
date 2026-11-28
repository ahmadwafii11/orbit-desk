//! Berguna untuk mengatur tentang Weather: lokasi tersimpan, cache prakiraan, dan pengambilan dari BMKG.
//! Semua method di sini blocking (disk/jaringan); 
//! Command memanggilnya lewat `spawn_blocking`. 
//! Mutex hanya dipegang sebentar, tidak pernah selama permintaan jaringan.
use std::sync::Mutex;

use crate::core::{
    error::AppError, 
    time::now_ms,
};
use crate::models::weather::{
    PlacesState, 
    WeatherCache, 
    WeatherResult, 
    WilayahHit, 
    WilayahInfo, 
    WilayahItem,
};
use crate::repositories::json_file::JsonFile;
use crate::services::{
    bmkg_client, 
    wilayah_index::WilayahIndex,
};

pub const MAX_PLACES: usize = 10;
const FRESH_MS: i64 = 30 * 60 * 1000; // BMKG akan memperbarui prakiraan cuaca 2x sehari; 30 menit sudah lebih dari cukup fresh.
const MIN_FORCE_GAP_MS: i64 = 30 * 1000; // Tombol "refresh" tidak mengulang permintaan untuk lokasi yang sama lebih cepat dari ini (melindungi batas 60 permintaan/menit/IP).

struct Inner {
    places: PlacesState,
    cache: WeatherCache,
}

pub struct WeatherService {
    index: WilayahIndex,
    places_file: JsonFile,
    cache_file: JsonFile,
    inner: Mutex<Inner>,
}

impl WeatherService {
    pub fn new(
        index: WilayahIndex,
        places_file: JsonFile,
        cache_file: JsonFile,
    ) -> Result<Self, AppError> {
        let places: PlacesState = places_file.load()?;
        let cache: WeatherCache = cache_file.load()?;

        Ok(Self {
            index,
            places_file,
            cache_file,
            inner: Mutex::new(Inner { places, cache }),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Inner>, AppError> {
        self.inner.lock().map_err(|e| AppError::Lock(e.to_string()))
    }

    // ---- Wilayah ----
    pub fn wilayah_info(&self) -> WilayahInfo {
        self.index.info()
    }

    pub fn wilayah_children(&self, parent: &str) -> Vec<WilayahItem> {
        self.index.children(parent)
    }

    pub fn wilayah_search(&self, query: &str) -> Vec<WilayahHit> {
        self.index.search(query, 30)
    }

    // ---- Lokasi tersimpan ----
    pub fn places(&self) -> Result<PlacesState, AppError> {
        Ok(self.lock()?.places.clone())
    }

    pub fn add_place(&self, code: &str) -> Result<PlacesState, AppError> {
        let place = self
            .index
            .find_place(code)
            .ok_or_else(|| AppError::PlaceNotFound(code.to_string()))?;

        let mut inner = self.lock()?;
        if inner.places.places.iter().any(|p| p.code == place.code) {
            return Err(AppError::AlreadyAdded);
        }
        if inner.places.places.len() >= MAX_PLACES {
            return Err(AppError::TooManyPlaces(MAX_PLACES));
        }

        let first = inner.places.places.is_empty();
        let new_code = place.code.clone();
        inner.places.places.push(place);
        if first || inner.places.default_code.is_none() {
            inner.places.default_code = Some(new_code);
        }

        self.places_file.save(&inner.places)?;
        Ok(inner.places.clone())
    }

    pub fn remove_place(&self, code: &str) -> Result<PlacesState, AppError> {
        let mut inner = self.lock()?;
        inner.places.places.retain(|p| p.code != code);

        if inner.places.default_code.as_deref() == Some(code) {
            inner.places.default_code = inner.places.places.first().map(|p| p.code.clone());
        }
        inner.cache.entries.remove(code);

        self.places_file.save(&inner.places)?;
        self.cache_file.save(&inner.cache)?;
        Ok(inner.places.clone())
    }

    pub fn set_default(&self, code: &str) -> Result<PlacesState, AppError> {
        let mut inner = self.lock()?;
        if !inner.places.places.iter().any(|p| p.code == code) {
            return Err(AppError::PlaceNotFound(code.to_string()));
        }
        inner.places.default_code = Some(code.to_string());
        self.places_file.save(&inner.places)?;
        Ok(inner.places.clone())
    }

    // ---- Prakiraan ----

    // Prakiraan untuk `code` (atau lokasi bawaan bila kosong).
    // Menggunakan cache apabila masih refresh; 
    // jika pengambilan gagal tetapi terdapat cache lama, mengembalikan cache itu dengan `stale = true`.
    pub fn get(&self, code: Option<&str>, force: bool) -> Result<WeatherResult, AppError> {
        let (place, cached) = {
            let inner = self.lock()?;
            let code = match code {
                Some(code) => code.to_string(),
                None => inner
                    .places
                    .default_code
                    .clone()
                    .or_else(|| inner.places.places.first().map(|p| p.code.clone()))
                    .ok_or_else(|| AppError::PlaceNotFound("belum ada lokasi".into()))?,
            };
            let place = inner
                .places
                .places
                .iter()
                .find(|p| p.code == code)
                .cloned()
                .ok_or(AppError::PlaceNotFound(code.clone()))?;
            let cached = inner.cache.entries.get(&code).cloned();
            (place, cached)
        };

        let now = now_ms();
        if let Some(snapshot) = &cached {
            let age = now - snapshot.fetched_at;
            let fresh = age < FRESH_MS && !force;
            let too_soon = force && age < MIN_FORCE_GAP_MS;
            if fresh || too_soon {
                return Ok(WeatherResult {
                    snapshot: snapshot.clone(),
                    stale: false,
                    error: None,
                });
            }
        }

        let fetched =
            bmkg_client::fetch(&place.code).and_then(|raw| bmkg_client::parse(&raw, &place, now));

        match fetched {
            Ok(snapshot) => {
                let mut inner = self.lock()?;
                inner
                    .cache
                    .entries
                    .insert(snapshot.code.clone(), snapshot.clone());
                // Gagal menyimpan cache tidak boleh menggagalkan tampilan.
                if let Err(error) = self.cache_file.save(&inner.cache) {
                    eprintln!("cache cuaca tidak tersimpan: {error}");
                }
                Ok(WeatherResult {
                    snapshot,
                    stale: false,
                    error: None,
                })
            }
            Err(error) => match cached {
                Some(snapshot) => Ok(WeatherResult {
                    snapshot,
                    stale: true,
                    error: Some(error.to_string()),
                }),
                None => Err(error),
            },
        }
    }
}