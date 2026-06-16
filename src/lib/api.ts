import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, B50Result, ScanResult, UploadB50Result } from "../types";

export function loadSettings() {
  return invoke<AppSettings>("load_settings");
}

export function saveSettings(
  dataDir: string,
  savedataDir: string,
  backgroundImage: string,
  uploadServerUrl: string,
  uploadQq: string,
  scoreSource: string,
  cloudServerUrl: string,
  cloudCardId: string,
  cloudPassword: string,
  cloudPcbid: string
) {
  return invoke("save_settings", {
    dataDir,
    savedataDir,
    backgroundImage,
    uploadServerUrl,
    uploadQq,
    scoreSource,
    cloudServerUrl,
    cloudCardId,
    cloudPassword,
    cloudPcbid
  });
}

export function scanInputs(dataDir: string, savedataDir: string) {
  return invoke<ScanResult>("scan_inputs", { dataDir, savedataDir });
}

export function generateB50(dataDir: string, savedataDir: string, refid: string) {
  return invoke<B50Result>("generate_b50", { dataDir, savedataDir, refid });
}

export function generateCloudB50(
  dataDir: string,
  serverUrl: string,
  cardId: string,
  password: string,
  pcbid: string,
  requestId: string
) {
  return invoke<B50Result>("generate_cloud_b50", { dataDir, serverUrl, cardId, password, pcbid, requestId });
}

export function defaultOutputPath(fileName: string) {
  return invoke<string>("default_output_path", { fileName });
}

export function savePng(bytes: Uint8Array, outputPath: string) {
  return invoke("save_png", {
    bytes: Array.from(bytes),
    outputPath
  });
}

export function readImageDataUrl(imagePath: string) {
  return invoke<string>("read_image_data_url", { imagePath });
}

export function uploadB50(
  serverUrl: string,
  qq: string,
  b50: B50Result,
  cloudServerUrl: string,
  cloudCardId: string,
  cloudPassword: string,
  cloudPcbid: string
) {
  return invoke<UploadB50Result>("upload_b50", {
    serverUrl,
    qq,
    b50,
    cloudServerUrl: cloudServerUrl || null,
    cloudCardId: cloudCardId || null,
    cloudPassword: cloudPassword || null,
    cloudPcbid: cloudPcbid || null
  });
}
