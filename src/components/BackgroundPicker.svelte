<script lang="ts">
  import type { Messages } from "../lib/i18n";

  export let backgroundImage = "";
  export let t: Messages;
  export let chooseBackgroundImage: () => void | Promise<void>;
  export let clearBackgroundImage: () => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">{t.background}</div>
  <div class="path-field">
    <span class="path-label">{t.previewAndExport}</span>
    <button class="path-button" type="button" on:click={chooseBackgroundImage}>
      <strong>{backgroundImage || t.selectBackgroundImage}</strong>
    </button>
  </div>
  <button class="secondary" type="button" disabled={!backgroundImage} on:click={clearBackgroundImage}>
    {t.clearBackground}
  </button>
</section>
