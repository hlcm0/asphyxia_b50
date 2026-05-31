<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import B50Preview from "./components/B50Preview.svelte";
  import EmptyState from "./components/EmptyState.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import {
    defaultOutputPath,
    generateB50 as generateB50Data,
    loadSettings,
    readImageDataUrl,
    savePng,
    saveSettings,
    scanInputs as scanInputFolders
  } from "./lib/api";
  import { renderBoardPng, sanitizeName } from "./lib/exportPng";
  import type { B50Result, PlayerSummary } from "./types";

  let dataDir = "";
  let savedataDir = "";
  let backgroundImage = "";
  let players: PlayerSummary[] = [];
  let selectedRefid = "";
  let b50: B50Result | null = null;
  let message = "";
  let isBusy = false;
  let isExporting = false;
  let backgroundImageUrl = "";
  let exportColumns = 5;
  let backgroundLoadId = 0;

  onMount(async () => {
    try {
      const settings = await loadSettings();
      dataDir = settings.dataDir || "";
      savedataDir = settings.savedataDir || "";
      await setBackgroundImage(settings.backgroundImage || "");
      if (dataDir || savedataDir || backgroundImage) {
        message = "Loaded saved settings.";
      }
    } catch (error) {
      message = String(error);
    }
  });

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

  function selectPlayer(refid: string) {
    selectedRefid = refid;
  }

  function resetResults() {
    players = [];
    selectedRefid = "";
    b50 = null;
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

    isBusy = true;
    b50 = null;
    message = "";
    try {
      b50 = await generateB50Data(dataDir, savedataDir, selectedRefid);
      message = `Generated ${b50.cards.length} cards for ${b50.player.name}.`;
    } catch (error) {
      message = String(error);
    } finally {
      isBusy = false;
    }
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
      await saveSettings(dataDir, savedataDir, backgroundImage);
    } catch (error) {
      message = String(error);
    }
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
      {players}
      {selectedRefid}
      hasB50={Boolean(b50)}
      {message}
      {isBusy}
      {isExporting}
      {chooseDataDir}
      {chooseSavedataDir}
      {chooseBackgroundImage}
      {clearBackgroundImage}
      {scanInputs}
      {generateB50}
      {exportPng}
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
