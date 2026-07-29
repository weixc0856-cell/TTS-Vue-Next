mod audio;
mod commands;
mod edge_tts;
mod speech;
mod utils;

use std::path::PathBuf;

use commands::practice::AppState;
use storage::Storage;
use tauri::Manager;

/// Get the database path from app data directory
fn get_db_path(app_data: &PathBuf) -> PathBuf {
    app_data.join("practice.db")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Initialize storage — don't panic, log and continue
            match app.path().app_data_dir() {
                Ok(app_data) => {
                    if let Err(e) = std::fs::create_dir_all(&app_data) {
                        log::warn!("Failed to create app data dir: {e}. Using in-memory fallback.");
                        let storage = Storage::open_in_memory().expect("Failed to init in-memory DB");
                        let app_state = AppState::new(storage);
                        app.manage(app_state);
                        return Ok(());
                    }

                    let db_path = get_db_path(&app_data);
                    log::info!("Initializing database at: {:?}", db_path);

                    match Storage::open(db_path.to_str().unwrap_or("practice.db")) {
                        Ok(storage) => {
                            let app_state = AppState::new(storage);
                            app.manage(app_state);
                            log::info!("App initialized successfully");
                            Ok(())
                        }
                        Err(e) => {
                            log::warn!("Failed to open database: {e}. Using in-memory fallback.");
                            let storage = Storage::open_in_memory().expect("Failed to init in-memory DB");
                            let app_state = AppState::new(storage);
                            app.manage(app_state);
                            Ok(())
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to get app data dir: {e}. Using in-memory fallback.");
                    let storage = Storage::open_in_memory().expect("Failed to init in-memory DB");
                    let app_state = AppState::new(storage);
                    app.manage(app_state);
                    Ok(())
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Existing commands
            commands::tts::tts_convert,
            commands::tts::tts_stop,
            commands::voices::get_voices,
            commands::file::read_text_file,
            commands::file::select_folder,
            commands::file::show_in_folder,
            commands::file::write_binary_file,
            commands::file::remove_file,
            commands::audio::convert_audio_format,
            // Practice commands
            commands::practice::seed_content,
            commands::practice::list_exercises,
            commands::practice::get_exercise_detail,
            commands::practice::start_session,
            commands::practice::end_session,
            commands::practice::record_attempt,
            commands::practice::get_session,
            commands::practice::create_custom_exercise,
            commands::practice::transcribe_and_score,
            commands::practice::split_sentences,
            commands::practice::get_session_history,
            commands::practice::get_practice_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
