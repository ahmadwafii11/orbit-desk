# Perhitungan Cuaca (`services/weather_calc.rs`)

Dokumen ini menjelaskan tiga perhitungan yang dilakukan Orbit Desk sendiri di
atas data BMKG, ditambah satu helper zona waktu. File `src-tauri/src/services/weather_calc.rs` bersifat
**murni**: tanpa jaringan, tanpa disk, tanpa state. Fungsinya hanya mengubah
angka menjadi angka (atau teks), sehingga mudah diuji
(`tests/weather_calc_test.rs`).

| Fungsi | Masukan | Keluaran | Dipakai untuk |
|---|---|---|---|
| `heat_index_c` | suhu °C, kelembapan % | suhu terasa °C | "Feels like" |
| `sun_times` | tanggal, lintang, bujur | terbit & terbenam (epoch detik, UTC) | Sunrise / Sunset, ikon siang/malam |
| `classify` | deskripsi BMKG (EN + ID) | golongan cuaca (teks) | Memilih ikon |
| `timezone_offset_seconds` | nama zona | selisih dari UTC (detik) | Cadangan bila `utc_datetime` tidak ada |

## Apa yang datang dari BMKG, apa yang dihitung sendiri

API prakiraan BMKG (`/publik/prakiraan-cuaca?adm4=<kode>`) memberi suhu (`t`),
kelembapan (`hu`), angin (`ws`, `wd`), tutupan awan (`tcc`), curah hujan (`tp`),
jarak pandang (`vs`), dan deskripsi cuaca. BMKG **tidak** memberi suhu terasa,
terbit/terbenam, UV, tekanan udara, atau peluang hujan.

Karena itu, **suhu terasa, terbit, dan terbenam adalah estimasi hasil hitungan
Orbit Desk, bukan data BMKG**. Halaman Weather menandainya ("Estimated…",
"Calculated").

---

## 1. Suhu terasa: `heat_index_c(temp_c, humidity)`

Memakai **heat index** dari regresi Rothfusz (dipakai US National Weather
Service). Rumus aslinya bekerja dalam °F, jadi ada konversi di awal dan akhir.

### Langkah

1. Ubah suhu ke Fahrenheit: `T = temp_c × 9/5 + 32`.
2. **Jika `T < 80 °F` (sekitar 26,7 °C), kembalikan `temp_c` apa adanya.**
   Regresi ini tidak berlaku untuk suhu sejuk; di bawah ambang itu "terasa"
   dianggap sama dengan suhu sebenarnya.
3. Batasi kelembapan ke 0–100: `RH = clamp(humidity, 0, 100)`.
4. Hitung:

   ```
   HI = -42.379
        + 2.04901523   · T
        + 10.14333127  · RH
        - 0.22475541   · T · RH
        - 0.00683783   · T²
        - 0.05481717   · RH²
        + 0.00122874   · T² · RH
        + 0.00085282   · T · RH²
        - 0.00000199   · T² · RH²
   ```

5. Ubah kembali ke Celsius: `(HI − 32) × 5/9`.
6. **Hasil tidak pernah lebih rendah dari suhu sebenarnya** (`max(hasil, temp_c)`).
   Di tepi rentang rumus (misalnya udara sangat kering) regresi bisa memberi
   angka sedikit di bawah suhu asli, dan itu tidak masuk akal secara fisik.

### Contoh

| Suhu | Kelembapan | Suhu terasa |
|---|---|---|
| 25 °C | 90 % | 25,0 °C (di bawah ambang, tidak diubah) |
| 27 °C | 80 % | ≈ 29,7 °C |
| 30 °C | 50 % | ≈ 31,0 °C |
| 32 °C | 70 % | ≈ 40,4 °C |
| 34 °C | 60 % | ≈ 42,2 °C |

### Batasan

- Hanya estimasi. Angin, sinar matahari langsung, dan tubuh tiap orang tidak
  diperhitungkan.
- Rumus dikalibrasi untuk kondisi panas-lembap; di Indonesia itu kasus yang
  paling relevan, tetapi di luar rentangnya hasil kurang akurat.
- Di UI, "Feels like" di kartu utama hanya ditampilkan bila selisihnya dengan
  suhu asli minimal 1 °C. Saran "terasa sangat panas" muncul mulai 38 °C.

---

