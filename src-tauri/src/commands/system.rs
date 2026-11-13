use tauri::State;

use crate::core::error::AppError;
use crate::models::system::{
    SystemInfo,
    SystemStats
};
use crate::services::system_monitor::SystemMonitor;

/// Info statis (OS, hostname, CPU). Cukup dipanggil sekali.
#[tauri::command]
pub async fn get_system_info(monitor: State<'_, SystemMonitor>) -> Result<SystemInfo, String> {
    Ok(monitor.info().clone())
}

/// Snapshot pemakaian terkini. Dipanggil berkala oleh frontend.
#[tauri::command]
pub async fn get_system_stats(monitor: State<'_, SystemMonitor>) -> Result<SystemStats, AppError> {
    monitor.snapshot()
}