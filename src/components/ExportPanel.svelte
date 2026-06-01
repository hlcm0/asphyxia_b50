<script lang="ts">
  export let hasB50 = false;
  export let isExporting = false;
  export let isUploading = false;
  export let message = "";
  export let exportPng: () => void | Promise<void>;
  export let uploadB50ToCloud: () => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">Export</div>
  <button class="primary" type="button" disabled={!hasB50 || isExporting} on:click={exportPng}>
    {isExporting ? "Exporting..." : "Export PNG"}
  </button>
  <button
    class="primary panel-action"
    type="button"
    disabled={!hasB50 || isUploading}
    on:click={uploadB50ToCloud}
  >
    {isUploading ? "Uploading..." : "Upload to Cloud"}
  </button>
  {#if message}
    <p class="status">{message}</p>
  {/if}
</section>
