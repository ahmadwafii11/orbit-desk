//! Perhitungan tanggal sipil tanpa crate tambahan (algoritma Howard Hinnant).
//! Hanya yang dibutuhkan modul cuaca: mengurai "YYYY-MM-DD HH:MM:SS" serta mengubah tanggal menjadi hitungan hari sejak 1970-01-01.

// Hari sejak 1970-01-01 untuk tanggal sipil `y-m-d`.
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

// Kebalikan (reverse) dari `days_from_civil`: (tahun, bulan, hari).
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

// Example: "2026-11-25" => (2026, 11, 25).
pub fn parse_date(text: &str) -> Option<(i64, u32, u32)> {
    let mut parts = text.trim().split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;

    if parts.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some((y, m, d))
}

// "2026-11-25 13:00:00" atau "2026-11-25T06:00:00Z" -> detik sejak epoch, dengan menganggap waktunya UTC. 
// Pemanggil mengurangi/menambah offset zona waktu sendiri kalau waktunya waktu lokal.
pub fn parse_datetime(text: &str) -> Option<i64> {
    let text = text.trim().trim_end_matches('Z');
    let (date, time) = text
        .split_once(|c| c == ' ' || c == 'T')
        .unwrap_or((text, "00:00:00"));

    let (y, m, d) = parse_date(date)?;

    let mut parts = time.split(':');
    let hour: i64 = parts.next()?.parse().ok()?;
    let minute: i64 = parts.next().unwrap_or("0").parse().ok()?;
    let second: i64 = parts
        .next()
        .map(|s| s.split('.').next().unwrap_or("0"))
        .unwrap_or("0")
        .parse()
        .ok()?;

    if !(0..24).contains(&hour) || !(0..60).contains(&minute) || !(0..61).contains(&second) {
        return None;
    }

    Some(days_from_civil(y, m, d) * 86_400 + hour * 3_600 + minute * 60 + second)
}