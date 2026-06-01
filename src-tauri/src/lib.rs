mod commands;
mod jacket;
mod models;
mod music_db;
mod savedata;
mod settings;
mod volforce;

fn create_main_window<R: tauri::Runtime>(app: &mut tauri::App<R>) -> tauri::Result<()> {
    let window_config = &app.config().app.windows[0];

    #[cfg(windows)]
    {
        let mut last_error = None;

        for data_dir in webview_data_dir_candidates() {
            if ensure_writable_dir(&data_dir).is_err() {
                continue;
            }

            let result = tauri::WebviewWindowBuilder::from_config(app.handle(), window_config)?
                .data_directory(data_dir.clone())
                .build();

            match result {
                Ok(_) => return Ok(()),
                Err(error) => {
                    eprintln!(
                        "failed to create WebView with data directory {}: {error}",
                        data_dir.display()
                    );
                    last_error = Some(error);
                }
            }
        }

        if let Some(error) = last_error {
            return Err(error);
        }
    }

    #[cfg(not(windows))]
    {
        tauri::WebviewWindowBuilder::from_config(app.handle(), window_config)?.build()?;
    }

    Ok(())
}

#[cfg(windows)]
fn webview_data_dir_candidates() -> Vec<std::path::PathBuf> {
    let mut candidates = Vec::new();

    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let local_app_data = std::path::PathBuf::from(local_app_data);
        candidates.push(local_app_data.join("sdvx-b50-tool").join("webview-data-v2"));
        candidates.push(local_app_data.join("net.local.sdvx-b50-tool"));
    }

    let temp_dir = std::env::temp_dir();
    candidates.push(temp_dir.join("sdvx-b50-tool-webview"));
    candidates.push(temp_dir.join(format!("sdvx-b50-tool-webview-{}", std::process::id())));

    candidates
}

#[cfg(windows)]
fn ensure_writable_dir(path: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;

    let probe = path.join(".write-test");
    std::fs::write(&probe, b"ok")?;
    let _ = std::fs::remove_file(probe);

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            create_main_window(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan_inputs,
            commands::generate_b50,
            commands::save_png,
            commands::read_image_data_url,
            commands::default_output_path,
            commands::load_settings,
            commands::save_settings,
            commands::upload_b50
        ])
        .run(tauri::generate_context!())
        .expect("error while running SDVX B50 Tool");
}
