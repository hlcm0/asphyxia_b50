use crate::models::JacketEntry;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub(crate) fn scan_jackets(music_dir: &Path) -> Result<HashMap<u32, Vec<JacketEntry>>, String> {
    let mut jackets: HashMap<u32, Vec<JacketEntry>> = HashMap::new();
    let dirs =
        fs::read_dir(music_dir).map_err(|err| format!("Failed to read music directory: {err}"))?;

    for dir in dirs {
        let dir = dir.map_err(|err| format!("Failed to read a music directory entry: {err}"))?;
        let path = dir.path();
        if !path.is_dir() {
            continue;
        }

        let Ok(files) = fs::read_dir(&path) else {
            continue;
        };
        for file in files.flatten() {
            let file_path = file.path();
            if !file_path.is_file() {
                continue;
            }
            let Some(name) = file_path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Some(stem) = name.strip_suffix(".png") else {
                continue;
            };
            let parts = stem.split('_').collect::<Vec<_>>();
            if parts.len() != 3 || parts[0] != "jk" {
                continue;
            }
            let (Ok(mid), Ok(number)) = (parts[1].parse::<u32>(), parts[2].parse::<u8>()) else {
                continue;
            };
            jackets.entry(mid).or_default().push(JacketEntry {
                number,
                path: file_path,
            });
        }
    }

    for entries in jackets.values_mut() {
        entries.sort_by_key(|entry| entry.number);
    }

    Ok(jackets)
}

pub(crate) fn select_jacket_data_url(
    jackets: &HashMap<u32, Vec<JacketEntry>>,
    mid: u32,
    target: u8,
) -> Option<String> {
    let entries = jackets.get(&mid)?;
    let selected = entries
        .iter()
        .filter(|entry| entry.number <= target)
        .max_by_key(|entry| entry.number)
        .or_else(|| entries.iter().min_by_key(|entry| entry.number))?;
    let bytes = fs::read(&selected.path).ok()?;
    Some(format!("data:image/png;base64,{}", BASE64.encode(bytes)))
}

pub(crate) fn placeholder_data_url() -> String {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128"><rect width="128" height="128" fill="#e5e7eb"/><rect x="12" y="12" width="104" height="104" rx="10" fill="#f8fafc" stroke="#cbd5e1" stroke-width="4"/><text x="64" y="59" text-anchor="middle" font-family="Arial,sans-serif" font-size="14" font-weight="700" fill="#64748b">NO</text><text x="64" y="78" text-anchor="middle" font-family="Arial,sans-serif" font-size="14" font-weight="700" fill="#64748b">JACKET</text></svg>"##;
    format!("data:image/svg+xml;base64,{}", BASE64.encode(svg))
}
