mod commands;
mod jacket;
mod models;
mod music_db;
mod savedata;
mod settings;
mod volforce;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_inputs,
            commands::generate_b50,
            commands::save_png,
            commands::read_image_data_url,
            commands::default_output_path,
            commands::load_settings,
            commands::save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running SDVX B50 Tool");
}
