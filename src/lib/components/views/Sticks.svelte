<script lang="ts">
  import { app } from "$lib/app.svelte";
  import Shaping from "../Shaping.svelte";
  import OutputMap from "../OutputMap.svelte";
  import StickGate from "../StickGate.svelte";
  import { tr } from "$lib/i18n";

  let side = $state<"leftStick" | "rightStick">("leftStick");

  const p = $derived(app.profile);
  const stick = $derived(p ? p[side] : null);
</script>

{#if p && stick}
  <nav class="seg">
    <button class:active={side === "leftStick"} aria-pressed={side === "leftStick"} onclick={() => (side = "leftStick")}>{tr("left")}</button>
    <button class:active={side === "rightStick"} aria-pressed={side === "rightStick"} onclick={() => (side = "rightStick")}>{tr("right")}</button>
  </nav>

  <div class="grid">
    <section class="card">
      <label class="field-row">
        <span>{tr("enabled")}</span>
        <input type="checkbox" bind:checked={stick.enabled} onchange={() => app.saveProfile()} />
      </label>
      <fieldset class="dep" disabled={!stick.enabled}>
        <label class="field-row">
          <span>{tr("square")}</span>
          <input type="checkbox" bind:checked={stick.square} onchange={() => app.saveProfile()} />
        </label>
        <p class="hint">{tr("hintSquare")}</p>
        <Shaping dead={stick.dead} linear={stick.linear} onchange={() => app.saveProfile()} />
      </fieldset>
    </section>

    <div class="side">
      <section class="card">
        <h3>{tr("deadzone")}</h3>
        <fieldset class="dep" disabled={!stick.enabled}>
          <StickGate dead={stick.dead} square={stick.square} label={`${tr(side === "leftStick" ? "left" : "right")} ${tr("deadzone")}`} />
        </fieldset>
      </section>
      <section class="card">
        <h3>{tr("output")}</h3>
        <fieldset class="dep" disabled={!stick.enabled}>
          <OutputMap map={stick.map} onchange={() => app.saveProfile()} />
        </fieldset>
      </section>
    </div>
  </div>
{/if}

<style>
  .seg {
    margin-bottom: 16px;
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
  .side {
    display: grid;
    gap: 16px;
  }
  h3 {
    margin: 0;
    font-size: 15px;
    font-weight: 500;
  }
  @media (max-width: 959px) {
    .grid {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
