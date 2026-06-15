use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub(crate) const SDVX_DB_FILE: &str = "sdvx@asphyxia.db";
pub(crate) const SDVX_VERSION: u64 = 7;
pub(crate) const CLOUD_PROGRESS_EVENT: &str = "cloud-b50-progress";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlayerSummary {
    pub(crate) refid: String,
    pub(crate) name: String,
    pub(crate) sdvx_id: u64,
    pub(crate) score_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScanResult {
    pub(crate) version: u64,
    pub(crate) players: Vec<PlayerSummary>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSettings {
    pub(crate) data_dir: String,
    pub(crate) savedata_dir: String,
    #[serde(default)]
    pub(crate) background_image: String,
    #[serde(default)]
    pub(crate) upload_server_url: String,
    #[serde(default)]
    pub(crate) upload_qq: String,
    #[serde(default)]
    pub(crate) score_source: String,
    #[serde(default)]
    pub(crate) cloud_server_url: String,
    #[serde(default)]
    pub(crate) cloud_card_id: String,
    #[serde(default)]
    pub(crate) cloud_password: String,
    #[serde(default)]
    pub(crate) cloud_pcbid: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct B50Result {
    pub(crate) version: u64,
    pub(crate) player: PlayerSummary,
    pub(crate) total_vf: String,
    pub(crate) generated_at: String,
    pub(crate) cards: Vec<B50Card>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct B50Card {
    pub(crate) rank: usize,
    pub(crate) mid: u32,
    pub(crate) title: String,
    pub(crate) difficulty_label: String,
    pub(crate) level: String,
    pub(crate) score: u32,
    pub(crate) clear_lamp: String,
    pub(crate) single_vf: String,
    pub(crate) jacket_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UploadB50Result {
    pub(crate) ok: bool,
    pub(crate) message: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CloudProgressEvent {
    pub(crate) request_id: String,
    pub(crate) stage: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Profile {
    pub(crate) refid: String,
    pub(crate) name: String,
    pub(crate) sdvx_id: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct MusicRecord {
    pub(crate) mid: u32,
    pub(crate) chart_type: u8,
    pub(crate) score: u32,
    pub(crate) clear: u8,
    pub(crate) grade: u8,
    pub(crate) volforce: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct MusicEntry {
    pub(crate) title: String,
    pub(crate) inf_ver: Option<u8>,
    pub(crate) levels: [Option<u32>; 6],
}

#[derive(Debug, Clone)]
pub(crate) struct JacketEntry {
    pub(crate) number: u8,
    pub(crate) path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ScanArgs {
    pub(crate) data_dir: String,
    pub(crate) savedata_dir: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GenerateArgs {
    pub(crate) data_dir: String,
    pub(crate) savedata_dir: String,
    pub(crate) refid: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GenerateCloudArgs {
    pub(crate) data_dir: String,
    pub(crate) server_url: String,
    pub(crate) card_id: String,
    pub(crate) password: String,
    pub(crate) pcbid: String,
    pub(crate) request_id: String,
}
