<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import B50Preview from "./components/B50Preview.svelte";
  import EmptyState from "./components/EmptyState.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import {
    defaultOutputPath,
    generateB50 as generateB50Data,
    generateCloudB50 as generateCloudB50Data,
    loadSettings,
    readImageDataUrl,
    savePng,
    saveSettings,
    scanInputs as scanInputFolders,
    uploadB50
  } from "./lib/api";
  import { renderBoardPng, sanitizeName } from "./lib/exportPng";
  import type { B50Result, PlayerSummary } from "./types";

  let dataDir = "";
  let savedataDir = "";
  let backgroundImage = "";
  let uploadServerUrl = "";
  let uploadQq = "";
  let scoreSource = "local";
  let cloudServerUrl = "";
  let cloudCardId = "";
  let cloudPassword = "";
  let cloudPcbid = "";
  let players: PlayerSummary[] = [];
  let selectedRefid = "";
  let b50: B50Result | null = null;
  let message = "";
  let isBusy = false;
  let isExporting = false;
  let isUploading = false;
  let backgroundImageUrl = "";
  let exportColumns = 5;
  let backgroundLoadId = 0;
  let generateRequestId = 0;
  let cloudProgressRequestId = "";
  let cloudProgressStage = "";
  let unlistenCloudProgress: UnlistenFn | null = null;

  const cloudProgressLabels: Record<string, string> = {
    auth_card: "Verifying password...",
    build_b50: "Building B50...",
    discover_services: "Discovering services...",
    load_profile: "Loading profile...",
    load_scores: "Loading scores...",
    prepare: "Preparing request...",
    query_card: "Querying card...",
    validate_data: "Checking game data..."
  };

  onMount(() => {
    listen<{ requestId: string; stage: string }>("cloud-b50-progress", (event) => {
      if (event.payload.requestId === cloudProgressRequestId) {
        cloudProgressStage = event.payload.stage;
        message = cloudProgressText(event.payload.stage);
      }
    }).then((unlisten) => {
      unlistenCloudProgress = unlisten;
    });

    loadInitialSettings();

    return () => {
      unlistenCloudProgress?.();
      unlistenCloudProgress = null;
    };
  });

  async function loadInitialSettings() {
    try {
      const settings = await loadSettings();
      dataDir = settings.dataDir || "";
      savedataDir = settings.savedataDir || "";
      uploadServerUrl = settings.uploadServerUrl || "";
      uploadQq = settings.uploadQq || "";
      scoreSource = settings.scoreSource === "cloud" ? "cloud" : "local";
      cloudServerUrl = settings.cloudServerUrl || "";
      cloudCardId = settings.cloudCardId || "";
      cloudPassword = settings.cloudPassword || "";
      cloudPcbid = settings.cloudPcbid || "";
      await setBackgroundImage(settings.backgroundImage || "");
      if (dataDir || savedataDir || backgroundImage) {
        message = "Loaded saved settings.";
      }
    } catch (error) {
      message = String(error);
    }
  }

  async function chooseDataDir() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select contents/data"
    });
    if (typeof selected === "string") {
      dataDir = selected;
      resetResults();
      await persistSettings();
    }
  }

  async function chooseSavedataDir() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select asphyxia/savedata"
    });
    if (typeof selected === "string") {
      savedataDir = selected;
      resetResults();
      await persistSettings();
    }
  }

  async function chooseBackgroundImage() {
    const selected = await open({
      directory: false,
      multiple: false,
      title: "Select B50 background image",
      filters: [
        {
          name: "Image",
          extensions: ["png", "jpg", "jpeg", "webp", "bmp", "gif"]
        }
      ]
    });
    if (typeof selected === "string") {
      await setBackgroundImage(selected);
      await persistSettings();
    }
  }

  async function clearBackgroundImage() {
    await setBackgroundImage("");
    await persistSettings();
  }

  async function selectPlayer(refid: string) {
    if (isBusy || selectedRefid === refid) {
      return;
    }
    selectedRefid = refid;
    await generateB50();
  }

  function resetResults() {
    players = [];
    selectedRefid = "";
    b50 = null;
    cloudProgressStage = "";
    message = "";
  }

  async function scanInputs() {
    if (!dataDir || !savedataDir) {
      message = "Select both folders first.";
      return;
    }

    isBusy = true;
    b50 = null;
    message = "";
    try {
      const result = await scanInputFolders(dataDir, savedataDir);
      players = result.players;
      selectedRefid = players[0]?.refid ?? "";
      message = players.length
        ? `Found ${players.length} SDVX 7 player profile${players.length > 1 ? "s" : ""}.`
        : "No SDVX 7 player with score data was found.";
      await persistSettings();
      if (selectedRefid) {
        await generateB50();
      }
    } catch (error) {
      message = String(error);
      players = [];
      selectedRefid = "";
    } finally {
      isBusy = false;
    }
  }

  async function generateB50() {
    if (!selectedRefid) {
      message = "Select a player first.";
      return;
    }

    const requestId = ++generateRequestId;
    const refid = selectedRefid;
    isBusy = true;
    b50 = null;
    message = "";
    try {
      const result = await generateB50Data(dataDir, savedataDir, refid);
      if (requestId === generateRequestId) {
        b50 = result;
        message = `Generated ${result.cards.length} cards for ${result.player.name}.`;
      }
    } catch (error) {
      if (requestId === generateRequestId) {
        message = String(error);
      }
    } finally {
      if (requestId === generateRequestId) {
        isBusy = false;
      }
    }
  }

  async function generateCloudB50() {
    if (!dataDir) {
      message = "Select contents/data first.";
      return;
    }
    if (!cloudServerUrl || !cloudCardId) {
      message = "Cloud server URL and card ID are required.";
      return;
    }

    const requestId = ++generateRequestId;
    const cloudRequestId = `cloud-${Date.now()}-${requestId}`;
    cloudProgressRequestId = cloudRequestId;
    cloudProgressStage = "validate_data";
    isBusy = true;
    players = [];
    selectedRefid = "";
    b50 = null;
    message = cloudProgressText(cloudProgressStage);
    try {
      await persistSettings();
      const result = await generateCloudB50Data(
        dataDir,
        normalizeUploadServerUrl(cloudServerUrl),
        cloudCardId,
        cloudPassword,
        cloudPcbid,
        cloudRequestId
      );
      if (requestId === generateRequestId) {
        b50 = result;
        cloudProgressStage = "";
        message = `Generated ${result.cards.length} cloud cards for ${result.player.name}.`;
      }
    } catch (error) {
      if (requestId === generateRequestId) {
        cloudProgressStage = "";
        message = String(error);
      }
    } finally {
      if (requestId === generateRequestId) {
        isBusy = false;
        cloudProgressRequestId = "";
      }
    }
  }

  function cloudProgressText(stage: string) {
    return cloudProgressLabels[stage] ?? "Working...";
  }

  async function exportPng() {
    if (!b50) {
      message = "Generate a B50 preview first.";
      return;
    }
    const board = document.getElementById("export-board");
    if (!board) {
      message = "Export board is not available.";
      return;
    }

    const defaultName = `sdvx-b50-${sanitizeName(b50.player.name)}-nemsys-${new Date()
      .toISOString()
      .slice(0, 10)}.png`;
    const defaultPath = await defaultOutputPath(defaultName);
    const outputPath = await save({
      title: "Save B50 PNG",
      defaultPath,
      filters: [{ name: "PNG Image", extensions: ["png"] }]
    });
    if (!outputPath) {
      return;
    }

    isExporting = true;
    message = "";
    try {
      const bytes = await renderBoardPng(board);
      await savePng(bytes, outputPath);
      message = `Saved ${outputPath}.`;
    } catch (error) {
      message = `Export failed: ${String(error)}`;
    } finally {
      isExporting = false;
    }
  }

  async function persistSettings() {
    try {
      await saveSettings(
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
      );
    } catch (error) {
      message = String(error);
    }
  }

  async function updateScoreSource(value: string) {
    scoreSource = value === "cloud" ? "cloud" : "local";
    resetResults();
    await persistSettings();
  }

  async function updateUploadServerUrl(value: string) {
    uploadServerUrl = value.trim();
    await persistSettings();
  }

  async function updateUploadQq(value: string) {
    uploadQq = value.replace(/\D/g, "");
    await persistSettings();
  }

  async function updateCloudServerUrl(value: string) {
    cloudServerUrl = value.trim();
    await persistSettings();
  }

  async function updateCloudCardId(value: string) {
    cloudCardId = value.trim();
    await persistSettings();
  }

  async function updateCloudPassword(value: string) {
    cloudPassword = value;
    await persistSettings();
  }

  async function updateCloudPcbid(value: string) {
    cloudPcbid = value.trim();
    await persistSettings();
  }

  async function uploadB50ToCloud() {
    if (!b50) {
      message = "Generate a B50 preview first.";
      return;
    }

    isUploading = true;
    message = "";
    try {
      message = await uploadB50IfConfigured(b50);
    } finally {
      isUploading = false;
    }
  }

  async function uploadB50IfConfigured(result: B50Result) {
    if (!uploadServerUrl && !uploadQq) {
      return "Cloud upload skipped: server address and QQ number are both required.";
    }
    if (!uploadServerUrl || !uploadQq) {
      return "Cloud upload skipped: server address and QQ number are both required.";
    }
    if (!/^\d{5,12}$/.test(uploadQq)) {
      return "Cloud upload skipped: QQ number must be 5 to 12 digits.";
    }
    if (!result.cards.length) {
      return "Cloud upload skipped: B50 data is empty.";
    }

    try {
      const uploadResult = await uploadB50(normalizeUploadServerUrl(uploadServerUrl), uploadQq, result);
      return uploadResult.message || "Cloud upload complete.";
    } catch (error) {
      return `Cloud upload failed: ${String(error)}`;
    }
  }

  function normalizeUploadServerUrl(value: string) {
    const trimmed = value.trim();
    return /^https?:\/\//i.test(trimmed) ? trimmed : `http://${trimmed}`;
  }

  async function setBackgroundImage(path: string) {
    backgroundImage = path;
    backgroundImageUrl = "";
    exportColumns = 5;
    const loadId = ++backgroundLoadId;

    if (!path) {
      return;
    }

    try {
      const dataUrl = await readImageDataUrl(path);
      if (loadId === backgroundLoadId) {
        backgroundImageUrl = dataUrl;
        exportColumns = await pickExportColumns(dataUrl);
      }
    } catch (error) {
      if (loadId === backgroundLoadId) {
        message = `Failed to load background image: ${String(error)}`;
      }
    }
  }

  async function pickExportColumns(dataUrl: string) {
    const { width, height } = await loadImageSize(dataUrl);
    const targetRatio = width / height;
    let bestColumns = 5;
    let bestDistance = Number.POSITIVE_INFINITY;

    for (let columns = 3; columns <= 10; columns += 1) {
      const ratio = estimateBoardRatio(columns);
      const distance = Math.abs(Math.log(ratio / targetRatio));
      if (distance < bestDistance) {
        bestColumns = columns;
        bestDistance = distance;
      }
    }

    return bestColumns;
  }

  function estimateBoardRatio(columns: number) {
    const cardWidth = 328;
    const cardHeight = cardWidth / 4;
    const gap = 12.8;
    const padding = 24 * 2;
    const header = 53;
    const rows = Math.ceil(50 / columns);
    const width = padding + columns * cardWidth + Math.max(0, columns - 1) * gap;
    const height = padding + header + rows * cardHeight + Math.max(0, rows - 1) * gap;

    return width / height;
  }

  function loadImageSize(src: string) {
    return new Promise<{ width: number; height: number }>((resolve, reject) => {
      const image = new Image();
      image.onload = () => resolve({ width: image.naturalWidth, height: image.naturalHeight });
      image.onerror = () => reject(new Error("Failed to inspect background image size."));
      image.src = src;
    });
  }
</script>

<main class="page">
  <div class="shell">
    <Sidebar
      {dataDir}
      {savedataDir}
      {backgroundImage}
      {uploadServerUrl}
      {uploadQq}
      {scoreSource}
      {cloudServerUrl}
      {cloudCardId}
      {cloudPassword}
      {cloudPcbid}
      {players}
      {selectedRefid}
      hasB50={Boolean(b50)}
      {message}
      {isBusy}
      {isExporting}
      {isUploading}
      cloudProgressText={cloudProgressStage ? cloudProgressText(cloudProgressStage) : ""}
      {chooseDataDir}
      {chooseSavedataDir}
      {chooseBackgroundImage}
      {clearBackgroundImage}
      {updateUploadServerUrl}
      {updateUploadQq}
      {updateScoreSource}
      {updateCloudServerUrl}
      {updateCloudCardId}
      {updateCloudPassword}
      {updateCloudPcbid}
      {scanInputs}
      {generateCloudB50}
      {exportPng}
      {uploadB50ToCloud}
      {selectPlayer}
    />

    <section class="workspace">
      {#if b50}
        <B50Preview {b50} {backgroundImageUrl} {exportColumns} />
      {:else}
        <EmptyState />
      {/if}
    </section>
  </div>
</main>
