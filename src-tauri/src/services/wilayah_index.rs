//! Berguna untuk mengatur Indeks wilayah administrasi (provinsi -> kab/kota -> kecamatan -> desa/kel) yang dibaca dari semua file `*.json` di satu folder. 
//! Satu file = satu provinsi yang berada dalam satu file JSON dengan struktur yang sama seperti `jatim_wilayah.json`.
//! Menambah provinsi = Menambahkan file JSON baru dalam folder tersebut. 
//! Bagian file JSON yang bermasalah seperti kode ganda, kode tidak cocok dengan induknya, dst. dapat dilewati dan dicatat di `problems`, bukan menggagalkan seluruh pemuatan.
use std::{
    collections::HashMap, 
    fs, 
    path::Path,
};
use serde::Deserialize;

use crate::models::weather::{
    Place, 
    WilayahHit, 
    WilayahInfo, 
    WilayahItem,
};

#[derive(Deserialize)]
struct RawProvince {
    kode_provinsi: String,
    nama_provinsi: String,
    #[serde(default)]
    kabupaten_kota: Vec<RawKabKota>,
}

#[derive(Deserialize)]
struct RawKabKota {
    kode_kabkota: String,
    nama_kabkota: String,
    #[serde(default)]
    kecamatan: Vec<RawKecamatan>,
}

#[derive(Deserialize)]
struct RawKecamatan {
    kode_kecamatan: String,
    nama_kecamatan: String,
    #[serde(default)]
    desa_kelurahan: Vec<RawDesa>,
}

#[derive(Deserialize)]
struct RawDesa {
    kode_wilayah: String,
    #[serde(default)]
    jenis_wilayah: String,
    nama_desakel: String,
}

struct Desa {
    code: String,
    name: String,
    jenis: String,
}

struct Kecamatan {
    code: String,
    name: String,
    desa: Vec<Desa>,
}

struct KabKota {
    code: String,
    name: String,
    kecamatan: Vec<Kecamatan>,
}

struct Province {
    code: String,
    name: String,
    kabkota: Vec<KabKota>,
}

pub struct WilayahIndex {
    provinces: Vec<Province>,
    desa_count: usize,
    problems: Vec<String>,
    dir: String,
}

