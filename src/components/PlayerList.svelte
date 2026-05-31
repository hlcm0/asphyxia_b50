<script lang="ts">
  import type { PlayerSummary } from "../types";

  export let players: PlayerSummary[] = [];
  export let selectedRefid = "";
  export let isBusy = false;
  export let selectPlayer: (refid: string) => void;
  export let generateB50: () => void | Promise<void>;
</script>

<section class="panel">
  <div class="panel-title">Players</div>
  {#if players.length}
    <div class="player-list">
      {#each players as player}
        <label class:selected={selectedRefid === player.refid} class="player-row">
          <input
            checked={selectedRefid === player.refid}
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
    <button class="primary" type="button" disabled={isBusy || !selectedRefid} on:click={generateB50}>
      Generate B50
    </button>
  {:else}
    <p class="muted">Scan folders to list SDVX 7 profiles.</p>
  {/if}
</section>
