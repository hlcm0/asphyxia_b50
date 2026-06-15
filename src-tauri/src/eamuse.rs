use crate::models::{MusicRecord, PlayerSummary};
use bytes::Bytes;
use md5::{Digest, Md5};
use rand::{distributions::Alphanumeric, Rng};
use roxmltree::Document;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MODEL: &str = "KFC:J:G:A:2026020300";
const INTERNAL_KEY: [u8; 26] = [
    0x69, 0xD7, 0x46, 0x27, 0xD9, 0x85, 0xEE, 0x21, 0x87, 0x16, 0x15, 0x70, 0xD0, 0x8D, 0x93,
    0xB1, 0x24, 0x55, 0x03, 0x5B, 0x6D, 0xF0, 0xD8, 0x20, 0x5D, 0xF5,
];
const GAME_SECURITY_KEY: &str = "c4120d310ac9830010c110c108132139";
pub(crate) const DEFAULT_PCBID: &str = "00010203040506070809";

pub(crate) struct CloudB50Input {
    pub(crate) server_url: String,
    pub(crate) card_id: String,
    pub(crate) password: String,
    pub(crate) pcbid: String,
    pub(crate) progress: Option<CloudProgressReporter>,
}

pub(crate) struct CloudScoreResult {
    pub(crate) player: PlayerSummary,
    pub(crate) records: Vec<MusicRecord>,
}

#[derive(Clone)]
pub(crate) struct CloudProgressReporter {
    emit: Arc<dyn Fn(&str) + Send + Sync>,
}

impl CloudProgressReporter {
    pub(crate) fn new(emit: impl Fn(&str) + Send + Sync + 'static) -> Self {
        Self {
            emit: Arc::new(emit),
        }
    }

    fn stage(&self, stage: &str) {
        (self.emit)(stage);
    }
}

pub(crate) async fn fetch_cloud_scores(input: CloudB50Input) -> Result<CloudScoreResult, String> {
    let progress = input.progress.clone();
    if let Some(progress) = &progress {
        progress.stage("prepare");
    }
    let mut client = EamuseClient::new(input)?;
    client.discover_services().await;
    let refid = client.get_refid().await?;
    if let Some(progress) = &progress {
        progress.stage("load_profile");
    }
    let profile_xml = client.get_profile(&refid).await.unwrap_or_default();
    let name = parse_profile_name(&profile_xml).unwrap_or_else(|| "UNKNOWN".to_string());
    if let Some(progress) = &progress {
        progress.stage("load_scores");
    }
    let records = client.get_scores(&refid).await?;
    let score_count = records.len();

    if records.is_empty() {
        return Err("Cloud server returned no SDVX 7 score data.".to_string());
    }

    Ok(CloudScoreResult {
        player: PlayerSummary {
            refid,
            name,
            sdvx_id: 0,
            score_count,
        },
        records,
    })
}

struct EamuseClient {
    http: reqwest::Client,
    base_url: String,
    service_urls: HashMap<String, String>,
    card_id: String,
    password: String,
    pcbid: String,
    tag: String,
    progress: Option<CloudProgressReporter>,
}

impl EamuseClient {
    fn new(input: CloudB50Input) -> Result<Self, String> {
        let base_url = normalize_server_url(&input.server_url)?;
        let card_id = input.card_id.trim().to_string();
        if card_id.is_empty() {
            return Err("Card ID is required.".to_string());
        }

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|err| format!("Failed to create cloud HTTP client: {err}"))?;

