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
  uploadQq: string
) {
  return invoke("save_settings", {
    dataDir,
    savedataDir,
    backgroundImage,
    uploadServerUrl,
    uploadQq
  });
}

export function scanInputs(dataDir: string, savedataDir: string) {
  return invoke<ScanResult>("scan_inputs", { dataDir, savedataDir });
}

export function generateB50(dataDir: string, savedataDir: string, refid: string) {
  return invoke<B50Result>("generate_b50", { dataDir, savedataDir, refid });
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

export function uploadB50(serverUrl: string, qq: string, b50: B50Result) {
  return invoke<UploadB50Result>("upload_b50", { serverUrl, qq, b50 });
}
