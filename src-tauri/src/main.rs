// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod csv;
mod db;
mod error;

use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::Builder;

fn main() {
    let specta_builder = Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
        commands::transactions::import_transactions,
        commands::transactions::list_transactions,
        commands::filters::list_filters,
        commands::filters::create_filter,
        commands::filters::update_filter,
        commands::filters::delete_filter,
        commands::reports::generate_report,
    ]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .invoke_handler(specta_builder.invoke_handler())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let db_path = app_data_dir.join("db.sqlite");
            let pool = tauri::async_runtime::block_on(db::init_db(&db_path))?;
            app.manage(pool);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
