use crate::models::AppSettings;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn default_output_path(file_name: String) -> Result<String, String> {
    let dir = app_base_dir()?;
    Ok(dir.join(file_name).to_string_lossy().to_string())
}

pub(crate) fn load_settings() -> Result<AppSettings, String> {
    let base_dir = app_base_dir()?;
    let discovered = discover_default_paths(&base_dir);
    let path = settings_path()?;
    if !path.is_file() {
        return Ok(discovered);
    }
    let content = fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read settings file {}: {err}", path.display()))?;
    let mut settings: AppSettings = serde_json::from_str(&content)
        .map_err(|err| format!("Failed to parse settings file {}: {err}", path.display()))?;

    if settings.data_dir.is_empty() {
        settings.data_dir = discovered.data_dir;
    }
    if settings.savedata_dir.is_empty() {
        settings.savedata_dir = discovered.savedata_dir;
    }

    Ok(settings)
}

pub(crate) fn save_settings(
    data_dir: String,
    savedata_dir: String,
    background_image: String,
    upload_server_url: String,
    upload_qq: String,
    score_source: String,
    cloud_server_url: String,
    cloud_card_id: String,
    cloud_password: String,
    cloud_pcbid: String,
) -> Result<(), String> {
    let path = settings_path()?;
    let settings = AppSettings {
        data_dir,
        savedata_dir,
        background_image,
        upload_server_url,
        upload_qq,
        score_source,
        cloud_server_url,
        cloud_card_id,
        cloud_password,
        cloud_pcbid,
    };
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|err| format!("Failed to serialize settings: {err}"))?;
    fs::write(&path, content)
        .map_err(|err| format!("Failed to write settings file {}: {err}", path.display()))
}

fn settings_path() -> Result<PathBuf, String> {
    Ok(app_base_dir()?.join("sdvx-b50-tool.settings.json"))
}

fn app_base_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|err| format!("Failed to resolve current executable path: {err}"))?;
    app_base_dir_from_exe(&exe)
}

fn app_base_dir_from_exe(exe: &Path) -> Result<PathBuf, String> {
    if cfg!(target_os = "macos") {
        for ancestor in exe.ancestors() {
            if ancestor.extension().and_then(|value| value.to_str()) == Some("app") {
                return ancestor
                    .parent()
                    .map(|path| path.to_path_buf())
                    .ok_or_else(|| "Failed to resolve app bundle directory.".to_string());
            }
        }
    }

    exe.parent()
        .map(|path| path.to_path_buf())
        .ok_or_else(|| "Failed to resolve executable directory.".to_string())
}

fn discover_default_paths(base_dir: &Path) -> AppSettings {
    let data_dir = first_existing_dir(&[
        base_dir.join("data"),
        base_dir.join("contents").join("data"),
    ]);
    let savedata_dir = first_existing_dir(&[
        base_dir.join("savedata"),
        base_dir.join("asphyxia").join("savedata"),
    ]);

    AppSettings {
        data_dir,
        savedata_dir,
        background_image: String::new(),
        upload_server_url: String::new(),
        upload_qq: String::new(),
        score_source: String::new(),
        cloud_server_url: String::new(),
        cloud_card_id: String::new(),
        cloud_password: String::new(),
        cloud_pcbid: String::new(),
    }
}

fn first_existing_dir(candidates: &[PathBuf]) -> String {
    candidates
        .iter()
        .find(|path| path.is_dir())
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn macos_app_base_dir_points_next_to_bundle() {
        let exe = Path::new("/Users/test/Desktop/SDVX B50 Tool.app/Contents/MacOS/sdvx-b50-tool");

        if cfg!(target_os = "macos") {
            assert_eq!(
                app_base_dir_from_exe(exe).unwrap(),
                PathBuf::from("/Users/test/Desktop")
            );
        }
    }

    #[test]
    fn non_bundle_base_dir_points_to_executable_parent() {
        let exe = Path::new("/Users/test/project/target/release/sdvx-b50-tool");

        assert_eq!(
            app_base_dir_from_exe(exe).unwrap(),
            PathBuf::from("/Users/test/project/target/release")
        );
    }

    #[test]
    fn discovers_direct_data_and_savedata_first() {
        let base = temp_base_dir("direct");
        fs::create_dir_all(base.join("data")).unwrap();
        fs::create_dir_all(base.join("contents").join("data")).unwrap();
        fs::create_dir_all(base.join("savedata")).unwrap();
        fs::create_dir_all(base.join("asphyxia").join("savedata")).unwrap();

        let settings = discover_default_paths(&base);

        assert_eq!(settings.data_dir, base.join("data").to_string_lossy());
        assert_eq!(
            settings.savedata_dir,
            base.join("savedata").to_string_lossy()
        );

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn discovers_nested_fallback_paths() {
        let base = temp_base_dir("nested");
        fs::create_dir_all(base.join("contents").join("data")).unwrap();
        fs::create_dir_all(base.join("asphyxia").join("savedata")).unwrap();

        let settings = discover_default_paths(&base);

        assert_eq!(
            settings.data_dir,
            base.join("contents").join("data").to_string_lossy()
        );
        assert_eq!(
            settings.savedata_dir,
            base.join("asphyxia").join("savedata").to_string_lossy()
        );

        let _ = fs::remove_dir_all(base);
    }

    fn temp_base_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sdvx-b50-settings-{label}-{nanos}"))
    }
}
