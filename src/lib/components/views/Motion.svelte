<script lang="ts">
  import { app } from "$lib/app.svelte";
  import Shaping from "../Shaping.svelte";
  import OutputMap from "../OutputMap.svelte";
  import Range from "../Range.svelte";
  import { KEY_CODES } from "$lib/keycodes";
  import { hasKey, NO_KEY, orientation, setOrientation } from "$lib/bitfields";
  import { tr } from "$lib/i18n";

  let side = $state<"aim" | "tilt">("aim");

  const p = $derived(app.profile);
  const motion = $derived(p ? p[side] : null);

  const BEHAVIOR: [number, string][] = [
    [0, "behOff"],
    [1, "behButton"],
    [2, "behToggle"],
    [3, "behAlways"],
  ];

  const ORIENTATION: [number, string][] = [
    [0, "orYaw"],
    [1, "orRoll"],
    [2, "orCombine"],
  ];
</script>

{#if p && motion}
  <nav class="seg">
    <button class:active={side === "aim"} aria-pressed={side === "aim"} onclick={() => (side = "aim")}>{tr("aim")}</button>
    <button class:active={side === "tilt"} aria-pressed={side === "tilt"} onclick={() => (side = "tilt")}>{tr("tilt")}</button>
  </nav>

  <div class="grid">
    <section class="card">
      <label class="field-row">
        <span>{tr("activateBehavior")}</span>
        <select bind:value={motion.sensorProfileStatus} onchange={() => app.saveProfile()}>
          {#each BEHAVIOR as [v, key] (v)}
            <option value={v}>{tr(key as "behOff")}</option>
          {/each}
        </select>
      </label>
      <p class="hint">{tr("hintGyroActivate")}</p>
      <fieldset class="dep" disabled={motion.sensorProfileStatus === 0}>
        <label class="field-row">
          <span>{tr("orientation")}</span>
          <select value={orientation(motion.activeAxis)} onchange={(e) => { motion.activeAxis = setOrientation(Number(e.currentTarget.value)); app.saveProfile(); }}>
            {#each ORIENTATION as [v, key] (v)}
              <option value={v}>{tr(key as "orYaw")}</option>
            {/each}
          </select>
        </label>
        <fieldset class="dep" disabled={motion.sensorProfileStatus === 3}>
          <label class="field-row">
            <span>{tr("triggerKey")}</span>
            <select bind:value={motion.sensorQuickKeyValue} onchange={() => app.saveProfile()}>
              <option value={NO_KEY}>{tr("none")}</option>
              {#each KEY_CODES.filter(([code]) => hasKey(code)) as [code, name] (code)}
                <option value={code}>{name}</option>
              {/each}
            </select>
          </label>
        </fieldset>
        <Shaping dead={motion.dead} linear={motion.linear} onchange={() => app.saveProfile()} />
      </fieldset>
    </section>

    <section class="card">
      <h3>{tr("output")}</h3>
      <p class="hint">{tr("hintOutput")}</p>
      <fieldset class="dep" disabled={motion.sensorProfileStatus === 0}>
        <OutputMap map={motion.map} withZ={app.model.family === "V2"} onchange={() => app.saveProfile()} />
        {#if app.model.family === "V2"}
          <Range label={tr("stickSensitivity")} bind:value={motion.stickSensitivity} max={100} onchange={() => app.saveProfile()} />
        {/if}
      </fieldset>
    </section>
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
