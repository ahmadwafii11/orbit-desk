// Pemantauan sistem (CPU, RAM, swap, disk).
// `System` dan `Disk` disimpan di state Tauri (`SystemMonitor`) agar pembacaan CPU akurat.
// Pembacaa CPU akurat: sysinfo menghitung CPU usage dari selisih dua pembacaan.
// Jadi, objek yang sama harus di-refres berulang kali.
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};
use sysinfo::{
    Disks,
    System
};

use super::disk_collector::collect_disks;
use crate::core::error::AppError;
use crate::models::system::{
    CpuStats,
    DiskStats,
    MemoryStats,
    SystemInfo,
    SystemStats,
};

// Daftar disk jarang berubah dan refresh-nya relatif mahal.
// Jadi perlu dibaca ulang disetiap polling.
const DISK_REFRESH_INTERVAL: Duration = Duration::from_secs(10);

struct Inner {
    sys: System,
    /// `None` jika daftar disk gagal dibaca di OS ini.
    disks: Option<Disks>,
    disk_cache: Vec<DiskStats>,
    disks_refreshed_at: Instant,
}

pub struct SystemMonitor {
    info: SystemInfo,
    inner: Mutex<Inner>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        // Di sysinfo 0.40 konstruktor mengembalikan Result. 
        // Gagal di sini berarti OS tidak didukung sysinfo.
        // Jadi memang tidak ada gunanya untuk melanjutkan.
        let mut sys = System::new().expect("sysinfo: gagal menginisialisasi System");
        
        // Pembacaan awal supaya polling pertama dari frontend sudah punya pembanding untuk menghitung CPU usage.
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        let disks = Disks::new_with_refreshed_list().ok();
        let disk_cache = disks.as_ref().map(collect_disks).unwrap_or_default();

        let info = SystemInfo {
            os_name: System::name()
                .ok()
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| "Unknown".into()),
            os_version: System::os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            host_name: System::host_name().unwrap_or_default(),
            cpu_brand: sys
                .cpus()
                .first()
                .map(|cpu| cpu.brand().trim().to_string())
                .unwrap_or_default(),
            cpu_cores: sys.cpus().len(),
        };

        Self {
            info,
            inner: Mutex::new(Inner {
                sys,
                disks,
                disk_cache,
                disks_refreshed_at: Instant::now(),
            }),
        }
    }

    // Info statis (OS, hostname, CPU)
    // Dibaca sekali saat start.
    pub fn info(&self) -> &SystemInfo {
        &self.info
    }

    // Snapshot pemakaian terbaru.
    pub fn snapshot(&self) -> Result<SystemStats, AppError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|e| AppError::Lock(e.to_string()))?;
        Ok(inner.read())
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Inner {
    fn read(&mut self) -> SystemStats {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();

        if self.disks_refreshed_at.elapsed() >= DISK_REFRESH_INTERVAL {
            if let Some(disks) = self.disks.as_mut() {
                disks.refresh(true);
                self.disk_cache = collect_disks(disks);
            }
            self.disks_refreshed_at = Instant::now();
        }

        SystemStats {
            cpu: CpuStats {
                usage: percent(self.sys.global_cpu_usage()),
                per_core: self
                    .sys
                    .cpus()
                    .iter()
                    .map(|cpu| percent(cpu.usage()))
                    .collect(),
            },
            memory: MemoryStats {
                total: self.sys.total_memory(),
                used: self.sys.used_memory(),
                available: self.sys.available_memory(),
                swap_total: self.sys.total_swap(),
                swap_used: self.sys.used_swap(),
            },
            disks: self.disk_cache.clone(),
            uptime_secs: System::uptime().unwrap_or(0),
        }
    }
}

// Batasi ke 0-100 dan bulatkan satu desimal.
fn percent(value: f32) -> f32 {
    (value.clamp(0.0, 100.0) * 10.0).round() / 10.0
}