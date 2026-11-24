use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use tauri::Manager;

pub mod commands;
pub mod db;
pub mod duplicates;
pub mod media_info;
pub mod naming;
pub mod providers;
pub mod renamer;
pub mod scanner;

pub struct DbState(pub Arc<Mutex<Connection>>);
pub struct ImdbDbState(pub Arc<Mutex<Connection>>);
pub struct WatcherState(pub Arc<Mutex<scanner::watcher::LibraryWatcher>>);

pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            providers::imdb::downloader::clean_temp_dir(&app_data_dir);
            let conn = db::init_db(&app_data_dir).expect("failed to init db");
            let db_arc = Arc::new(Mutex::new(conn));
            app.manage(DbState(db_arc.clone()));

            let imdb_db_path = app_data_dir.join("imdb.db");
            let imdb_conn = providers::imdb::db::init_imdb_db(&imdb_db_path).expect("failed to init imdb db");
            app.manage(ImdbDbState(Arc::new(Mutex::new(imdb_conn))));

            // Initialize Phase 4 Library Watcher
            let mut watcher = scanner::watcher::LibraryWatcher::new();
            if let Ok(conn_guard) = db_arc.lock() {
                let auto_watch = db::queries::get_setting(&conn_guard, "auto_watch_enabled")
                    .ok()
                    .flatten()
                    .map(|v| v != "0")
                    .unwrap_or(true);

                if let Ok(roots) = db::queries::get_library_roots(&conn_guard) {
                    drop(conn_guard);
                    let _ = watcher.set_enabled(auto_watch, roots, Some(app.handle().clone()), db_arc.clone());
                }
            }
            app.manage(WatcherState(Arc::new(Mutex::new(watcher))));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Scanner
            commands::scanner::scan_library,
            // Watcher (Phase 4)
            commands::watcher::get_watcher_status,
            commands::watcher::set_watcher_enabled,
            commands::watcher::reload_watched_roots,
            // Media
            commands::media::get_movies,
            commands::media::get_movie_detail,
            commands::media::get_tv_shows,
            commands::media::get_tv_show_detail,
            commands::media::get_episodes,
            // Settings
            commands::settings::add_library_root,
            commands::settings::get_library_roots,
            commands::settings::remove_library_root,
            commands::settings::toggle_library_root_enabled,
            commands::settings::get_app_setting,
            commands::settings::set_app_setting,
            commands::settings::get_system_text_scale,
            commands::settings::set_window_zoom,
            // Phase 2: Metadata Providers & IMDb
            commands::providers::get_provider_configs,
            commands::providers::set_provider_config,
            commands::providers::get_field_priorities,
            commands::providers::set_field_priority,
            commands::providers::search_metadata_movie,
            commands::providers::search_metadata_tv,
            commands::providers::scrape_and_save_movie,
            commands::providers::scrape_and_save_tv,
            commands::providers::preview_episode_remap,
            commands::providers::check_imdb_updates,
            commands::providers::get_imdb_sync_status,
            commands::providers::index_imdb_from_local,
            commands::providers::is_imdb_installed,
            commands::providers::is_imdb_stale,
            commands::providers::is_imdb_syncing,
            commands::providers::download_and_index_imdb,
            commands::providers::auto_refresh_imdb_if_stale,
            // Phase 3: Renamer & Undo
            commands::renamer::plan_rename_movie,
            commands::renamer::plan_rename_tv_show,
            commands::renamer::plan_batch_rename,
            commands::renamer::plan_resolve_duplicates,
            commands::renamer::execute_rename_plan,
            commands::renamer::undo_last_rename,
            commands::renamer::can_undo,
            // Phase 5: Media Info & Duplicates
            commands::media_info::get_media_stream_info,
            commands::media_info::probe_media_file,
            commands::media_info::probe_pending_batch,
            commands::media_info::get_ffprobe_path,
            commands::media_info::set_ffprobe_path,
            commands::duplicates::get_movie_duplicates,
            commands::duplicates::get_episode_duplicates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