// Example: "KABUPATEN PACITAN" -> "Kabupaten Pacitan". 
// Nama yang sudah campuran huruf besar-kecil akan dibiarkan.
fn prettify(name: &str) -> String {
    let name = name.trim();
    if name.chars().any(|c| c.is_lowercase()) {
        return name.to_string();
    }

    name.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first
                    .to_uppercase()
                    .chain(chars.flat_map(|c| c.to_lowercase()))
                    .collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// Mennyesuaikan `needle` (sudah huruf kecil) di `haystack` tanpa membedakan huruf besar-kecil serta tanpa membuat string baru.
fn contains_ci(haystack: &str, needle: &str) -> bool {
    let needle = needle.as_bytes();
    if needle.is_empty() {
        return true;
    }

    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

impl WilayahIndex {
    // Indeks kosong, dengan satu catatan masalah.
    pub fn empty(dir: &Path, problem: String) -> Self {
        Self {
            provinces: Vec::new(),
            desa_count: 0,
            problems: vec![problem],
            dir: dir.display().to_string(),
        }
    }

    // Membaca semua file yang berformat `*.json` di `dir`, urut nama file.
    pub fn load_dir(dir: &Path) -> Self {
        let Ok(entries) = fs::read_dir(dir) else {
            return Self::empty(
                dir,
                format!(
                    "Folder data wilayah tidak ditemukan: {}. Taruh file JSON provinsi di sana",
                    dir.display()
                ),
            );
        };

        let mut files: Vec<_> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("json"))
            })
            .collect();
        files.sort();

        let mut index = Self {
            provinces: Vec::new(),
            desa_count: 0,
            problems: Vec::new(),
            dir: dir.display().to_string(),
        };

        if files.is_empty() {
            index.problems.push(format!(
                "Belum ada file wilayah (*.json) di {}",
                dir.display()
            ));
        }

        let mut seen_desa: HashMap<String, String> = HashMap::new();

        for path in files {
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();

            let parsed = fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|raw| {
                    serde_json::from_str::<RawProvince>(&raw).map_err(|e| e.to_string())
                });

            match parsed {
                Ok(raw) => index.add_province(raw, &file_name, &mut seen_desa),
                Err(error) => index
                    .problems
                    .push(format!("{file_name}: tidak bisa dibaca ({error})")),
            }
        }

        index
    }

    fn add_province(
        &mut self,
        raw: RawProvince,
        file: &str,
        seen_desa: &mut HashMap<String, String>,
    ) {
        if self.provinces.iter().any(|p| p.code == raw.kode_provinsi) {
            self.problems.push(format!(
                "{file}: provinsi {} sudah dimuat dari file lain, dilewati",
                raw.kode_provinsi
            ));
            return;
        }

        let mut province = Province {
            code: raw.kode_provinsi,
            name: prettify(&raw.nama_provinsi),
            kabkota: Vec::new(),
        };

        for raw_kab in raw.kabupaten_kota {
            if !raw_kab
                .kode_kabkota
                .strip_prefix(&province.code)
                .is_some_and(|rest| rest.starts_with('.'))
            {
                self.problems.push(format!(
                    "{file}: kab/kota {} ({}) tidak cocok dengan provinsi {}, dilewati",
                    raw_kab.kode_kabkota, raw_kab.nama_kabkota, province.code
                ));
                continue;
            }

            if raw_kab.kecamatan.is_empty() {
                self.problems.push(format!(
                    "{file}: {} ({}) belum punya kecamatan",
                    raw_kab.nama_kabkota, raw_kab.kode_kabkota
                ));
            }

            let mut kab = KabKota {
                name: prettify(&raw_kab.nama_kabkota),
                code: raw_kab.kode_kabkota,
                kecamatan: Vec::new(),
            };

            for raw_kec in raw_kab.kecamatan {
                if !raw_kec
                    .kode_kecamatan
                    .strip_prefix(&kab.code)
                    .is_some_and(|rest| rest.starts_with('.'))
                {
                    self.problems.push(format!(
                        "{file}: kecamatan {} ({}) tidak cocok dengan {}, dilewati",
                        raw_kec.kode_kecamatan, raw_kec.nama_kecamatan, kab.code
                    ));
                    continue;
                }

                let mut kec = Kecamatan {
                    name: raw_kec.nama_kecamatan.trim().to_string(),
                    code: raw_kec.kode_kecamatan,
                    desa: Vec::new(),
                };

                for raw_desa in raw_kec.desa_kelurahan {
                    let name = raw_desa.nama_desakel.trim().to_string();

                    if !raw_desa
                        .kode_wilayah
                        .strip_prefix(&kec.code)
                        .is_some_and(|rest| rest.starts_with('.'))
                    {
                        self.problems.push(format!(
                            "{file}: desa {} ({name}) tidak cocok dengan kecamatan {} ({}), dilewati",
                            raw_desa.kode_wilayah, kec.code, kec.name
                        ));
                        continue;
                    }

                    if let Some(first) = seen_desa.get(&raw_desa.kode_wilayah) {
                        self.problems.push(format!(
                            "{file}: kode {} ganda: {name} (kec. {}) dan {first}. Yang kedua dilewati",
                            raw_desa.kode_wilayah, kec.name
                        ));
                        continue;
                    }

                    seen_desa.insert(
                        raw_desa.kode_wilayah.clone(),
                        format!("{name} (kec. {})", kec.name),
                    );

                    self.desa_count += 1;
                    kec.desa.push(Desa {
                        code: raw_desa.kode_wilayah,
                        name,
                        jenis: if raw_desa.jenis_wilayah.trim().is_empty() {
                            "Desa".to_string()
                        } else {
                            raw_desa.jenis_wilayah.trim().to_string()
                        },
                    });
                }

                if kec.desa.is_empty() {
                    self.problems.push(format!(
                        "{file}: kecamatan {} ({}) tidak punya desa/kelurahan",
                        kec.name, kec.code
                    ));
                }
                kab.kecamatan.push(kec);
            }

            province.kabkota.push(kab);
        }

        self.provinces.push(province);
    }

    pub fn info(&self) -> WilayahInfo {
        WilayahInfo {
            provinces: self.provinces.len(),
            desa: self.desa_count,
            problems: self.problems.clone(),
            dir: self.dir.clone(),
        }
    }

    // Isi dari satu tingkat. 
    // `parent` kosong = daftar provinsi; 
    // "35" = kab/kota di provinsi tersebut; 
    // "35.01" = kecamatannya; 
    // "35.01.04" = desanya atau kelurahannya.
    pub fn children(&self, parent: &str) -> Vec<WilayahItem> {
        let item =
            |code: &str, name: &str, level: &str, jenis: Option<&str>, has_children: bool| {
                WilayahItem {
                    code: code.to_string(),
                    name: name.to_string(),
                    level: level.to_string(),
                    jenis: jenis.map(str::to_string),
                    has_children,
                }
            };

        match parent.split('.').count() {
            _ if parent.is_empty() => self
                .provinces
                .iter()
                .map(|p| item(&p.code, &p.name, "provinsi", None, !p.kabkota.is_empty()))
                .collect(),
            1 => self
                .provinces
                .iter()
                .find(|p| p.code == parent)
                .map(|p| {
                    p.kabkota
                        .iter()
                        .map(|k| item(&k.code, &k.name, "kabkota", None, !k.kecamatan.is_empty()))
                        .collect()
                })
                .unwrap_or_default(),
            2 => self
                .provinces
                .iter()
                .flat_map(|p| &p.kabkota)
                .find(|k| k.code == parent)
                .map(|k| {
                    k.kecamatan
                        .iter()
                        .map(|c| item(&c.code, &c.name, "kecamatan", None, !c.desa.is_empty()))
                        .collect()
                })
                .unwrap_or_default(),
            3 => self
                .provinces
                .iter()
                .flat_map(|p| &p.kabkota)
                .flat_map(|k| &k.kecamatan)
                .find(|c| c.code == parent)
                .map(|c| {
                    c.desa
                        .iter()
                        .map(|d| item(&d.code, &d.name, "desa", Some(&d.jenis), false))
                        .collect()
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    // Mencari desa/kelurahan. 
    // Setiap kata pada `query` harus ada di salah satu nama pada hierarkinya (desa, kecamatan, kab/kota, provinsi), jadi semisal "ploso pacitan" menemukan Ploso di Pacitan.
    // Nama yang diawali kata pertama diurutkan lebih dulu.
    pub fn search(&self, query: &str, limit: usize) -> Vec<WilayahHit> {
        let tokens: Vec<String> = query.split_whitespace().map(|t| t.to_lowercase()).collect();
        if tokens.is_empty() || limit == 0 {
            return Vec::new();
        }

        // Mengumpulkan lebih banyak dari `limit` agar pengurutan berarti.
        let gather = limit.saturating_mul(8).max(200);
        let mut hits: Vec<WilayahHit> = Vec::new();

        'all: for province in &self.provinces {
            for kab in &province.kabkota {
                for kec in &kab.kecamatan {
                    for desa in &kec.desa {
                        let matches = tokens.iter().all(|token| {
                            contains_ci(&desa.name, token)
                                || contains_ci(&kec.name, token)
                                || contains_ci(&kab.name, token)
                                || contains_ci(&province.name, token)
                        });

                        if matches {
                            hits.push(WilayahHit {
                                code: desa.code.clone(),
                                desa: desa.name.clone(),
                                jenis: desa.jenis.clone(),
                                kecamatan: kec.name.clone(),
                                kotkab: kab.name.clone(),
                                provinsi: province.name.clone(),
                            });

                            if hits.len() >= gather {
                                break 'all;
                            }
                        }
                    }
                }
            }
        }

        let first = tokens[0].as_str();
        hits.sort_by_key(|h| {
            let starts = h.desa.to_lowercase().starts_with(first);
            (!starts, h.desa.to_lowercase())
        });
        hits.truncate(limit);
        hits
    }

    // Data lengkap sebuah desa untuk disimpan sebagai lokasi.
    pub fn find_place(&self, code: &str) -> Option<Place> {
        for province in &self.provinces {
            for kab in &province.kabkota {
                for kec in &kab.kecamatan {
                    if let Some(desa) = kec.desa.iter().find(|d| d.code == code) {
                        return Some(Place {
                            code: desa.code.clone(),
                            desa: desa.name.clone(),
                            kecamatan: kec.name.clone(),
                            kotkab: kab.name.clone(),
                            provinsi: province.name.clone(),
                        });
                    }
                }
            }
        }
        None
    }
}