        Ok(Self {
            http,
            base_url,
            service_urls: HashMap::new(),
            card_id,
            password: input.password,
            pcbid: normalized_pcbid(&input.pcbid),
            tag: generate_tag(),
            progress: input.progress,
        })
    }

    async fn discover_services(&mut self) {
        self.report("discover_services");
        let xml = services_get_xml(&self.pcbid, &self.tag);
        if let Ok(response) = self.send_raw(&self.base_url, "services.get", &xml).await {
            if let Ok(services) = parse_services(&response) {
                if !services.is_empty() {
                    self.service_urls = services;
                }
            }
        }
    }

    async fn get_refid(&self) -> Result<String, String> {
        self.report("query_card");
        let response = self
            .call(
                "cardmng",
                "cardmng.inquire",
                &cardmng_inquire_xml(&self.pcbid, &self.tag, &self.card_id),
            )
            .await?;
        let card = parse_cardmng(&response)?;
        let refid = card
            .refid
            .clone()
            .ok_or_else(|| "Card was not found on the cloud server.".to_string())?;

        if should_authenticate(&card) || !self.password.trim().is_empty() {
            if self.password.trim().is_empty() {
                return Err("Cloud server requires a password for this card.".to_string());
            }

            self.report("auth_card");
            let auth_response = self
                .call(
                    "cardmng",
                    "cardmng.authpass",
                    &cardmng_authpass_xml(&self.pcbid, &self.tag, &refid, &self.password),
                )
                .await?;
            let auth = parse_cardmng(&auth_response)?;
            if auth.status.as_deref().unwrap_or("0") != "0" {
                return Err("Cloud server rejected the card password.".to_string());
            }
        }

        Ok(refid)
    }

    async fn get_profile(&self, refid: &str) -> Result<String, String> {
        self.call(
            "sdvx",
            "game.sv7_load",
            &profile_load_xml(&self.pcbid, &self.tag, refid),
        )
        .await
    }

    async fn get_scores(&self, refid: &str) -> Result<Vec<MusicRecord>, String> {
        let response = self
            .call(
                "sdvx",
                "game.sv7_load_m",
                &score_load_xml(&self.pcbid, &self.tag, refid),
            )
            .await?;
        parse_score_records(&response)
    }

    async fn call(&self, service: &str, method_name: &str, xml: &str) -> Result<String, String> {
        let url = self
            .service_urls
            .get(service)
            .or_else(|| {
                if service == "sdvx" {
                    self.service_urls.get("local")
                } else {
                    None
                }
            })
            .unwrap_or(&self.base_url);
        self.send_raw(url, method_name, xml).await
    }

    async fn send_raw(&self, url: &str, method_name: &str, xml: &str) -> Result<String, String> {
        let info = generate_eamuse_info();
        let body = xml_to_kbin(xml)?;
        let encrypted = rc4_crypt(&derive_key(&info)?, &body);
        let endpoint = method_endpoint(url, method_name);
        let response = self
            .http
            .post(endpoint)
            .header("X-Eamuse-Info", &info)
            .header("User-Agent", "EAMUSE.XRPC/1.0")
            .header("X-Compress", "none")
            .header("Content-Length", body_len_header(&encrypted))
            .body(encrypted)
            .send()
            .await
            .map_err(|err| format!("Cloud request failed: {err}"))?;
        let status = response.status();
        let response_info = response
            .headers()
            .get("X-Eamuse-Info")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let bytes = response
            .bytes()
            .await
            .map_err(|err| format!("Failed to read cloud response: {err}"))?;

        if !status.is_success() {
            let text = String::from_utf8_lossy(&bytes);
            return Err(format!(
                "Cloud server returned HTTP {}{}",
                status.as_u16(),
                response_suffix(&text)
            ));
        }

        let decoded = if let Some(info) = response_info {
            rc4_crypt(&derive_key(&info)?, &bytes)
        } else {
            bytes.to_vec()
        };
        kbin_to_xml(&decoded)
    }

    fn report(&self, stage: &str) {
        if let Some(progress) = &self.progress {
            progress.stage(stage);
        }
    }
}

#[derive(Debug)]
struct CardmngResponse {
    refid: Option<String>,
    status: Option<String>,
    authneeded: bool,
}

fn services_get_xml(pcbid: &str, tag: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><call model="{MODEL}" srcid="{pcbid}" tag="{tag}"><services method="get"/></call>"#
    )
}

fn cardmng_inquire_xml(pcbid: &str, tag: &str, card_id: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><call model="{MODEL}" srcid="{pcbid}" tag="{tag}"><cardmng cardid="{}" cardtype="1" method="inquire" update="0"/></call>"#,
        escape_xml_attr(card_id)
    )
}

fn cardmng_authpass_xml(pcbid: &str, tag: &str, refid: &str, password: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><call model="{MODEL}" srcid="{pcbid}" tag="{tag}"><cardmng method="authpass" pass="{}" refid="{}"/></call>"#,
        escape_xml_attr(password),
        escape_xml_attr(refid)
    )
}

fn score_load_xml(pcbid: &str, tag: &str, refid: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><call model="{MODEL}" srcid="{pcbid}" tag="{tag}"><game k="{GAME_SECURITY_KEY}" method="sv7_load_m" ver="0"><refid __type="str">{}</refid></game></call>"#,
        escape_xml_text(refid)
    )
}

