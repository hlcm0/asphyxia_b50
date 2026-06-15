<script lang="ts">
  import type { Messages } from "../lib/i18n";

  export let dataDir = "";
  export let savedataDir = "";
  export let scoreSource = "local";
  export let isBusy = false;
  export let t: Messages;
  export let chooseDataDir: () => void | Promise<void>;
  export let chooseSavedataDir: () => void | Promise<void>;
  export let scanInputs: () => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">{t.folders}</div>
  <div class="path-field">
    <span class="path-label">{t.gameData}</span>
    <button class="path-button" type="button" on:click={chooseDataDir}>
      <strong>{dataDir || t.selectContentsData}</strong>
    </button>
  </div>
  {#if scoreSource === "local"}
    <div class="path-field">
      <span class="path-label">{t.savedata}</span>
      <button class="path-button" type="button" on:click={chooseSavedataDir}>
        <strong>{savedataDir || t.selectSavedata}</strong>
      </button>
    </div>
    <button class="primary" type="button" disabled={isBusy || !dataDir || !savedataDir} on:click={scanInputs}>
      {isBusy ? t.working : t.scanPlayers}
    </button>
  {/if}
</section>
