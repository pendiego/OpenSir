<script lang="ts">
  import Range from "./Range.svelte";
  import DualRange from "./DualRange.svelte";
  import CurveEditor from "./CurveEditor.svelte";
  import { tr } from "$lib/i18n";
  import type { DeadZone, Linear } from "$lib/model";

  let {
    dead,
    linear,
    onchange,
  }: {
    dead: DeadZone;
    linear: Linear;
    onchange: () => void;
  } = $props();
</script>

<div class="shaping">
  <div class="pair">
    <div class="band">
      <DualRange label={tr("deadzone")} unit="%" bind:low={dead.front} bind:high={dead.back} max={100} {onchange} />
      <p class="hint">{tr("deadzoneHint")}</p>
    </div>
    <div class="band">
      <DualRange label={tr("antiDeadzone")} unit="%" bind:low={dead.antiFront} bind:high={dead.antiBack} max={100} {onchange} />
      <p class="hint">{tr("antiDeadzoneHint")}</p>
    </div>
  </div>

  <CurveEditor {dead} {linear} {onchange} />
  {#if linear.curveType === 1 || linear.curveType === 2}
    <Range label={tr("curvature")} unit="%" bind:value={linear.curvature} max={100} {onchange} />
  {/if}
</div>

<style>
  .shaping {
    display: grid;
    gap: 12px;
    container: shaping / inline-size;
  }
  .pair,
  .band {
    display: grid;
    gap: 12px;
    align-content: start;
  }
  @container shaping (min-width: 520px) {
    .pair {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      align-items: start;
    }
  }
</style>
