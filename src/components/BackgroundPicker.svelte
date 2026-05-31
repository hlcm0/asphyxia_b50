<script lang="ts">
  export let backgroundImage = "";
  export let chooseBackgroundImage: () => void | Promise<void>;
  export let clearBackgroundImage: () => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">Background</div>
  <div class="path-field">
    <span class="path-label">Preview and export</span>
    <button class="path-button" type="button" on:click={chooseBackgroundImage}>
      <strong>{backgroundImage || "Select background image"}</strong>
    </button>
  </div>
  <button class="secondary" type="button" disabled={!backgroundImage} on:click={clearBackgroundImage}>
    Clear Background
  </button>
</section>