## 2. Terbit & terbenam: `sun_times(year, month, day, lat, lon)`

Memakai **sunrise equation** (varian dari Astronomical Almanac / Wikipedia
"Sunrise equation"). Akurat sekitar satu sampai dua menit untuk lintang rendah
seperti Indonesia, cukup untuk tampilan jam terbit/terbenam.

Masukan: tanggal sipil **lokal**, lintang `lat` (derajat, selatan negatif), dan
bujur `lon` (derajat, timur positif). Koordinat berasal dari bagian `lokasi`
pada respons BMKG. Keluaran: dua epoch detik UTC `(terbit, terbenam)`, atau
`None` bila matahari tidak terbit/terbenam pada hari itu (wilayah kutub).

### Langkah

1. **Hari sejak J2000.** `n = days_from_civil(tanggal) − 10957`
   (10957 = jumlah hari dari 1970-01-01 ke 2000-01-01).
2. **Koreksi bujur.** `mean_noon = n − lon / 360`. Setiap 15° bujur kira-kira
   satu jam, sehingga ini memilih siang matahari terdekat pada bujur lokasi.
3. **Anomali rata-rata matahari.**
   `A = (357.5291 + 0.98560028 · mean_noon) mod 360`, lalu `M = A` dalam radian.
4. **Persamaan pusat** (koreksi orbit lonjong bumi):
   `C = 1.9148·sin M + 0.02·sin 2M + 0.0003·sin 3M`.
5. **Bujur ekliptika.** `λ = (A + C + 180 + 102.9372) mod 360`.
6. **Transit matahari** (siang matahari sejati, dalam hari Julian):
   `J = 2451545 + mean_noon + 0.0053·sin M − 0.0069·sin 2λ`.
7. **Deklinasi matahari.** `sin δ = sin λ · sin 23.4397°`.
8. **Sudut jam** saat matahari menyentuh cakrawala:

   ```
   cos H = ( sin(−0.833°) − sin(lat) · sin δ ) / ( cos(lat) · cos δ )
   ```

   `−0.833°` adalah gabungan pembiasan atmosfer dan jari-jari piringan matahari,
   yaitu definisi standar "terbit/terbenam". Bila `cos H` di luar −1…1, matahari
   tidak terbit atau tidak terbenam, dan fungsi mengembalikan `None`.
9. **Hasil.** `H = acos(cos H)` dalam derajat.
   `terbit = J − H/360`, `terbenam = J + H/360` (hari Julian), lalu diubah ke
   epoch detik: `(julian − 2440587.5) × 86400`.

### Contoh

Jakarta (−6,2; 106,85), 25 November 2026:

- Terbit ≈ **05:27 WIB**, terbenam ≈ **17:53 WIB**.
- `H ≈ 93,2°`, jadi siang sekitar 12 jam 26 menit.

Tes memeriksa rentang 05:22–05:30 dan 17:48–17:56 WIB terhadap algoritma
referensi NOAA, dan memeriksa bahwa di kutub hasilnya `None`.

### Kegunaan di app

- Menampilkan **Sunrise** dan **Sunset** (`bmkg_client::sun_days` memanggil
  fungsi ini untuk setiap tanggal lokal yang ada di prakiraan).
- Menentukan **siang atau malam** untuk ikon: pada malam hari ikon matahari
  diganti bulan (`isNight` di `utils.ts`). Bila data terbit/terbenam tidak ada,
  dipakai aturan cadangan: malam = sebelum jam 06 atau mulai jam 18.

### Catatan

- Tanggal yang dipakai adalah tanggal **lokal** lokasi, lalu hasilnya dikonversi
  ke UTC. Bujur ikut menentukan hasil, sehingga Jawa Timur dan Papua
  mendapat jam yang berbeda untuk tanggal yang sama.
- Hasil tidak memperhitungkan ketinggian tempat atau pegunungan yang menutupi
  cakrawala.

---

## 3. Penggolongan cuaca: `classify(desc_en, desc_id)`

BMKG memberi deskripsi teks ("Hujan Ringan" / "Light Rain"), bukan kode ikon.
Fungsi ini mengubahnya menjadi satu dari sebelas golongan yang dipakai UI untuk
memilih ikon, saran singkat, dan ringkasan harian.

### Cara kerja

