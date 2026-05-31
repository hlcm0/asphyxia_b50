<script lang="ts">
  export let dataDir = "";
  export let savedataDir = "";
  export let isBusy = false;
  export let chooseDataDir: () => void | Promise<void>;
  export let chooseSavedataDir: () => void | Promise<void>;
  export let scanInputs: () => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">Folders</div>
  <div class="path-field">
    <span class="path-label">Game data</span>
    <button class="path-button" type="button" on:click={chooseDataDir}>
      <strong>{dataDir || "Select contents/data"}</strong>
    </button>
  </div>
  <div class="path-field">
    <span class="path-label">Savedata</span>
    <button class="path-button" type="button" on:click={chooseSavedataDir}>
      <strong>{savedataDir || "Select asphyxia/savedata"}</strong>
    </button>
  </div>
  <button class="primary" type="button" disabled={isBusy || !dataDir || !savedataDir} on:click={scanInputs}>
    {isBusy ? "Working..." : "Scan Players"}
  </button>
</section>
