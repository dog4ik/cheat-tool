// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use cheat_toolkit::{
    process::{find_processes, ProcessListItem, ValueSize},
    Process,
};
use nix::unistd::Pid;
use tauri::State;

#[tauri::command]
fn get_process_list(query: &str) -> Vec<ProcessListItem> {
    let processes = find_processes(query);
    return processes;
}

#[tauri::command]
fn get_all_possible_memory(app_state: State<AppState>, sizing: usize) -> Result<Vec<u32>, String> {
    let sizing: ValueSize = sizing.try_into().unwrap();
    let process = app_state.process.lock().unwrap().clone();
    let mut result = Vec::new();
    if let Some(process) = process {
        for location in &process.memory {
            for bytes in location.data.windows(sizing.into()) {
                if let Ok(sized_bytes) = bytes.try_into() {
                    result.push(u32::from_le_bytes(sized_bytes));
                }
            }
        }
    };
    return Ok(result);
}

#[tauri::command]
fn get_process_memory(app_state: State<AppState>, sizing: usize) -> Result<Vec<u32>, String> {
    let sizing: ValueSize = sizing.try_into().unwrap();
    let process = app_state.process.lock().unwrap().clone();
    let mut result = Vec::new();
    if let Some(process) = process {
        for location in &process.memory {
            for bytes in location.data.windows(sizing.into()).step_by(sizing.into()) {
                if let Ok(sized_bytes) = bytes.try_into() {
                    result.push(u32::from_le_bytes(sized_bytes));
                }
            }
        }
    };
    return Ok(result);
}

#[tauri::command]
fn watch_memory(app_state: State<AppState>) {}

#[tauri::command]
fn set_desired_value(app_state: State<AppState>, value: usize) {
    let desired = &mut app_state.desired_value.lock().unwrap().unwrap();
    *desired = value;
}

#[tauri::command]
fn refresh_memory(app_state: State<AppState>) -> Result<(), String> {
    app_state
        .process
        .lock()
        .unwrap()
        .clone()
        .ok_or("failed to refresh memory")?
        .refresh_memory()
        .map_err(|_| "failed to refresh memory")?;

    return Ok(());
}

#[tauri::command]
fn select_process(app_state: State<AppState>, pid: i32) -> Result<String, String> {
    let pid = Pid::from_raw(pid);
    let process = Process::try_from(pid).map_err(|err| format!("{err}"))?;
    let id = process.pid.to_string().clone();
    *app_state.process.lock().unwrap() = Some(process);
    return Ok(id);
}

type ArcMutex<T> = Arc<Mutex<T>>;

struct AppState {
    process: ArcMutex<Option<Process>>,
    desired_value: ArcMutex<Option<usize>>,
}

fn main() {
    let app_state = AppState {
        process: Arc::new(Mutex::new(None)),
        desired_value: Arc::new(Mutex::new(None)),
    };
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            select_process,
            get_process_list,
            refresh_memory,
            get_process_memory,
            get_all_possible_memory,
            set_desired_value
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