1. Gabungkan deskripsi Inggris dan Indonesia menjadi satu teks huruf kecil.
2. Periksa daftar kata kunci **berurutan dari atas ke bawah**; golongan pertama
   yang cocok dipakai.

| Urutan | Golongan | Kata kunci (EN / ID) |
|---|---|---|
| 1 | `thunder` | thunder, petir |
| 2 | `heavy_rain` | heavy rain, hujan lebat, sangat lebat |
| 3 | `drizzle` | light rain, hujan ringan, drizzle, gerimis |
| 4 | `rain` | rain, hujan, shower |
| 5 | `fog` | fog, kabut, mist |
| 6 | `haze` | haze, kabur, asap, smoke |
| 7 | `overcast` | overcast, berawan tebal |
| 8 | `partly` | partly, cerah berawan |
| 9 | `cloudy` | cloudy, berawan |
| 10 | `clear` | clear, cerah, sunny |
| 11 | `unknown` | tidak ada yang cocok |

### Mengapa urutan penting

- "Hujan Petir" mengandung "hujan" **dan** "petir"; `thunder` harus dicek lebih
  dulu supaya tidak jatuh ke `rain`.
- "Hujan Lebat" dan "Hujan Ringan" harus dicek sebelum kata umum "hujan".
- "Cerah Berawan" mengandung "berawan"; `partly` dicek sebelum `cloudy`.
- "Berawan Tebal" dicek sebelum "berawan" untuk alasan yang sama.

### Contoh

| BMKG (ID / EN) | Golongan |
|---|---|
| Hujan Petir / Thunderstorm | `thunder` |
| Hujan Lebat / Heavy Rain | `heavy_rain` |
| Hujan Ringan / Light Rain | `drizzle` |
| Hujan Sedang / Moderate Rain | `rain` |
| Kabut / Fog | `fog` |
| Cerah Berawan / Partly Cloudy | `partly` |
| Berawan / Mostly Cloudy | `cloudy` |
| Cerah / Sunny | `clear` |
| (teks baru yang belum dikenal) | `unknown` |

### Bila BMKG menambah deskripsi baru

Cukup tambahkan kata kuncinya ke daftar yang sesuai, **pada posisi yang benar**
dalam urutan di atas, lalu tambahkan satu baris di tes klasifikasi. Golongan
`unknown` tampil sebagai ikon awan polos, jadi deskripsi yang belum dikenal
tidak merusak tampilan.

---

## 4. Zona waktu: `timezone_offset_seconds(timezone)`

| Nama zona | Selisih |
|---|---|
| `Asia/Makassar`, `WITA`, `+0800`, `+08:00` | +8 jam (28.800 detik) |
| `Asia/Jayapura`, `WIT`, `+0900`, `+09:00` | +9 jam (32.400 detik) |
| lainnya (termasuk `Asia/Jakarta`, WIB) | +7 jam (25.200 detik) |

Ini hanya **cadangan**. Saat parsing (`bmkg_client::parse`), selisih zona
dihitung dari pasangan `local_datetime − utc_datetime` pada slot pertama, karena
itu lebih tepercaya daripada menebak dari nama zona. Fungsi ini baru dipakai bila
respons tidak menyertakan `utc_datetime`.

---

## Alur data ringkas

```
BMKG JSON ──► bmkg_client::parse
                 ├─ tiap slot: classify(desc_en, desc_id)        → kind
                 ├─ tiap slot: heat_index_c(t, hu)               → feelsLikeC
                 ├─ selisih zona dari local − utc                → utcOffsetS
                 └─ tiap tanggal: sun_times(tgl, lat, lon)       → sunriseMs / sunsetMs
                              │
                              ▼
                    WeatherSnapshot ──► frontend (ikon, "Feels like", Sunrise/Sunset)
```

## Pengujian

`tests/weather_calc_test.rs` memuat 7 tes:

1. Konversi tanggal sipil ↔ hari (bolak-balik dan titik acuan).
2. Pembacaan format tanggal-waktu BMKG.
3. Suhu terasa naik saat panas-lembap dan tidak turun di bawah ambang.
4. Terbit/terbenam Jakarta masuk akal (terhadap NOAA).
5. Terbit/terbenam di kutub bisa tidak ada.
6. Tabel klasifikasi kondisi BMKG.
7. Zona waktu Indonesia.

Jalankan dengan `cargo test weather_calc` di folder `src-tauri`.