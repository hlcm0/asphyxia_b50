<script lang="ts">
  import type { Messages } from "../lib/i18n";
  import type { PlayerSummary } from "../types";

  export let players: PlayerSummary[] = [];
  export let selectedRefid = "";
  export let isBusy = false;
  export let t: Messages;
  export let selectPlayer: (refid: string) => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">{t.players}</div>
  {#if players.length}
    <div class="player-list">
      {#each players as player}
        <label class:selected={selectedRefid === player.refid} class="player-row">
          <input
            checked={selectedRefid === player.refid}
            disabled={isBusy}
            name="selected-player"
            type="radio"
            value={player.refid}
            on:change={() => selectPlayer(player.refid)}
          />
          <span>
            <strong>{player.name}</strong>
            <small>ID {player.sdvxId} / {t.scoreCount(player.scoreCount)}</small>
          </span>
        </label>
      {/each}
    </div>
  {:else}
    <p class="muted">{t.scanToListProfiles}</p>
  {/if}
</section>
