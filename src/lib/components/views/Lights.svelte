<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { app } from "$lib/app.svelte";
  import Range from "../Range.svelte";
  import { tr } from "$lib/i18n";
  import { hexRgb, parseRgb } from "$lib/util";
  import type { Frame, Rgb } from "$lib/model";
  import { lightSel } from "$lib/lightSel.svelte";
  import { LIGHT_PRESETS, applyPreset, matchPreset, type LightPreset } from "$lib/lightPresets";
  import EffectLights from "../EffectLights.svelte";

  // Behaviour follows the vendor app's light page (docs/spec/ui-cyclone2.md §8).
  const MAX_FRAMES = 8;
  /** The page edits 4 animations; ConfigIndex 4 is not editable there and is swapped for 0 on entry. */
  const SLOTS = [0, 1, 2, 3];
  /** Sleep-when-idle choices in minutes (0 = off), cycled in the vendor app. */
  const STANDBY = [0, 1, 5, 10, 20];
  // Frame LED index for each numbered bulb (1 lower left, 2 Home, 3 lower right, 4 back); index 2 is never shown.
  const LEDS: [number, string][] = [
    [0, "slotLowerLeft"],
    [3, "slotHome"],
    [1, "slotLowerRight"],
    [4, "slotUpperRight"],
  ];

  let frameIdx = $state(0);
  // Vendor: the page opens playing; paused, the controller shows the selected frame.
  let playing = $state(true);
  let copying = $state(false);
  let dragFrom = $state<number | null>(null);
  let confirmDefaults = $state(false);

  const light = $derived(app.light);
  // Same slot as the dashboard Lights card, so the editor opens on what the card showed.
  const anim = $derived(light ? light.animations[lightSel.slot] : null);
  // The device always sends MAX_FRAMES frames; only the first keyFrameCount are played.
  const count = $derived(anim ? Math.min(MAX_FRAMES, Math.max(1, anim.keyFrameCount)) : 0);
  const cur = $derived(Math.min(frameIdx, Math.max(0, count - 1)));
  const frame = $derived(anim ? anim.frames[cur] : null);
  const indices = $derived(Array.from({ length: count }, (_, f) => f));

  const activePreset = $derived(anim ? matchPreset(anim) : null);

  // ConfigIndex 4 is not editable on this page: switch to 0 before the first playback sync, so the save goes out first.
  if ((app.light?.configIndex ?? 0) > 3) selectSlot(0);

  // The controller mirrors the editor: playing, or paused on the selected frame (1-based on the wire, like the vendor).
  $effect(() => {
    if (!light) return;
    const on = playing;
    const f = cur + 1;
    // untrack: the call flushes pending saves, which read the whole light; edits must not re-send playback.
    untrack(() => void app.setPlayback(on, f));
  });

  // Leaving the page resumes the animation on the controller, as the vendor app does. C2 only:
  // V2 pads render EffectLights here and must not get the C2 `0f 0d` command.
  onMount(() => {
    if (!app.light) return;
    const id = app.openId;
    return () => {
      if (app.openId === id) void app.setPlayback(true, 1);
    };
  });

  function save() {
    app.saveLight();
  }

  // The chosen slot becomes the animation the controller plays (ConfigIndex).
  function selectSlot(i: number) {
    if (copying) return copyTo(i);
    lightSel.slot = i;
    frameIdx = 0;
    if (light && light.configIndex !== i) {
      light.configIndex = i;
      save();
    }
  }

  function copyTo(i: number) {
    copying = false;
    if (!light || !anim || i === lightSel.slot) return;
    light.animations[i] = $state.snapshot(anim);
    light.configIndex = i;
    lightSel.slot = i;
    frameIdx = 0;
    playing = false;
    save();
  }

  function usePreset(p: LightPreset) {
    if (!anim) return;
    applyPreset(anim, p);
    frameIdx = 0;
    save();
  }
  const strip = (p: LightPreset) => `linear-gradient(90deg, ${p.steps.flatMap((s) => [s.ll, s.home, s.lr]).join(", ")})`;

  function blankFrame(): Frame {
    return [[0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0]];
  }

  function addFrame() {
    if (!anim || count >= MAX_FRAMES) return;
    anim.frames[count] = anim.frames[count - 1].map((c) => [...c] as Rgb) as Frame;
    anim.keyFrameCount = count + 1;
    frameIdx = count;
    save();
  }

  function removeFrame() {
    if (!anim || count <= 1) return;
    anim.frames.splice(cur, 1);
    anim.frames.push(blankFrame());
    anim.keyFrameCount = count - 1;
    save();
  }

  // Vendor app: dropping one frame on another swaps the two.
  function swapFrames(a: number, b: number) {
    if (!anim || a === b) return;
    [anim.frames[a], anim.frames[b]] = [anim.frames[b], anim.frames[a]];
    frameIdx = b;
    save();
  }

  function setColor(led: number, hex: string) {
    if (!frame) return;
    frame[led] = parseRgb(hex);
    save();
  }

  // Vendor page reset: sleep 10 min, wake on raise, no audio mode, full brightness, Flow on this slot.
  function restoreDefaults() {
    if (!confirmDefaults) {
      confirmDefaults = true;
      setTimeout(() => (confirmDefaults = false), 4000);
      return;
    }
    confirmDefaults = false;
    if (!light || !anim) return;
    light.standbyTime = 10;
    light.raiseWakeUp = true;
    light.audioReactive = false;
    anim.brightness = 100;
    applyPreset(anim, LIGHT_PRESETS[0]);
    frameIdx = 0;
    save();
  }
</script>

<svelte:window onkeydown={(e) => copying && e.key === "Escape" && (copying = false)} />

