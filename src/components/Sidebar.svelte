<script lang="ts">
  import type { PlayerSummary } from "../types";
  import BackgroundPicker from "./BackgroundPicker.svelte";
  import CloudUploadSettings from "./CloudUploadSettings.svelte";
  import ExportPanel from "./ExportPanel.svelte";
  import FolderPicker from "./FolderPicker.svelte";
  import PlayerList from "./PlayerList.svelte";

  export let dataDir = "";
  export let savedataDir = "";
  export let backgroundImage = "";
  export let uploadServerUrl = "";
  export let uploadQq = "";
  export let players: PlayerSummary[] = [];
  export let selectedRefid = "";
  export let hasB50 = false;
  export let message = "";
  export let isBusy = false;
  export let isExporting = false;
  export let isUploading = false;
  export let chooseDataDir: () => void | Promise<void>;
  export let chooseSavedataDir: () => void | Promise<void>;
  export let chooseBackgroundImage: () => void | Promise<void>;
  export let clearBackgroundImage: () => void | Promise<void>;
  export let updateUploadServerUrl: (value: string) => void | Promise<void>;
  export let updateUploadQq: (value: string) => void | Promise<void>;
  export let scanInputs: () => void | Promise<void>;
  export let exportPng: () => void | Promise<void>;
  export let uploadB50ToCloud: () => void | Promise<void>;
  export let selectPlayer: (refid: string) => void | Promise<void>;
</script>

<aside class="sidebar">
  <FolderPicker
    {dataDir}
    {savedataDir}
    {isBusy}
    {chooseDataDir}
    {chooseSavedataDir}
    {scanInputs}
  />

  <PlayerList {players} {selectedRefid} {isBusy} {selectPlayer} />

  <ExportPanel {hasB50} {isExporting} {isUploading} {message} {exportPng} {uploadB50ToCloud} />

  <BackgroundPicker {backgroundImage} {chooseBackgroundImage} {clearBackgroundImage} />

  <CloudUploadSettings
    {uploadServerUrl}
    {uploadQq}
    {isBusy}
    {updateUploadServerUrl}
    {updateUploadQq}
  />
</aside>
