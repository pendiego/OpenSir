<script lang="ts">
  import { app } from "$lib/app.svelte";
  import Shaping from "../Shaping.svelte";
  import Range from "../Range.svelte";
  import { tr } from "$lib/i18n";
  import { hairMode, setHairMode } from "$lib/bitfields";

  let side = $state<"l2" | "r2">("l2");

  const p = $derived(app.profile);
  const trig = $derived(p ? p[side] : null);

  const HAIR: [number, string][] = [
    [0, "hairOff"],
    [1, "hairRapid"],
    [2, "hairFix"],
  ];
</script>

{#if p && trig}
  <nav class="seg">
    <button class:active={side === "l2"} aria-pressed={side === "l2"} onclick={() => (side = "l2")}>L2</button>
    <button class:active={side === "r2"} aria-pressed={side === "r2"} onclick={() => (side = "r2")}>R2</button>
  </nav>

  <div class="grid">
    <section class="card">
      <Shaping dead={trig.dead} linear={trig.linear} onchange={() => app.saveProfile()} />
    </section>

    <section class="card">
      <h3>{tr("hair")}</h3>
      <div class="section-label">{tr("hairMode")}</div>
      <p class="hint">{tr("hintHair")}</p>
      <div class="seg wide" role="group" aria-label={tr("hairMode")}>
        {#each HAIR as [v, key] (v)}
          <button
            class:active={hairMode(trig.quickTriggerStatus) === v}
            aria-pressed={hairMode(trig.quickTriggerStatus) === v}
            onclick={() => {
              trig.quickTriggerStatus = setHairMode(trig.quickTriggerStatus, v);
              app.saveProfile();
            }}>{tr(key as "hairOff")}</button
          >
        {/each}
      </div>
      <fieldset class="dep" disabled={hairMode(trig.quickTriggerStatus) === 0}>
        <Range label={tr("rangeBegin")} unit="%" bind:value={trig.quickTriggerStart} max={100} onchange={() => app.saveProfile()} />
        <Range label={tr("rangeEnd")} unit="%" bind:value={trig.quickTriggerEnd} max={100} onchange={() => app.saveProfile()} />
      </fieldset>
    </section>
  </div>
{/if}

<style>
  .seg {
    margin-bottom: 16px;
  }
  .wide {
    display: grid;
  }
  .grid {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
    gap: 16px;
    align-items: start;
  }
  .card {
    display: grid;
    gap: 12px;
    align-content: start;
  }
  h3 {
    margin: 0;
    font-size: 15px;
    font-weight: 500;
  }
  .section-label {
    margin: 0;
  }
  @media (max-width: 959px) {
    .grid {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