{#if app.effectLight}
  <EffectLights />
{:else if light && anim && frame}
  <div class="lights">
    <div class="slots">
      <nav class="seg tabs" aria-label={tr("lightConfig")}>
        {#each SLOTS as i (i)}
          <button
            class:active={lightSel.slot === i}
            aria-pressed={lightSel.slot === i}
            disabled={copying && lightSel.slot === i}
            onclick={() => selectSlot(i)}>{tr("lightConfig")} {i + 1}</button
          >
        {/each}
      </nav>
      <button class:active={copying} aria-pressed={copying} onclick={() => (copying = !copying)}>{tr("copyTo")}</button>
      <button class="primary play" aria-pressed={playing} onclick={() => (playing = !playing)}>
        {playing ? tr("pause") : tr("play")}
      </button>
      {#if copying}<p class="hint" role="status">{tr("copyPick")}</p>{/if}
    </div>

    <section class="card presets">
      <h2 class="section-label">{tr("presets")}</h2>
      <div class="chips">
        {#each LIGHT_PRESETS as p (p.id)}
          <button class:active={activePreset === p.id} aria-pressed={activePreset === p.id} onclick={() => usePreset(p)}>
            <span class="strip" style:background={strip(p)} aria-hidden="true"></span>
            {tr(p.name)}
          </button>
        {/each}
      </div>
      <p class="hint">{tr("presetsHint")}</p>
    </section>

    <section class="card">
      <Range label={tr("brightness")} unit="%" bind:value={anim.brightness} max={100} onchange={save} />
      <Range label={tr("speed")} bind:value={anim.speed} max={20} onchange={save} />

      <div class="field-row">
        <span>{tr("frames")}</span>
        <div class="ctl">
          <div class="seg">
            {#each indices as f (f)}
              <button
                class:active={cur === f}
                class:drop={dragFrom !== null && dragFrom !== f}
                aria-pressed={cur === f}
                draggable="true"
                ondragstart={() => (dragFrom = f)}
                ondragend={() => (dragFrom = null)}
                ondragover={(e) => e.preventDefault()}
                ondrop={(e) => {
                  e.preventDefault();
                  if (dragFrom !== null) swapFrames(dragFrom, f);
                  dragFrom = null;
                }}
                onclick={() => (frameIdx = f)}>{f + 1}</button
              >
            {/each}
          </div>
          <button onclick={addFrame} disabled={count >= MAX_FRAMES}>{tr("addFrame")}</button>
          <button onclick={removeFrame} disabled={count <= 1}>{tr("removeFrame")}</button>
        </div>
      </div>
      <p class="hint">{tr("framesHint")}</p>

      {#each LEDS as [led, key], n (led)}
        <label class="field-row">
          <span>{n + 1} · {tr(key as "slotHome")}</span>
          <span class="swatch">
            <span class="value">{hexRgb(frame[led])}</span>
            <input type="color" value={hexRgb(frame[led])} onchange={(e) => setColor(led, e.currentTarget.value)} />
          </span>
        </label>
      {/each}
    </section>

    <section class="card">
      <label class="field-row">
        <span>{tr("profileLed")}</span>
        <span class="swatch">
          <span class="value">{hexRgb(light.profileLed)}</span>
          <input
            type="color"
            value={hexRgb(light.profileLed)}
            onchange={(e) => {
              light.profileLed = parseRgb(e.currentTarget.value);
              save();
            }}
          />
        </span>
      </label>
      <label class="field-row">
        <span>{tr("audioReactive")}</span>
        <input type="checkbox" bind:checked={light.audioReactive} onchange={save} />
      </label>
      <label class="field-row">
        <span>{tr("wakeUp")}</span>
        <input type="checkbox" bind:checked={light.raiseWakeUp} onchange={save} />
      </label>
      <label class="field-row">
        <span>{tr("standby")}</span>
        <select bind:value={light.standbyTime} onchange={save}>
          {#if !STANDBY.includes(light.standbyTime)}<option value={light.standbyTime}>{tr("minutes", { n: light.standbyTime })}</option>{/if}
          {#each STANDBY as m (m)}
            <option value={m}>{m === 0 ? tr("standbyOff") : tr("minutes", { n: m })}</option>
          {/each}
        </select>
      </label>
      <button class="defaults" class:danger={confirmDefaults} onclick={restoreDefaults}>
        {confirmDefaults ? tr("lightDefaultsConfirm") : tr("lightDefaults")}
      </button>
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
  .slots {
    grid-column: 1 / -1;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
  }
  .slots .hint { flex-basis: 100%; }
  .seg button.drop { outline: 1px dashed var(--line-strong); outline-offset: -3px; }
  .seg button[draggable="true"] { cursor: grab; }
  /* Play/Pause sits at the end of the slot bar, like the vendor's play button over the key frames. */
  .play { margin-left: auto; min-width: 120px; }
  .defaults { justify-self: start; margin-top: 8px; }
  section {
    display: grid;
    gap: 4px;
    align-content: start;
  }
  .field-row + .field-row {
    border-top: 1px solid var(--line);
  }
  .ctl {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    min-width: 0;
  }
  .swatch {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .field-row > select { max-width: 160px; }
  .presets {
    grid-column: 1 / -1;
  }
  .chips {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(112px, 1fr));
    gap: 8px;
    margin: 4px 0 8px;
  }
  .chips button {
    display: grid;
    gap: 6px;
    justify-items: start;
    padding: 8px 10px;
    min-height: 52px;
    text-align: left;
  }
  .strip {
    width: 100%;
    height: 6px;
    border-radius: 3px;
  }
  @media (max-width: 959px) {
    .lights {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
