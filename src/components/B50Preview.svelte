<script lang="ts">
  import type { B50Result } from "../types";
  import B50Card from "./B50Card.svelte";

  export let b50: B50Result;
  export let backgroundImageUrl = "";
  export let exportColumns = 5;

  $: exportWidth = 48 + exportColumns * 328 + Math.max(0, exportColumns - 1) * 12.8;
  $: backgroundStyle = backgroundImageUrl
    ? `--b50-bg-image: url("${backgroundImageUrl.replace(/"/g, '\\"')}");`
    : "";
  $: boardStyle = `${backgroundStyle} --b50-export-columns: ${exportColumns}; --b50-export-width: ${exportWidth}px;`;
</script>

<div class="export-stage">
  <div
    class:has-custom-background={Boolean(backgroundImageUrl)}
    class="vf-export-board"
    id="export-board"
    style={boardStyle}
  >
    <div class="vf-export-header">
      <div class="vf-export-title">SDVX B50</div>
      <div class="vf-export-info">
        <span class="vf-export-meta">∇</span>
        <span class="vf-export-info-divider">|</span>
        <span class="vf-export-info-label">PLAYER:</span>
        <span class="vf-export-info-value">{b50.player.name}</span>
        <span class="vf-export-info-divider">|</span>
        <span class="vf-export-info-label">VF:</span>
        <span class="vf-export-info-value">{b50.totalVf}</span>
      </div>
    </div>
    <div class="vf-grid-container">
      {#each b50.cards as card}
        <B50Card {card} />
      {/each}
    </div>
  </div>
</div>