fn profile_load_xml(pcbid: &str, tag: &str, refid: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><call model="{MODEL}" srcid="{pcbid}" tag="{tag}"><game k="{GAME_SECURITY_KEY}" method="sv7_load" ver="0"><refid __type="str">{}</refid></game></call>"#,
        escape_xml_text(refid)
    )
}

fn parse_services(xml: &str) -> Result<HashMap<String, String>, String> {
    let mut services = HashMap::new();
    for item in xml.split("<item").skip(1) {
        let end = item.find('>').unwrap_or(item.len());
        let attrs = &item[..end];
        if let (Some(name), Some(url)) = (extract_attr(attrs, "name"), extract_attr(attrs, "url")) {
            services.insert(name, url);
        }
    }
    if !services.contains_key("sdvx") {
        for name in [
            "local", "lobby", "local2", "lobby2", "player2", "pcb2", "shop2", "info2", "lab",
            "local3",
        ] {
            if let Some(url) = services.get(name).cloned() {
                services.insert("sdvx".to_string(), url);
                break;
            }
        }
    }
    Ok(services)
}

fn extract_attr(attrs: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=\"");
    let start = attrs.find(&needle)? + needle.len();
    let rest = &attrs[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn parse_cardmng(xml: &str) -> Result<CardmngResponse, String> {
    let doc = Document::parse(xml).map_err(|err| format!("Failed to parse card response: {err}"))?;
    let node = doc
        .descendants()
        .find(|node| node.has_tag_name("cardmng"))
        .ok_or_else(|| "Cloud server returned no card response.".to_string())?;

    Ok(CardmngResponse {
        refid: node.attribute("refid").map(str::to_string),
        status: node.attribute("status").map(str::to_string),
        authneeded: node.attribute("authneeded") == Some("1"),
    })
}

fn should_authenticate(card: &CardmngResponse) -> bool {
    card.authneeded || card.status.as_deref().is_some_and(|status| status != "0")
}

fn parse_score_records(xml: &str) -> Result<Vec<MusicRecord>, String> {
    let doc = Document::parse(xml).map_err(|err| format!("Failed to parse score XML: {err}"))?;
    let mut records = Vec::new();
    for param in doc.descendants().filter(|node| node.has_tag_name("param")) {
        let Some(text) = param.text() else {
            continue;
        };
        let values = text
            .split_whitespace()
            .map(|value| value.parse::<u32>())
            .collect::<Result<Vec<_>, _>>();
        let Ok(values) = values else {
            continue;
        };
        if values.len() <= 11 {
            continue;
        }
        records.push(MusicRecord {
            mid: values[0],
            chart_type: values[1] as u8,
            score: values[2],
            clear: values[4] as u8,
            grade: values[5] as u8,
            volforce: values[11],
        });
    }
    Ok(records)
}

fn parse_profile_name(xml: &str) -> Option<String> {
    let doc = Document::parse(xml).ok()?;
    let candidates = ["name", "player_name", "playerName", "nickname"];
    for node in doc.descendants() {
        for key in candidates {
            if let Some(value) = node.attribute(key).map(str::trim).filter(|value| !value.is_empty()) {
                return Some(value.to_string());
            }
        }
        if candidates.iter().any(|name| node.has_tag_name(*name)) {
            if let Some(value) = node.text().map(str::trim).filter(|value| !value.is_empty()) {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn xml_to_kbin(xml: &str) -> Result<Vec<u8>, String> {
    let (nodes, _) = kbinxml::from_text_xml(xml.as_bytes())
        .map_err(|err| format!("Failed to encode XML request: {err}"))?;
    kbinxml::to_binary(&nodes).map_err(|err| format!("Failed to encode KBinXML request: {err}"))
}

fn kbin_to_xml(bytes: &[u8]) -> Result<String, String> {
    if bytes.is_empty() {
        return Err("Cloud server returned an empty response body.".to_string());
    }

    if kbinxml::is_binary_xml(bytes) {
        let (nodes, _) = kbinxml::from_binary(Bytes::copy_from_slice(bytes))
            .map_err(|err| format!("Failed to decode KBinXML response: {err}"))?;
        let xml = kbinxml::to_text_xml(&nodes)
            .map_err(|err| format!("Failed to render cloud XML response: {err}"))?;
        String::from_utf8(xml).map_err(|err| format!("Cloud XML response is not UTF-8: {err}"))
    } else {
        Ok(String::from_utf8_lossy(bytes).to_string())
    }
}

fn derive_key(info: &str) -> Result<[u8; 16], String> {
    let hex = info
        .strip_prefix("1-")
        .unwrap_or(info)
        .replace('-', "");
    let material = decode_hex(&hex)?;
    let mut hasher = Md5::new();
    hasher.update(material);
    hasher.update(INTERNAL_KEY);
    Ok(hasher.finalize().into())
}

fn rc4_crypt(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut state = [0u8; 256];
    for (index, value) in state.iter_mut().enumerate() {
        *value = index as u8;
    }

    let mut j = 0usize;
    for i in 0..256 {
        j = (j + state[i] as usize + key[i % key.len()] as usize) & 0xff;
        state.swap(i, j);
    }

    let mut i = 0usize;
    j = 0;
    let mut output = Vec::with_capacity(data.len());
    for byte in data {
        i = (i + 1) & 0xff;
        j = (j + state[i] as usize) & 0xff;
        state.swap(i, j);
        let key_byte = state[(state[i] as usize + state[j] as usize) & 0xff];
        output.push(byte ^ key_byte);
    }
    output
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 {
        return Err("Invalid Eamuse key material.".to_string());
    }
    value
        .as_bytes()
        .chunks(2)
        .map(|chunk| {
            let text = std::str::from_utf8(chunk).map_err(|_| "Invalid hex data.".to_string())?;
            u8::from_str_radix(text, 16).map_err(|_| "Invalid hex data.".to_string())
        })
        .collect()
}

fn normalize_server_url(value: &str) -> Result<String, String> {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("Cloud server address is required.".to_string());
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Ok(trimmed.to_string())
    } else {
        Ok(format!("http://{trimmed}"))
    }
}

fn normalized_pcbid(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        DEFAULT_PCBID.to_string()
    } else {
        trimmed.to_string()
    }
}

fn method_endpoint(url: &str, method_name: &str) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}model={MODEL}&f={method_name}")
}

fn generate_eamuse_info() -> String {
    let timestamp = unix_seconds();
    let random = rand::thread_rng().gen_range(0..=0xffff);
    format!("1-{timestamp:08x}-{random:04x}")
}

fn generate_tag() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn body_len_header(body: &[u8]) -> String {
    body.len().to_string()
}

fn response_suffix(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!(": {trimmed}")
    }
}

