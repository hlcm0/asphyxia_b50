<script lang="ts">
  import type { B50Card } from "../types";

  export let card: B50Card;

  const formatScore = (score: number) => score.toLocaleString("en-US");
  const diffClass = (label: string) => `diff-${label.toLowerCase()}`;
</script>

<article class="vf-card">
  <div class="vf-card-header">
    <div class="vf-card-rank">#{card.rank}</div>
    <div class="vf-card-lamp">{card.clearLamp}</div>
  </div>
  <div class="vf-card-content-wrapper">
    <div class="vf-card-jacket">
      <img src={card.jacketPath} alt={`Jacket ${card.mid}`} />
    </div>
    <div class="vf-card-info">
      <div class="vf-card-songname" title={card.title}>{card.title}</div>
      <div class="vf-card-details">
        <span class={`vf-card-difficulty ${diffClass(card.difficultyLabel)}`}>
          {card.difficultyLabel}{card.level}
        </span>
        <span class="vf-card-score">{formatScore(card.score)}</span>
        <span class="vf-card-vf">{card.singleVf}</span>
      </div>
    </div>
  </div>
</article>
