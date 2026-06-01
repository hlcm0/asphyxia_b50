<script lang="ts">
  export let uploadServerUrl = "";
  export let uploadQq = "";
  export let isBusy = false;
  export let updateUploadServerUrl: (value: string) => void | Promise<void>;
  export let updateUploadQq: (value: string) => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">Cloud Upload</div>
  <label class="input-field">
    <span class="path-label">Server address</span>
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
    <span class="path-label">QQ number</span>
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
