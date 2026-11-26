<script lang="ts">
  import { app } from "$lib/app.svelte";
  import Range from "./Range.svelte";
  import { tr, type Key } from "$lib/i18n";
  import type { Hsv } from "$lib/types";
  import { lightSel } from "$lib/lightSel.svelte";

  // Mode codes in device order; a setting belongs to the mode whose name prefixes it.
  const MODES = ["singleColor", "breathing", "phantom", "rainbow", "radar"] as const;
  const MODE_KEYS: Record<(typeof MODES)[number], Key> = {
    singleColor: "modeSingleColor",
    breathing: "modeBreathing",
    phantom: "modePhantom",
    rainbow: "modeRainbow",
    radar: "modeRadar",
  };
  const ZONE_KEYS: Record<string, Key> = {
    panel: "zonePanel",
    home: "slotHome",
    lowerLeft: "slotLowerLeft",
    upperLeft: "slotUpperLeft",
    upperRight: "slotUpperRight",
    lowerRight: "slotLowerRight",
    profileNum: "zoneProfileNum",
    homeButton: "zoneHomeButton",
  };
  const SETTING_KEYS: Record<string, Key> = {
    homeRgbEffectIndex: "homeEffect",
    profileNumRgbEffectIndex: "profileEffect",
    homeRgbPhantomSpeed: "homePhantomSpeed",
    profileNumRgbPhantomSpeed: "profilePhantomSpeed",
  };
  const TAIL_KEYS: Record<string, Key> = {
    standbyTime: "standby",
    dockLedMode: "dockMode",
    dockLedBrightness: "dockBrightness",
    reportRate: "reportRate",
  };
  const STANDBY = [0, 1, 5, 10, 20];

  const light = $derived(app.effectLight);
  const spec = $derived(app.model.light);
  const anim = $derived(light && spec ? light.animations[lightSel.slot] : null);
  const modeName = $derived(anim ? MODES[anim.mode] : "");
  // Mode-specific settings, plus the ones no mode owns (K2 home and profile effects).
  const settings = $derived(
    spec && anim
      ? spec.settings
          .map((name, i) => ({ name, i }))
          .filter(({ name }) => name.startsWith(modeName) || !MODES.some((m) => name.startsWith(m)))
      : [],
  );
  const tails = $derived(spec && light ? spec.tail.map((name, i) => ({ name, i })).filter(({ name }) => !name.startsWith("deviceId")) : []);

  function save() {
    app.saveEffectLight();
  }

  function kindKey(name: string): Key {
    return SETTING_KEYS[name] ?? (name.endsWith("Hue") ? "hue" : name.endsWith("Speed") ? "speed" : "brightness");
  }

  // Upper bound of each value; the device ranges are not documented, so these are the UI's limits.
  function maxOf(name: string): number {
    if (name.endsWith("Hue")) return 255;
    if (name.endsWith("EffectIndex")) return 4;
    if (name === "reportRate") return 2;
    return 100;
  }

  function hsvToHex({ h, s, v }: Hsv): string {
    const c = (n: number) => {
      const k = (n + h / 60) % 6;
      const x = (v / 100) - (v / 100) * (s / 100) * Math.max(0, Math.min(k, 4 - k, 1));
      return Math.round(x * 255).toString(16).padStart(2, "0");
    };
    return `#${c(5)}${c(3)}${c(1)}`;
  }

  function hexToHsv(hex: string): Hsv {
    const r = parseInt(hex.slice(1, 3), 16) / 255;
    const g = parseInt(hex.slice(3, 5), 16) / 255;
    const b = parseInt(hex.slice(5, 7), 16) / 255;
    const max = Math.max(r, g, b);
    const d = max - Math.min(r, g, b);
    let h = 0;
    if (d) h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
    return { h: (Math.round(h * 60) + 360) % 360, s: max ? Math.round((d / max) * 100) : 0, v: Math.round(max * 100) };
  }

  function setZone(i: number, hex: string) {
    if (!anim) return;
    anim.zones[i] = hexToHsv(hex);
    save();
  }
</script>

{#if light && anim && spec}
  <div class="lights">
    <nav class="seg tabs" aria-label={tr("lightConfig")}>
      {#each [0, 1, 2, 3, 4] as i (i)}
        <button class:active={lightSel.slot === i} aria-pressed={lightSel.slot === i} onclick={() => (lightSel.slot = i)}>{tr("lightConfig")} {i + 1}</button>
      {/each}
    </nav>

    <section class="card">
      <div class="field-row">
        <span>{tr("mode")}</span>
        <nav class="seg">
          {#each MODES as m, i (m)}
            <button
              class:active={anim.mode === i}
              aria-pressed={anim.mode === i}
              onclick={() => {
                anim.mode = i;
                save();
              }}>{tr(MODE_KEYS[m])}</button
            >
          {/each}
        </nav>
      </div>
      {#each settings as { name, i } (name)}
        <Range label={tr(kindKey(name))} bind:value={anim.settings[i]} max={maxOf(name)} onchange={save} />
      {/each}
    </section>

    <section class="card">
      {#each spec.zones as zone, i (zone)}
        <label class="field-row">
          <span>{tr(ZONE_KEYS[zone])}</span>
          <span class="swatch">
            <span class="value">{hsvToHex(anim.zones[i])}</span>
            <input type="color" value={hsvToHex(anim.zones[i])} oninput={(e) => setZone(i, e.currentTarget.value)} onchange={save} />
          </span>
        </label>
      {/each}
      <label class="field-row">
        <span>{tr("wakeUp")}</span>
        <input type="checkbox" bind:checked={light.raiseWakeUp} onchange={save} />
      </label>
    </section>

    <section class="card">
      {#each tails as { name, i } (name)}
        {#if name === "dockLedMode"}
          <label class="field-row">
            <span>{tr(TAIL_KEYS[name])}</span>
            <input type="checkbox" checked={light.tail[i] !== 0} onchange={(e) => { light.tail[i] = e.currentTarget.checked ? 1 : 0; save(); }} />
          </label>
        {:else if name === "standbyTime"}
          <label class="field-row">
            <span>{tr(TAIL_KEYS[name])}</span>
            <select bind:value={light.tail[i]} onchange={save}>
              {#if !STANDBY.includes(light.tail[i])}<option value={light.tail[i]}>{tr("minutes", { n: light.tail[i] })}</option>{/if}
              {#each STANDBY as m (m)}
                <option value={m}>{m === 0 ? tr("standbyOff") : tr("minutes", { n: m })}</option>
              {/each}
            </select>
          </label>
        {:else}
          <Range label={tr(TAIL_KEYS[name])} bind:value={light.tail[i]} max={maxOf(name)} onchange={save} />
        {/if}
      {/each}
    </section>
  </div>
{/if}

<style>
  .lights {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(240px, 300px);
    gap: 16px;
    align-items: start;
  }
  .tabs {
    grid-column: 1 / -1;
    justify-self: start;
  }
  section {
    display: grid;
    gap: 4px;
    align-content: start;
  }
  .field-row + .field-row {
    border-top: 1px solid var(--line);
  }
  .swatch {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  @media (max-width: 959px) {
    .lights {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
