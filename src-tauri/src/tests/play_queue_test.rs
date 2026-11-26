use crate::models::music::RepeatMode;
use crate::services::play_queue::PlayQueue;

fn ids(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("t{i}")).collect()
}

fn drain(q: &mut PlayQueue, repeat: RepeatMode) -> Vec<String> {
    let mut seen = vec![q.current().unwrap().to_string()];
    while q.advance(repeat, true) && seen.len() < 100 {
        seen.push(q.current().unwrap().to_string());
        if repeat != RepeatMode::Off && seen.len() >= q.len() {
            break;
        }
    }
    seen
}

#[test]
fn set_dimulai_dari_lagu_yang_dipilih() {
    let mut q = PlayQueue::new(1);
    assert!(q.set(ids(5), "t2", false));
    assert_eq!(q.current(), Some("t2"));
    assert_eq!(drain(&mut q, RepeatMode::Off), ["t2", "t3", "t4"]);
}

#[test]
fn set_dengan_id_tidak_ada_tidak_mengubah_antrean() {
    let mut q = PlayQueue::new(1);
    q.set(ids(3), "t1", false);
    assert!(!q.set(ids(9), "zzz", false));
    assert_eq!(q.len(), 3);
    assert_eq!(q.current(), Some("t1"));
}

#[test]
fn shuffle_memutar_setiap_lagu_tepat_sekali_dan_mulai_dari_pilihan() {
    let mut q = PlayQueue::new(42);
    q.set(ids(20), "t7", true);
    assert_eq!(q.current(), Some("t7"));
    let mut seen = drain(&mut q, RepeatMode::Off);
    assert_eq!(seen.len(), 20);
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), 20);
}

#[test]
fn mengubah_shuffle_tidak_mengganti_lagu_sekarang() {
    let mut q = PlayQueue::new(7);
    q.set(ids(10), "t4", false);
    q.set_shuffle(true);
    assert_eq!(q.current(), Some("t4"));
    q.set_shuffle(false);
    assert_eq!(q.current(), Some("t4"));
    assert!(q.advance(RepeatMode::Off, false)); // Urutan kembali normal dari lagu sekarang.
    assert_eq!(q.current(), Some("t5"));
}

#[test]
fn repeat_off_berhenti_di_akhir_repeat_all_kembali_ke_awal() {
    let mut q = PlayQueue::new(1);
    q.set(ids(3), "t2", false);
    assert!(!q.advance(RepeatMode::Off, true));
    assert_eq!(q.current(), Some("t2"));
    assert!(q.advance(RepeatMode::All, true));
    assert_eq!(q.current(), Some("t0"));
}

#[test]
fn repeat_one_mengulang_saat_selesai_tapi_next_manual_tetap_maju() {
    let mut q = PlayQueue::new(1);
    q.set(ids(3), "t0", false);
    assert!(q.advance(RepeatMode::One, true));
    assert_eq!(q.current(), Some("t0"));
    assert!(q.advance(RepeatMode::One, false));
    assert_eq!(q.current(), Some("t1"));
}

#[test]
fn retreat_dan_batas_awal() {
    let mut q = PlayQueue::new(1);
    q.set(ids(3), "t1", false);
    assert!(q.retreat(RepeatMode::Off));
    assert_eq!(q.current(), Some("t0"));
    assert!(!q.retreat(RepeatMode::Off));
    assert!(q.retreat(RepeatMode::All));
    assert_eq!(q.current(), Some("t2"));
}

#[test]
fn antrean_kosong_aman() {
    let mut q = PlayQueue::new(1);
    assert_eq!(q.current(), None);
    assert!(!q.advance(RepeatMode::All, true));
    assert!(!q.retreat(RepeatMode::All));
    q.set_shuffle(true);
}