fn escape_xml_attr(value: &str) -> String {
    escape_xml_text(value).replace('"', "&quot;")
}

fn escape_xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rc4_matches_known_vector() {
        assert_eq!(rc4_crypt(b"Key", b"Plaintext"), b"\xbb\xf3\x16\xe8\xd9@\xaf\n\xd3");
    }

    #[test]
    fn parses_sv7_score_params() {
        let xml = r#"<response><game><music><info><param __type="u32" __count="26">1 2 9900000 0 5 10 0 0 0 0 0 12345 0 0 0 0 0 0 0 0 0 0 0 0 0 0</param></info></music></game></response>"#;
        let records = parse_score_records(xml).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].mid, 1);
        assert_eq!(records[0].chart_type, 2);
        assert_eq!(records[0].score, 9_900_000);
        assert_eq!(records[0].clear, 5);
        assert_eq!(records[0].grade, 10);
        assert_eq!(records[0].volforce, 12_345);
    }

    #[test]
    fn falls_back_to_default_pcbid() {
        assert_eq!(normalized_pcbid(""), DEFAULT_PCBID);
    }

    #[test]
    fn parses_service_urls_with_raw_ampersands() {
        let xml = r#"<response><services><item name="keepalive" url="ping://127.0.0.1/core/keepalive?t1=2&t2=15"/><item name="cardmng" url="http://example.test/service/card"/><item name="local" url="http://example.test/service/sdvx"/></services></response>"#;
        let services = parse_services(xml).unwrap();

        assert_eq!(
            services.get("cardmng").map(String::as_str),
            Some("http://example.test/service/card")
        );
        assert_eq!(
            services.get("sdvx").map(String::as_str),
            Some("http://example.test/service/sdvx")
        );
    }
}
