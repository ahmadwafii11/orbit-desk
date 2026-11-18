use serde::Serialize;
use tauri::{
    Emitter, 
    State, 
    WebviewWindow,
};

use crate::core::error::AppError;
use crate::models::notes::Note;
use crate::services::notes_service::NotesService;

// Nama event yang dikirim ke semua window setiap kali note berubah.
const NOTES_CHANGED_EVENT: &str = "notes-changed";

// `soure` = label window yang melakukan perubahan.
// Window itu sendiri sudah mempunyai datanya
// Jadi frontend akan mengabaikan event dari dirinya sendiri.
// Jika tidak, maka isi editor dapat tertimpa data yang lebih lama saat mengetik.
#[derive(Clone, Serialize)]
struct NotesChanged {
    source: String,
}

fn notify_changed(window: &WebviewWindow) {
    let payload = NotesChanged{
        source: window.label().to_string(),
    };

    // Gagal mengirimkan event tidak boleh menggagalkan penyimpanan.
    let _= window.emit(NOTES_CHANGED_EVENT, payload);
}

#[tauri::command]
pub async  fn list_notes(service: State<'_, NotesService>) -> Result<Vec<Note>, AppError> {
    service.list()
}

#[tauri::command]
pub async fn create_note(window: WebviewWindow, service: State<'_, NotesService>) -> Result<Note, AppError> {
    let note = service.create()?;
    notify_changed(&window);
    Ok(note)
}

#[tauri::command]
pub async fn save_note(window: WebviewWindow, service: State<'_, NotesService>, id: String, content: String) -> Result<Note, AppError> {
    let note = service.save_content(&id, content)?;
    notify_changed(&window);
    Ok(note)
}

#[tauri::command]
pub async fn set_note_pinned(window: WebviewWindow, service: State<'_, NotesService>, id: String, pinned: bool) -> Result<Note, AppError> {
    let note = service.set_pinned(&id, pinned)?;
    notify_changed(&window);
    Ok(note)
}

#[tauri::command]
pub async fn delete_note(window: WebviewWindow, service: State<'_, NotesService>, id: String) -> Result<(), AppError> {
    service.delete(&id)?;
    notify_changed(&window);
    Ok(())
}