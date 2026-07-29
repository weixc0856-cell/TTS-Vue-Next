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
            // Initialize storage
            let app_data = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data).expect("Failed to create app data dir");
            let db_path = get_db_path(&app_data);

            log::info!("Initializing database at: {:?}", db_path);
            let storage = Storage::open(
                db_path.to_str().expect("Invalid db path"),
            )
            .expect("Failed to initialize database");

            // Initialize app state
            let app_state = AppState::new(storage);
            app.manage(app_state);

            log::info!("App initialized successfully");
            Ok(())
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
