//! Berguna untuk mengatur antrean dalam memutar music: urutan lagu, posisi sekarang, shuffle, dan repeat.
//! Logika murni tanpa audio, jadi mudah diuji.
use crate::models::music::RepeatMode;

pub struct PlayQueue {
    ids: Vec<String>,
    order: Vec<usize>, // Urutan putar sebagai indeks ke `ids` (diacak jika shuffle).
    pos: usize,
    shuffled: bool,
    rng: u64,
}

impl PlayQueue {
    pub fn new(seed: u64) -> Self {
        Self {
            ids: Vec::new(),
            order: Vec::new(),
            pos: 0,
            shuffled: false,
            rng: seed | 1, // xorshift tidak boleh 0.
        }
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn clear(&mut self) {
        self.ids.clear();
        self.order.clear();
        self.pos = 0;
    }

    pub fn current(&self) -> Option<&str> {
        self.order
            .get(self.pos)
            .and_then(|&i| self.ids.get(i))
            .map(String::as_str)
    }

    fn next_random(&mut self) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.rng
    }

    fn shuffle_order(&mut self) {
        for i in (1..self.order.len()).rev() {
            let j = (self.next_random() % (i as u64 + 1)) as usize;
            self.order.swap(i, j);
        }
    }

    // Menyusun ulang `order`. 
    // Lagu `first` (jika ada) akan selalu jadi yang pertama saat shuffle, agar lagu yang sedang diputar tidak berpindah.
    fn rebuild(&mut self, first: Option<usize>) {
        self.order = (0..self.ids.len()).collect();

        if self.shuffled {
            self.shuffle_order();
            if let Some(first) = first {
                if let Some(at) = self.order.iter().position(|&i| i == first) {
                    self.order.swap(0, at);
                }
            }
            self.pos = 0;
        } else {
            self.pos = first.unwrap_or(0);
        }
    }

    // Mengganti antrean dan mulai dari `start_id`. 
    // `false` jika `start_id` tidak ada di `ids` (antrean tidak diubah).
    pub fn set(&mut self, ids: Vec<String>, start_id: &str, shuffle: bool) -> bool {
        let Some(start) = ids.iter().position(|id| id == start_id) else {
            return false;
        };

        self.ids = ids;
        self.shuffled = shuffle;
        self.rebuild(Some(start));
        true
    }

    /// Menyalakan / mematikan shuffle tanpa mengganti lagu yang sedang diputar.
    pub fn set_shuffle(&mut self, shuffle: bool) {
        if self.shuffled == shuffle {
            return;
        }

        let current = self.order.get(self.pos).copied();
        self.shuffled = shuffle;
        self.rebuild(current);
    }

    // Maju satu lagu jika lagu selesai. 
    // `auto` = lagu selesai sendiri (bukan tombol next).
    // `false` = tidak terdapat lagu berikutnya (lagu berada di list terakhir).
    pub fn advance(&mut self, repeat: RepeatMode, auto: bool) -> bool {
        if self.order.is_empty() {
            return false;
        }

        if repeat == RepeatMode::One && auto {
            return true;
        }

        if self.pos + 1 < self.order.len() {
            self.pos += 1;
            return true;
        }

        if repeat == RepeatMode::Off {
            return false;
        }

        // Putaran lagu baru. 
        // Saat shuffle, mengacak ulang lagu agar urutannya tidak sama.
        if self.shuffled {
            self.shuffle_order();
        }
        self.pos = 0;
        true
    }

    // Mundur satu lagu. 
    // `false` = sudah di awal dan tidak terdapat repeat.
    pub fn retreat(&mut self, repeat: RepeatMode) -> bool {
        if self.order.is_empty() {
            return false;
        }

        if self.pos > 0 {
            self.pos -= 1;
            return true;
        }

        if repeat == RepeatMode::Off {
            return false;
        }

        self.pos = self.order.len() - 1;
        true
    }
}