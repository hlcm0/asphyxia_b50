use crate::models::{MusicEntry, MusicRecord, Profile, SDVX_DB_FILE, SDVX_VERSION};
use crate::volforce::{better_clear, calculate_volforce};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn validate_savedata_dir(savedata_dir: &str) -> Result<PathBuf, String> {
    let path = Path::new(savedata_dir);
    if !path.is_dir() {
        return Err("The selected savedata path is not a directory.".to_string());
    }
    let db_path = path.join(SDVX_DB_FILE);
    if !db_path.is_file() {
        return Err(format!(
            "Missing {SDVX_DB_FILE} under the selected savedata path."
        ));
    }
    Ok(db_path)
}

pub(crate) fn read_nedb_lines(db_path: &Path) -> Result<Vec<Value>, String> {
    let content = fs::read_to_string(db_path)
        .map_err(|err| format!("Failed to read {}: {err}", db_path.display()))?;
    let mut docs = Vec::new();

    for (index, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value = serde_json::from_str::<Value>(line).map_err(|err| {
            format!(
                "Failed to parse {} line {} as JSON: {err}",
                db_path.display(),
                index + 1
            )
        })?;
        docs.push(value);
    }

    Ok(docs)
}

pub(crate) fn extract_profiles(docs: &[Value]) -> Vec<Profile> {
    let mut seen = HashMap::new();
    let mut profiles = Vec::new();

    for profile in docs
        .iter()
        .filter(|doc| string_field(doc, "collection") == Some("profile"))
        .filter(|doc| number_field(doc, "version") == Some(SDVX_VERSION))
        .filter_map(|doc| {
            Some(Profile {
                refid: string_field(doc, "__refid")?.to_string(),
                name: string_field(doc, "name").unwrap_or("UNKNOWN").to_string(),
                sdvx_id: number_field(doc, "id").unwrap_or(0),
            })
        })
    {
        if seen.insert(profile.refid.clone(), ()).is_none() {
            profiles.push(profile);
        }
    }

    profiles
}

pub(crate) fn count_version7_scores(docs: &[Value]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for doc in docs {
        if string_field(doc, "collection") != Some("music") {
            continue;
        }
        if number_field(doc, "version") != Some(SDVX_VERSION) {
            continue;
        }
        if let Some(refid) = string_field(doc, "__refid") {
            *counts.entry(refid.to_string()).or_insert(0) += 1;
        }
    }
    counts
}

pub(crate) fn extract_music_records(docs: &[Value], refid: &str) -> Vec<MusicRecord> {
    docs.iter()
        .filter(|doc| string_field(doc, "collection") == Some("music"))
        .filter(|doc| number_field(doc, "version") == Some(SDVX_VERSION))
        .filter(|doc| string_field(doc, "__refid") == Some(refid))
        .filter_map(|doc| {
            Some(MusicRecord {
                mid: number_field(doc, "mid")? as u32,
                chart_type: number_field(doc, "type")? as u8,
                score: number_field(doc, "score").unwrap_or(0) as u32,
                clear: number_field(doc, "clear").unwrap_or(0) as u8,
                grade: number_field(doc, "grade").unwrap_or(0) as u8,
                volforce: number_field(doc, "volforce").unwrap_or(0) as u32,
            })
        })
        .collect()
}

pub(crate) fn aggregate_music_records(
    records: Vec<MusicRecord>,
    music_db: &HashMap<u32, MusicEntry>,
) -> Vec<MusicRecord> {
    let mut grouped: HashMap<(u32, u8), MusicRecord> = HashMap::new();

    for record in records {
        grouped
            .entry((record.mid, record.chart_type))
            .and_modify(|current| {
                current.score = current.score.max(record.score);
                current.clear = better_clear(current.clear, record.clear);
                current.grade = current.grade.max(record.grade);
            })
            .or_insert(record);
    }

    grouped
        .into_values()
        .map(|mut record| {
            record.volforce = calculate_volforce(&record, music_db);
            record
        })
        .collect()
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(|value| value.as_str())
}

fn number_field(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(|value| value.as_u64())
}
