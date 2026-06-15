<script lang="ts">
  import type { Messages } from "../lib/i18n";

  export let hasB50 = false;
  export let isExporting = false;
  export let isUploading = false;
  export let message = "";
  export let t: Messages;
  export let exportPng: () => void | Promise<void>;
  export let uploadB50ToCloud: () => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">{t.export}</div>
  <button class="primary" type="button" disabled={!hasB50 || isExporting} on:click={exportPng}>
    {isExporting ? t.exporting : t.exportPng}
  </button>
  <button
    class="primary panel-action"
    type="button"
    disabled={!hasB50 || isUploading}
    on:click={uploadB50ToCloud}
  >
    {isUploading ? t.uploading : t.uploadToCloud}
  </button>
  {#if message}
    <p class="status">{message}</p>
  {/if}
</section>
