use crate::jacket::{placeholder_data_url, scan_jackets, select_jacket_data_url};
use crate::models::{
    AppSettings, B50Card, B50Result, GenerateArgs, PlayerSummary, ScanArgs, ScanResult,
    SDVX_VERSION,
};
use crate::music_db::{parse_music_db, validate_data_dir};
use crate::savedata::{
    aggregate_music_records, count_version7_scores, extract_music_records, extract_profiles,
    read_nedb_lines, validate_savedata_dir,
};
use crate::settings;
use crate::volforce::{clear_lamp, difficulty_label, format_level, format_single_vf, generated_at};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::fs;
use std::path::{Path, PathBuf};

#[tauri::command]
pub(crate) fn scan_inputs(data_dir: String, savedata_dir: String) -> Result<ScanResult, String> {
    scan_inputs_inner(ScanArgs {
        data_dir,
        savedata_dir,
    })
}

#[tauri::command]
pub(crate) fn generate_b50(
    data_dir: String,
    savedata_dir: String,
    refid: String,
) -> Result<B50Result, String> {
    generate_b50_inner(GenerateArgs {
        data_dir,
        savedata_dir,
        refid,
    })
}

#[tauri::command]
pub(crate) fn save_png(bytes: Vec<u8>, output_path: String) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("PNG data is empty.".to_string());
    }

    let path = PathBuf::from(output_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create output directory: {err}"))?;
    }

    fs::write(&path, bytes).map_err(|err| format!("Failed to save PNG: {err}"))
}

#[tauri::command]
pub(crate) fn read_image_data_url(image_path: String) -> Result<String, String> {
    let path = PathBuf::from(image_path);
    if !path.is_file() {
        return Err("Background image file does not exist.".to_string());
    }

    let bytes = fs::read(&path)
        .map_err(|err| format!("Failed to read background image {}: {err}", path.display()))?;
    Ok(format!(
        "data:{};base64,{}",
        image_mime_type(&path)?,
        BASE64.encode(bytes)
    ))
}

#[tauri::command]
pub(crate) fn default_output_path(file_name: String) -> Result<String, String> {
    settings::default_output_path(file_name)
}

#[tauri::command]
pub(crate) fn load_settings() -> Result<AppSettings, String> {
    settings::load_settings()
}

#[tauri::command]
pub(crate) fn save_settings(
    data_dir: String,
    savedata_dir: String,
    background_image: String,
) -> Result<(), String> {
    settings::save_settings(data_dir, savedata_dir, background_image)
}

fn scan_inputs_inner(args: ScanArgs) -> Result<ScanResult, String> {
    validate_data_dir(&args.data_dir)?;
    let db_path = validate_savedata_dir(&args.savedata_dir)?;
    let docs = read_nedb_lines(&db_path)?;
    let profiles = extract_profiles(&docs);
    let score_counts = count_version7_scores(&docs);

    let mut players: Vec<PlayerSummary> = profiles
        .into_iter()
        .map(|profile| PlayerSummary {
            score_count: *score_counts.get(&profile.refid).unwrap_or(&0),
            refid: profile.refid,
            name: profile.name,
            sdvx_id: profile.sdvx_id,
        })
        .filter(|player| player.score_count > 0)
        .collect();

    players.sort_by(|a, b| {
        b.score_count
            .cmp(&a.score_count)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.refid.cmp(&b.refid))
    });

    Ok(ScanResult {
        version: SDVX_VERSION,
        players,
    })
}

fn generate_b50_inner(args: GenerateArgs) -> Result<B50Result, String> {
    let data_dir = PathBuf::from(&args.data_dir);
    validate_data_dir(&args.data_dir)?;
    let db_path = validate_savedata_dir(&args.savedata_dir)?;

    let docs = read_nedb_lines(&db_path)?;
    let profiles = extract_profiles(&docs);
    let score_counts = count_version7_scores(&docs);
    let profile = profiles
        .into_iter()
        .find(|profile| profile.refid == args.refid)
        .ok_or_else(|| "Selected player has no SDVX 7 profile.".to_string())?;

    let player = PlayerSummary {
        score_count: *score_counts.get(&profile.refid).unwrap_or(&0),
        refid: profile.refid.clone(),
        name: profile.name.clone(),
        sdvx_id: profile.sdvx_id,
    };

    let music_db = parse_music_db(&data_dir.join("others").join("music_db.xml"))?;
    let jackets = scan_jackets(&data_dir.join("music"))?;
    let placeholder = placeholder_data_url();

    let mut records = aggregate_music_records(extract_music_records(&docs, &args.refid), &music_db);
    records.retain(|record| music_db.contains_key(&record.mid));
    records.sort_by(|a, b| {
        b.volforce
            .cmp(&a.volforce)
            .then_with(|| b.score.cmp(&a.score))
            .then_with(|| a.mid.cmp(&b.mid))
            .then_with(|| a.chart_type.cmp(&b.chart_type))
    });

    let top_records = records.into_iter().take(50).collect::<Vec<_>>();
    let total_vf = top_records
        .iter()
        .map(|record| record.volforce)
        .sum::<u32>() as f32
        / 1000.0;

    let cards = top_records
        .into_iter()
        .enumerate()
        .map(|(index, record)| {
            let music = music_db
                .get(&record.mid)
                .expect("record was filtered by music_db");
            let level = music
                .levels
                .get(record.chart_type as usize)
                .and_then(|level| *level)
                .map(format_level)
                .unwrap_or_else(|| "0".to_string());
            let jacket_path =
                select_jacket_data_url(&jackets, record.mid, record.chart_type.saturating_add(1))
                    .unwrap_or_else(|| placeholder.clone());

            B50Card {
                rank: index + 1,
                mid: record.mid,
                title: music.title.clone(),
                difficulty_label: difficulty_label(record.chart_type, music.inf_ver),
                level,
                score: record.score,
                clear_lamp: clear_lamp(record.clear),
                single_vf: format_single_vf(record.volforce),
                jacket_path,
            }
        })
        .collect::<Vec<_>>();

    Ok(B50Result {
        version: SDVX_VERSION,
        player,
        total_vf: format!("{total_vf:.3}"),
        generated_at: generated_at(),
        cards,
    })
}

fn image_mime_type(path: &Path) -> Result<&'static str, String> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => Ok("image/png"),
        Some("jpg") | Some("jpeg") => Ok("image/jpeg"),
        Some("webp") => Ok("image/webp"),
        Some("bmp") => Ok("image/bmp"),
        Some("gif") => Ok("image/gif"),
        _ => Err("Unsupported background image type.".to_string()),
    }
}
