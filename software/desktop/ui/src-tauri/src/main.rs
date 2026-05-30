// Tucklet desktop GUI — Tauri commands.
// Thin bridge: every command delegates to the verified tucklet-desktop-core. The
// React frontend calls these via `invoke`.
//
// NOTE: not compiled in this repo's CI sandbox (the tauri crate needs the
// platform webview + system libraries). Build with the Node toolchain in your
// environment. The logic it calls (core) IS compiled + tested.
//
// License: PolyForm Noncommercial 1.0.0

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use tucklet_desktop_core::proto::{DataTransport, MediaItem, RadioKind};
use tucklet_desktop_core::{b64, estimate, link, DataClient};

#[derive(Default)]
struct AppState {
    client: Mutex<Option<DataClient>>,
}

#[derive(serde::Serialize)]
struct StatusDto {
    free_bytes: u64,
    total_bytes: u64,
    item_count: usize,
}

/// Connect using a host + token obtained from the BLE handshake (the React
/// onboarding screen performs the handshake; here we accept the resulting
/// session). CONFIRM: move the BLE handshake into a command behind the `ble`
/// feature so the UI can do one-tap pairing without a manual token.
#[tauri::command]
fn connect(host: String, token: String, state: State<AppState>) -> Result<(), String> {
    *state.client.lock().unwrap() = Some(DataClient::new(host, token));
    Ok(())
}

#[tauri::command]
fn status(state: State<AppState>) -> Result<StatusDto, String> {
    with_client(&state, |c| {
        let m = c.manifest().map_err(|e| e.to_string())?;
        Ok(StatusDto {
            free_bytes: m.free_bytes,
            total_bytes: m.total_bytes,
            item_count: m.items.len(),
        })
    })
}

#[tauri::command]
fn library(state: State<AppState>) -> Result<Vec<MediaItem>, String> {
    with_client(&state, |c| {
        c.manifest().map(|m| m.items).map_err(|e| e.to_string())
    })
}

#[tauri::command]
fn thumbnail_b64(id: String, state: State<AppState>) -> Result<Option<String>, String> {
    with_client(&state, |c| {
        c.thumbnail(&id)
            .map(|opt| opt.map(|bytes| b64::encode(&bytes)))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
fn pull(id: String, out: String, state: State<AppState>) -> Result<u64, String> {
    with_client(&state, |c| {
        c.download(&id, &PathBuf::from(out)).map_err(|e| e.to_string())
    })
}

#[tauri::command]
fn push(file: String, item: MediaItem, state: State<AppState>) -> Result<(), String> {
    with_client(&state, |c| {
        c.upload(&PathBuf::from(file), &item).map_err(|e| e.to_string())
    })
}

#[tauri::command]
fn delete(id: String, state: State<AppState>) -> Result<(), String> {
    with_client(&state, |c| c.delete(&id).map_err(|e| e.to_string()))
}

#[derive(serde::Serialize)]
struct EstimateDto {
    seconds: u32,
    human: String,
    bytes_total: u64,
    files: u32,
}

#[tauri::command]
fn estimate_ids(ids: Vec<String>, state: State<AppState>) -> Result<EstimateDto, String> {
    with_client(&state, |c| {
        let m = c.manifest().map_err(|e| e.to_string())?;
        let sizes: Vec<u64> = m
            .items
            .iter()
            .filter(|it| ids.is_empty() || ids.contains(&it.id))
            .map(|it| it.size_bytes)
            .collect();
        let prof = link::profile_for(RadioKind::SingleC5, DataTransport::SoftAp);
        let est = estimate::estimate_sizes(&sizes, prof);
        Ok(EstimateDto {
            seconds: est.seconds,
            human: est.human(),
            bytes_total: est.bytes_total,
            files: est.files,
        })
    })
}

fn with_client<T, F>(state: &State<AppState>, f: F) -> Result<T, String>
where
    F: FnOnce(&DataClient) -> Result<T, String>,
{
    let guard = state.client.lock().unwrap();
    let c = guard.as_ref().ok_or("not connected")?;
    f(c)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            connect,
            status,
            library,
            thumbnail_b64,
            pull,
            push,
            delete,
            estimate_ids
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tucklet");
}

fn main() {
    run()
}
