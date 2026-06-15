<script lang="ts">
  import type { Messages } from "../lib/i18n";

  export let dataDir = "";
  export let cloudServerUrl = "";
  export let cloudCardId = "";
  export let cloudPassword = "";
  export let cloudPcbid = "";
  export let isBusy = false;
  export let cloudProgressText = "";
  export let t: Messages;
  export let updateCloudServerUrl: (value: string) => void | Promise<void>;
  export let updateCloudCardId: (value: string) => void | Promise<void>;
  export let updateCloudPassword: (value: string) => void | Promise<void>;
  export let updateCloudPcbid: (value: string) => void | Promise<void>;
  export let generateCloudB50: () => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">{t.cloudScores}</div>
  <label class="input-field">
    <span class="path-label">{t.serverUrl}</span>
    <input
      class="text-input"
      type="url"
      spellcheck="false"
      placeholder="http://localhost:8083"
      value={cloudServerUrl}
      disabled={isBusy}
      on:change={(event) => updateCloudServerUrl(event.currentTarget.value)}
    />
  </label>
  <label class="input-field">
    <span class="path-label">{t.cardId}</span>
    <input
      class="text-input"
      type="text"
      spellcheck="false"
      placeholder="E004..."
      value={cloudCardId}
      disabled={isBusy}
      on:change={(event) => updateCloudCardId(event.currentTarget.value)}
    />
  </label>
  <label class="input-field">
    <span class="path-label">{t.password}</span>
    <input
      class="text-input"
      type="password"
      spellcheck="false"
      placeholder={t.optional}
      value={cloudPassword}
      disabled={isBusy}
      on:change={(event) => updateCloudPassword(event.currentTarget.value)}
    />
  </label>
  <label class="input-field">
    <span class="path-label">PCBID</span>
    <input
      class="text-input"
      type="text"
      spellcheck="false"
      placeholder="00010203040506070809"
      value={cloudPcbid}
      disabled={isBusy}
      on:change={(event) => updateCloudPcbid(event.currentTarget.value)}
    />
  </label>
  <button
    class="primary"
    type="button"
    disabled={isBusy || !dataDir || !cloudServerUrl || !cloudCardId}
    on:click={generateCloudB50}
  >
    {isBusy ? cloudProgressText || t.working : t.fetchCloudB50}
  </button>
</section>
