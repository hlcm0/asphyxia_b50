use crate::models::{MusicEntry, MusicRecord};
use std::collections::HashMap;

pub(crate) fn calculate_volforce(record: &MusicRecord, music_db: &HashMap<u32, MusicEntry>) -> u32 {
    let Some(music) = music_db.get(&record.mid) else {
        return record.volforce;
    };
    let Some(raw_level) = music
        .levels
        .get(record.chart_type as usize)
        .and_then(|level| *level)
    else {
        return record.volforce;
    };

    let level = if raw_level > 20 {
        raw_level as f32 / 10.0
    } else {
        raw_level as f32
    };
    let score = record.score as f32 / 10_000_000.0;
    let vf = level * score * grade_factor(record.grade) * clear_factor(record.clear) * 20.0;
    vf.floor() as u32
}

pub(crate) fn better_clear(left: u8, right: u8) -> u8 {
    if clear_factor(right) > clear_factor(left) {
        right
    } else {
        left
    }
}

pub(crate) fn difficulty_label(chart_type: u8, inf_ver: Option<u8>) -> String {
    match chart_type {
        0 => "NOV".to_string(),
        1 => "ADV".to_string(),
        2 => "EXH".to_string(),
        3 => match inf_ver.unwrap_or(2) {
            2 => "INF".to_string(),
            3 => "GRV".to_string(),
            4 => "HVN".to_string(),
            5 => "VVD".to_string(),
            6 => "XCD".to_string(),
            _ => "INF".to_string(),
        },
        4 => "MXM".to_string(),
        5 => "ULT".to_string(),
        _ => "UNK".to_string(),
    }
}

pub(crate) fn clear_lamp(clear: u8) -> String {
    match clear {
        1 => "C",
        2 => "EC",
        3 => "HC",
        4 => "MC",
        5 => "UC",
        6 => "PUC",
        _ => "ND",
    }
    .to_string()
}

pub(crate) fn format_level(level: u32) -> String {
    if level <= 20 {
        return level.to_string();
    }

    if level % 10 == 0 {
        (level / 10).to_string()
    } else {
        format!("{:.1}", level as f32 / 10.0)
    }
}

pub(crate) fn format_single_vf(volforce: u32) -> String {
    format!("{:.2}", volforce as f32 / 20.0)
}

pub(crate) fn generated_at() -> String {
    let now = time::OffsetDateTime::now_utc();
    now.format(&time::macros::format_description!(
        "[year]-[month]-[day] [hour]:[minute]"
    ))
    .unwrap_or_else(|_| "unknown time".to_string())
}

fn grade_factor(grade: u8) -> f32 {
    match grade {
        1 => 0.80,
        2 => 0.82,
        3 => 0.85,
        4 => 0.88,
        5 => 0.91,
        6 => 0.94,
        7 => 0.97,
        8 => 1.00,
        9 => 1.02,
        10 => 1.05,
        _ => 0.0,
    }
}

fn clear_factor(clear: u8) -> f32 {
    match clear {
        1 => 0.5,
        2 => 1.0,
        3 => 1.02,
        4 => 1.04,
        5 => 1.06,
        6 => 1.10,
        _ => 0.0,
    }
}
