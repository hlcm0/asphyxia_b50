<script lang="ts">
  import type { Messages } from "../lib/i18n";

  export let uploadServerUrl = "";
  export let uploadQq = "";
  export let isBusy = false;
  export let t: Messages;
  export let updateUploadServerUrl: (value: string) => void | Promise<void>;
  export let updateUploadQq: (value: string) => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">{t.b50Upload}</div>
  <label class="input-field">
    <span class="path-label">{t.serverAddress}</span>
    <input
      class="text-input"
      type="url"
      spellcheck="false"
      placeholder="https://example.com"
      value={uploadServerUrl}
      disabled={isBusy}
      on:change={(event) => updateUploadServerUrl(event.currentTarget.value)}
    />
  </label>
  <label class="input-field">
    <span class="path-label">{t.qqNumber}</span>
    <input
      class="text-input"
      type="text"
      inputmode="numeric"
      pattern="[0-9]*"
      spellcheck="false"
      placeholder="123456789"
      value={uploadQq}
      disabled={isBusy}
      on:change={(event) => updateUploadQq(event.currentTarget.value)}
    />
  </label>
</section>
