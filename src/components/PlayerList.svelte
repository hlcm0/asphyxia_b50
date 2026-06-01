<script lang="ts">
  import type { PlayerSummary } from "../types";

  export let players: PlayerSummary[] = [];
  export let selectedRefid = "";
  export let isBusy = false;
  export let selectPlayer: (refid: string) => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">Players</div>
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
            <small>ID {player.sdvxId} / {player.scoreCount} scores</small>
          </span>
        </label>
      {/each}
    </div>
  {:else}
    <p class="muted">Scan folders to list SDVX 7 profiles.</p>
  {/if}
</section>
