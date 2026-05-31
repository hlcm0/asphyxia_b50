use crate::models::MusicEntry;
use encoding_rs::SHIFT_JIS;
use roxmltree::{Document, Node};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub(crate) fn validate_data_dir(data_dir: &str) -> Result<(), String> {
    let path = Path::new(data_dir);
    if !path.is_dir() {
        return Err("The selected game data path is not a directory.".to_string());
    }
    if !path.join("others").join("music_db.xml").is_file() {
        return Err("Missing others/music_db.xml under the selected game data path.".to_string());
    }
    if !path.join("music").is_dir() {
        return Err("Missing music directory under the selected game data path.".to_string());
    }
    Ok(())
}

pub(crate) fn parse_music_db(path: &Path) -> Result<HashMap<u32, MusicEntry>, String> {
    let bytes = fs::read(path).map_err(|err| format!("Failed to read music_db.xml: {err}"))?;
    let (decoded, _, _) = SHIFT_JIS.decode(&bytes);
    let xml =
        translate_special_chars(&decoded).replace("encoding=\"shift-jis\"", "encoding=\"utf-8\"");
    let document =
        Document::parse(&xml).map_err(|err| format!("Failed to parse music_db.xml: {err}"))?;

    let mut entries = HashMap::new();
    for music in document
        .descendants()
        .filter(|node| node.has_tag_name("music"))
    {
        let Some(id) = music.attribute("id").and_then(|id| id.parse::<u32>().ok()) else {
            continue;
        };
        let Some(info) = child(music, "info") else {
            continue;
        };
        let title = child_text(info, "title_name").unwrap_or_else(|| format!("Unknown Song {id}"));
        let inf_ver = child_text(info, "inf_ver").and_then(|value| value.parse::<u8>().ok());
        let levels = parse_levels(music);
        entries.insert(
            id,
            MusicEntry {
                title,
                inf_ver,
                levels,
            },
        );
    }

    Ok(entries)
}

fn translate_special_chars(input: &str) -> String {
    let replacements = [
        ("龕", "€"),
        ("釁", "🍄"),
        ("驩", "Ø"),
        ("曦", "à"),
        ("齷", "é"),
        ("骭", "ü"),
        ("齶", "♡"),
        ("彜", "ū"),
        ("罇", "ê"),
        ("雋", "Ǜ"),
        ("鬻", "♃"),
        ("鬥", "Ã"),
        ("鬆", "Ý"),
        ("曩", "è"),
        ("驫", "ā"),
        ("齲", "♥"),
        ("騫", "á"),
        ("趁", "Ǣ"),
        ("鬮", "¡"),
        ("盥", "⚙︎"),
        ("隍", "︎Ü"),
        ("頽", "ä"),
        ("餮", "Ƶ"),
        ("黻", "*"),
        ("蔕", "ũ"),
        ("闃", "Ā"),
        ("饌", "²"),
        ("煢", "ø"),
        ("鑷", "ゔ"),
        ("=墸Σ", "=͟͟͞ Σ"),
        ("鹹", "Ĥ"),
        ("瀑i", "Ài"),
        ("疉", "Ö"),
        ("鑒", "₩"),
        ("Ryu??", "Ryu☆"),
    ];

    let mut output = input.to_string();
    for (old, new) in replacements {
        output = output.replace(old, new);
    }
    output
}

fn parse_levels(music: Node<'_, '_>) -> [Option<u32>; 6] {
    let mut levels = [None; 6];
    let Some(difficulty) = child(music, "difficulty") else {
        return levels;
    };

    let tags = [
        "novice", "advanced", "exhaust", "infinite", "maximum", "ultimate",
    ];
    for (index, tag) in tags.iter().enumerate() {
        levels[index] = child(difficulty, tag)
            .and_then(|node| child_text(node, "difnum"))
            .and_then(|value| value.parse::<u32>().ok());
    }

    levels
}

fn child<'a, 'input>(node: Node<'a, 'input>, tag: &str) -> Option<Node<'a, 'input>> {
    node.children().find(|child| child.has_tag_name(tag))
}

fn child_text(node: Node<'_, '_>, tag: &str) -> Option<String> {
    child(node, tag)
        .and_then(|child| child.text())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}
