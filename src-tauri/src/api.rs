use std::sync::Mutex;

use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use tauri::{self, AppHandle, Manager, State};

use crate::{
    db::{ClientProcesses, ClientVariables, Db, DbSettings, DbVariables},
    process::{BufferValue, ProcessListItem, ValueSize, Variable},
    Process,
};

#[derive(Debug)]
struct AppState {
    process: Mutex<Option<Process>>,
    settings: Mutex<Settings>,
    db: Db,
}

//NOTE: Variable vs VariableWithValue vs BufferValue
#[derive(Serialize, Deserialize, Clone, Copy)]
struct VariableWithValue {
    size: usize,
    offset: usize,
    value: usize,
}

impl From<VariableWithValue> for Variable {
    fn from(val: VariableWithValue) -> Self {
        Variable {
            size: val.size.try_into().expect("to be valid"),
            position: val.offset,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Settings {
    value_size: ValueSize,
}

impl Default for Settings {
    fn default() -> Self {
        return Settings {
            value_size: ValueSize::U32,
        };
    }
}

//NOTE: Save settings vs update settings
// update settings updates rust settings
// save settings update only DB settings
#[tauri::command]
fn update_settings(app_state: State<AppState>, settings: Settings) {
    let mut app_settings = app_state.settings.lock().unwrap();
    *app_settings = settings;
}

#[tauri::command]
fn select_process(app_state: State<AppState>, pid: i32) -> Result<String, String> {
    let pid = nix::unistd::Pid::from_raw(pid);
    let process = Process::try_from(pid).map_err(|err| format!("{err}"))?;
    let id = process.pid.to_string().clone();
    *app_state.process.lock().unwrap() = Some(process);
    return Ok(id);
}

#[tauri::command]
fn get_current_process(app_state: State<AppState>) -> Option<Process> {
    app_state.process.lock().unwrap().clone()
}

#[tauri::command]
fn refresh_memory(app_state: State<AppState>) -> Result<(), &str> {
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
fn reset_state(app_state: State<AppState>) {
    let mut process = app_state.process.lock().unwrap();
    let mut settings = app_state.settings.lock().unwrap();
    *settings = Settings::default();
    *process = None;
}

#[tauri::command]
fn reset_values(app_state: State<AppState>) {
    let mut process = app_state.process.lock().unwrap();
    if let Some(process) = &mut *process {
        process.buffer = None;
    }
}

#[tauri::command]
async fn write_value(
    app_state: State<'_, AppState>,
    variable: Variable,
    value: usize,
) -> Result<(), &str> {
    let process = app_state.process.lock().unwrap();
    if let Some(process) = &*process {
        process
            .write(
                &Variable {
                    size: variable.size,
                    position: variable.position,
                },
                value,
            )
            .map_err(|_| "failed to write value")?;
    }
    Ok(())
}

#[tauri::command]
async fn watch_value(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
    position: usize,
    size: usize,
) -> Result<(), ()> {
    let process = app_state.process.lock().unwrap();
    let process = process.as_ref().unwrap();
    let pid = process.pid;
    tokio::spawn(async move {
        let (mut reciever, abort) = Process::watch_value(
            pid,
            &Variable {
                position,
                size: size.try_into().expect("to convert"),
            },
            10,
        )
        .await
        .unwrap();
        while let Some(val) = reciever.recv().await {
            app_handle.emit_all("value_update", val).unwrap();
        }
        app_handle.once_global("unlisten_value", move |_| abort.abort());
    });
    return Ok(());
}

#[tauri::command]
async fn get_neighbors(
    app_state: State<'_, AppState>,
    offset: usize,
) -> Result<Vec<VariableWithValue>, &str> {
    let capacity = 50;
    let process = app_state.process.lock().unwrap();
    if let Some(process) = &*process {
        let mut result = Vec::with_capacity(capacity);
        let start = offset - capacity / 2;
        let end = offset + capacity / 2;
        for i in start..end {
            let value = process
                .get_value(&Variable {
                    size: ValueSize::U32,
                    position: i,
                })
                .unwrap();
            result.push(VariableWithValue {
                offset: i,
                value: value as usize,
                size: 4,
            });
        }
        return Ok(result);
    } else {
        return Err("process is not defined");
    }
}

#[tauri::command]
fn get_process_memory(app_state: State<AppState>, sizing: usize) -> Result<Vec<u32>, String> {
    let sizing: ValueSize = sizing.try_into().unwrap();
    let process = app_state.process.lock().unwrap();
    let mut result = Vec::new();
    if let Some(process) = &*process {
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
fn expect_change(app_state: State<AppState>, is_changed: bool) -> Result<(), String> {
    let mut process = app_state.process.lock().unwrap();
    if let Some(process) = &mut *process {
        process
            .expect_change(is_changed)
            .map_err(|x| x.to_string())?;
    } else {
        return Err("Process is not defined".into());
    }
    return Ok(());
}

#[tauri::command]
fn get_process_list(query: &str) -> Vec<ProcessListItem> {
    let processes = crate::process::find_processes(query);
    return processes;
}

#[tauri::command]
fn scan_next(app_state: State<AppState>, value: usize) -> Result<Vec<BufferValue>, &str> {
    let mut process = app_state.process.lock().unwrap();
    if let Some(process) = &mut *process {
        Ok(process.scan_next(value))
    } else {
        Err("process is not defined")
    }
}

#[tauri::command]
fn populate_buffer_with_value(
    app_state: State<AppState>,
    value: usize,
    sizing: usize,
) -> Result<Vec<VariableWithValue>, String> {
    let mut process = app_state.process.lock().unwrap();
    if let Some(process) = &mut *process {
        let values = process.populate_buffer_with_value(value, sizing)?;
        return Ok(values
            .into_iter()
            .map(|x| VariableWithValue {
                value,
                offset: x.offset,
                size: sizing,
            })
            .collect());
    } else {
        return Err("process is not defined".into());
    }
}

// db stuff

#[tauri::command]
async fn save_variable(
    app_state: State<'_, AppState>,
    variable: ClientVariables,
) -> Result<(), &str> {
    app_state
        .db
        .save_variable(variable)
        .await
        .map_err(|_| "failed to save variable")
}

#[tauri::command]
async fn get_variable_by_id(app_state: State<'_, AppState>, id: i32) -> Result<DbVariables, &str> {
    app_state
        .db
        .get_variables_by_id(id)
        .await
        .map_err(|_| "failed to get variable")
}

#[tauri::command]
async fn delete_variable_by_id(app_state: State<'_, AppState>, id: i32) -> Result<(), &str> {
    app_state
        .db
        .remove_variable_by_id(id)
        .await
        .map_err(|_| "failed to delete variable")?;
    Ok(())
}

#[tauri::command]
async fn get_variables(
    app_state: State<'_, AppState>,
    take: Option<u32>,
    skip: Option<u32>,
) -> Result<(), &str> {
    app_state
        .db
        .get_variables(skip, take)
        .await
        .map_err(|_| "failed to get variables")?;
    Ok(())
}

#[tauri::command]
async fn save_process(
    app_state: State<'_, AppState>,
    process: ClientProcesses,
) -> Result<(), &str> {
    app_state
        .db
        .save_process(process)
        .await
        .map_err(|_| "failed to save process")?;
    Ok(())
}

#[tauri::command]
async fn delete_process(app_state: State<'_, AppState>, id: i32) -> Result<(), &str> {
    app_state
        .db
        .delete_process(id)
        .await
        .map_err(|_| "failed to save process")?;
    Ok(())
}

#[tauri::command]
async fn get_settings(app_state: State<'_, AppState>) -> Result<(), &str> {
    app_state
        .db
        .get_settings()
        .await
        .map_err(|_| "failed to get settings")?;
    Ok(())
}

#[tauri::command]
async fn save_settings(app_state: State<'_, AppState>, settings: DbSettings) -> Result<(), &str> {
    app_state
        .db
        .save_settings(settings)
        .await
        .map_err(|_| "failed to save settings")?;
    Ok(())
}

pub async fn run() {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env variable to be set");
    let db = Db::new(&db_url).await.expect("database to init");

    let app_state = AppState {
        process: Mutex::new(None),
        settings: Mutex::new(Settings::default()),
        db,
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            select_process,
            get_process_list,
            refresh_memory,
            get_process_memory,
            populate_buffer_with_value,
            reset_values,
            expect_change,
            scan_next,
            reset_state,
            write_value,
            watch_value,
            get_neighbors,
            update_settings,
            get_current_process,
            save_variable,
            get_variable_by_id,
            delete_variable_by_id,
            get_variables,
            save_process,
            delete_process,
            get_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
