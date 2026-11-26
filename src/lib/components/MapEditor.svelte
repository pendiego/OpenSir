<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { tr } from "$lib/i18n";
  import { keyName } from "$lib/keycodes";
  import type { KeyData } from "$lib/keys";
  import KeyPicker, { keyLabel } from "./KeyPicker.svelte";
  import Range from "./Range.svelte";
  import TurboToggle from "./TurboToggle.svelte";
  import MacroEditor from "./MacroEditor.svelte";

  let {
    data,
    onchange,
    label,
  }: {
    data: KeyData;
    onchange: () => void;
    label: string;
  } = $props();

  const targets = $derived(data.map.filter((v) => v > 0));
  const macro = $derived("steps" in data ? data : null);
  const touch = $derived("triggerMode" in data ? data : null);
  // T3CE touch-key trigger modes (vendor names Off, OneShot, Hold, AlwaysOn).
  const TOUCH_MODES = ["touchModeOff", "touchModeOnce", "touchModeHold", "touchModeAlways"] as const;

  // Clearing every target is the device's "default" (no remap) state.
  function setTargets(codes: number[]) {
    const map: [number, number, number] = [0, 0, 0];
    codes.forEach((c, i) => (map[i] = c));
    data.map = map;
    data.mapped = codes.length > 0;
    onchange();
  }
</script>

<section class="map-editor">
  <!-- Stays at the top of the scrolling inspector: which button, and what it sends now. -->
  <div class="head">
    <h3 class="title">{label}</h3>

    <p class="sends">
      {#if targets.length > 0}
        <span class="muted">{tr("remapNow", { from: label })}</span>
        {#each targets as code, i (i)}
          <kbd>{keyLabel(keyName(code))}</kbd>
        {/each}
      {:else}
        <span class="muted">{tr("mappedTo")}</span>
        <span>{tr("defaultNoRemap")}</span>
      {/if}
    </p>
  </div>

  <KeyPicker bind:selected={() => targets, setTargets} {onchange} />

  <button type="button" class="clear" onclick={() => setTargets([])} disabled={targets.length === 0}>
    {tr("restoreDefault")}
  </button>

  {#if !app.restricted}
    <h4 class="section-label">{tr("turbo")}</h4>
    <TurboToggle bind:turbo={data.turbo} bind:toggleOnPress={data.toggleOnPress} {onchange} />
  {/if}

  {#if touch}
    <h4 class="section-label">{tr("touchKey")}</h4>
    <label class="field">
      <span>{tr("touchMode")}</span>
      <select bind:value={touch.triggerMode} onchange={onchange}>
        {#each TOUCH_MODES as key, i (key)}
          <option value={i}>{tr(key)}</option>
        {/each}
      </select>
    </label>
    <Range label={tr("touchSensitivity")} bind:value={touch.sensitivity} max={1000} {onchange} />
  {/if}

  {#if macro}
    <h4 class="section-label">{tr("macro")}</h4>
    <MacroEditor
      bind:steps={macro.steps}
      bind:macroEnabled={macro.macroEnabled}
      bind:macroPressToTrigger={macro.macroPressToTrigger}
      bind:macroCycle={macro.macroCycle}
      bind:cycleMs={macro.cycleMs}
      {onchange}
    />
  {/if}
</section>

<style>
  .map-editor {
    display: grid;
    gap: 8px;
  }
  /* Sticks over the inspector card's 20px padding too (top and sides), so nothing scrolls by around it. */
  .head {
    position: sticky;
    top: -20px;
    z-index: 1;
    display: grid;
    gap: 8px;
    margin: -20px -20px 0;
    padding: 20px 20px 8px;
    border-bottom: 1px solid var(--line);
    background: var(--surface);
    /* Hides the sub-pixel row of scrolled content left above it at fractional zoom. */
    box-shadow: 0 -2px 0 var(--surface);
  }
  .head .title {
    margin: 0;
  }
  .sends {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    margin: 0;
    font-size: 13px;
  }
  kbd {
    padding: 2px 6px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--raised);
    font-family: var(--mono);
    font-size: 12px;
    color: var(--fg);
  }
  .clear {
    justify-self: start;
  }
</style>
