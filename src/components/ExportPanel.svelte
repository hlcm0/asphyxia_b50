<script lang="ts">
  export let hasB50 = false;
  export let isExporting = false;
  export let message = "";
  export let exportPng: () => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">Export</div>
  <button class="primary" type="button" disabled={!hasB50 || isExporting} on:click={exportPng}>
    {isExporting ? "Exporting..." : "Export PNG"}
  </button>
  {#if message}
    <p class="status">{message}</p>
  {/if}
</section>
