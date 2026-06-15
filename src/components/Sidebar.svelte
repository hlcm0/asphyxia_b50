<script lang="ts">
  import type { PlayerSummary } from "../types";
  import BackgroundPicker from "./BackgroundPicker.svelte";
  import CloudScoreSettings from "./CloudScoreSettings.svelte";
  import CloudUploadSettings from "./CloudUploadSettings.svelte";
  import ExportPanel from "./ExportPanel.svelte";
  import FolderPicker from "./FolderPicker.svelte";
  import PlayerList from "./PlayerList.svelte";

  export let dataDir = "";
  export let savedataDir = "";
  export let backgroundImage = "";
  export let uploadServerUrl = "";
  export let uploadQq = "";
  export let scoreSource = "local";
  export let cloudServerUrl = "";
  export let cloudCardId = "";
  export let cloudPassword = "";
  export let cloudPcbid = "";
  export let players: PlayerSummary[] = [];
  export let selectedRefid = "";
  export let hasB50 = false;
  export let message = "";
  export let isBusy = false;
  export let isExporting = false;
  export let isUploading = false;
  export let cloudProgressText = "";
  export let chooseDataDir: () => void | Promise<void>;
  export let chooseSavedataDir: () => void | Promise<void>;
  export let chooseBackgroundImage: () => void | Promise<void>;
  export let clearBackgroundImage: () => void | Promise<void>;
  export let updateUploadServerUrl: (value: string) => void | Promise<void>;
  export let updateUploadQq: (value: string) => void | Promise<void>;
  export let updateScoreSource: (value: string) => void | Promise<void>;
  export let updateCloudServerUrl: (value: string) => void | Promise<void>;
  export let updateCloudCardId: (value: string) => void | Promise<void>;
  export let updateCloudPassword: (value: string) => void | Promise<void>;
  export let updateCloudPcbid: (value: string) => void | Promise<void>;
  export let scanInputs: () => void | Promise<void>;
  export let generateCloudB50: () => void | Promise<void>;
  export let exportPng: () => void | Promise<void>;
  export let uploadB50ToCloud: () => void | Promise<void>;
  export let selectPlayer: (refid: string) => void | Promise<void>;
</script>

<aside class="sidebar">
  <section class="panel">
    <div class="panel-title">Score Source</div>
    <div class="segmented">
      <button
        class:active={scoreSource === "local"}
        type="button"
        disabled={isBusy}
        on:click={() => updateScoreSource("local")}
      >
        Local Savedata
      </button>
      <button
        class:active={scoreSource === "cloud"}
        type="button"
        disabled={isBusy}
        on:click={() => updateScoreSource("cloud")}
      >
        Cloud Server
      </button>
    </div>
  </section>

  <FolderPicker
    {dataDir}
    {savedataDir}
    {scoreSource}
    {isBusy}
    {chooseDataDir}
    {chooseSavedataDir}
    {scanInputs}
  />

  {#if scoreSource === "local"}
    <PlayerList {players} {selectedRefid} {isBusy} {selectPlayer} />
  {:else}
    <CloudScoreSettings
      {dataDir}
      {cloudServerUrl}
      {cloudCardId}
      {cloudPassword}
      {cloudPcbid}
      {isBusy}
      {cloudProgressText}
      {updateCloudServerUrl}
      {updateCloudCardId}
      {updateCloudPassword}
      {updateCloudPcbid}
      {generateCloudB50}
    />
  {/if}

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
