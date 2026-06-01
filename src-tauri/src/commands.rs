use crate::jacket::{placeholder_data_url, scan_jackets, select_jacket_data_url};
use crate::models::{
    AppSettings, B50Card, B50Result, GenerateArgs, PlayerSummary, ScanArgs, ScanResult,
    UploadB50Result, SDVX_VERSION,
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
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

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
    upload_server_url: String,
    upload_qq: String,
) -> Result<(), String> {
    settings::save_settings(
        data_dir,
        savedata_dir,
        background_image,
        upload_server_url,
        upload_qq,
    )
}

#[tauri::command]
pub(crate) async fn upload_b50(
    server_url: String,
    qq: String,
    b50: B50Result,
) -> Result<UploadB50Result, String> {
    upload_b50_inner(server_url, qq, b50).await
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

async fn upload_b50_inner(
    server_url: String,
    qq: String,
    b50: B50Result,
) -> Result<UploadB50Result, String> {
    let endpoint = upload_endpoint(&server_url)?;
    validate_upload_qq(&qq)?;

    if b50.cards.is_empty() {
        return Err("B50 data is empty.".to_string());
    }

    let payload = UploadB50Payload {
        schema_version: 1,
        game: "sdvx",
        version: b50.version,
        qq,
        player: b50.player,
        total_vf: b50.total_vf,
        generated_at: b50.generated_at,
        cards: b50.cards.into_iter().map(UploadB50Card::from).collect(),
        client: UploadClientInfo {
            app: "sdvx-b50-tool",
            upload_at: upload_timestamp(),
        },
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|err| format!("Failed to create HTTP client: {err}"))?;
    let response = client
        .post(endpoint)
        .json(&payload)
        .send()
        .await
        .map_err(|err| format!("Upload request failed: {err}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|err| format!("Failed to read upload response: {err}"))?;

    if !status.is_success() {
        return Err(format!(
            "Server returned HTTP {}{}",
            status.as_u16(),
            response_suffix(&body)
        ));
    }

    if body.trim().is_empty() {
        return Ok(UploadB50Result {
            ok: true,
            message: "Cloud upload complete.".to_string(),
        });
    }

    match serde_json::from_str::<UploadB50Result>(&body) {
        Ok(result) if result.ok => Ok(result),
        Ok(result) => Err(result.message),
        Err(_) => Ok(UploadB50Result {
            ok: true,
            message: "Cloud upload complete.".to_string(),
        }),
    }
}

fn upload_endpoint(server_url: &str) -> Result<String, String> {
    let trimmed = server_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("Server address is required.".to_string());
    }

    let normalized = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    };

    Ok(format!("{normalized}/api/sdvx/b50"))
}

fn validate_upload_qq(qq: &str) -> Result<(), String> {
    if (5..=12).contains(&qq.len()) && qq.chars().all(|char| char.is_ascii_digit()) {
        Ok(())
    } else {
        Err("QQ number must be 5 to 12 digits.".to_string())
    }
}

fn upload_timestamp() -> String {
    let now = time::OffsetDateTime::now_utc();
    now.format(&time::macros::format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second]Z"
    ))
    .unwrap_or_else(|_| "unknown time".to_string())
}

fn response_suffix(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!(": {trimmed}")
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UploadB50Payload {
    schema_version: u64,
    game: &'static str,
    version: u64,
    qq: String,
    player: PlayerSummary,
    total_vf: String,
    generated_at: String,
    cards: Vec<UploadB50Card>,
    client: UploadClientInfo,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UploadClientInfo {
    app: &'static str,
    upload_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UploadB50Card {
    rank: usize,
    mid: u32,
    title: String,
    difficulty_label: String,
    level: String,
    score: u32,
    clear_lamp: String,
    single_vf: String,
}

impl From<B50Card> for UploadB50Card {
    fn from(card: B50Card) -> Self {
        Self {
            rank: card.rank,
            mid: card.mid,
            title: card.title,
            difficulty_label: card.difficulty_label,
            level: card.level,
            score: card.score,
            clear_lamp: card.clear_lamp,
            single_vf: card.single_vf,
        }
    }
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
