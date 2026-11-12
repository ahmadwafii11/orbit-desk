//! Tipe data modul System. Cermin dari `src/modules/system/types.ts`.
use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub host_name: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuStats {
    /// Pemakaian total, 0-100.
    pub usage: f32,
    /// Pemakaian per core/thread, 0-100.
    pub per_core: Vec<f32>,
}

/// Semua ukuran dalam byte.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiskStats {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    /// "SSD", "HDD", atau "Unknown".
    pub kind: &'static str,
    pub removable: bool,
    pub total: u64,
    pub available: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub cpu: CpuStats,
    pub memory: MemoryStats,
    pub disks: Vec<DiskStats>,
    pub uptime_secs: u64,
}