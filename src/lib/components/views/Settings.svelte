<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { tr } from "$lib/i18n";
  import type { Fun } from "$lib/model";

  type Flag = "dpadDiagonalLock" | "xinputAbxyChange" | "switchAbxyChange" | "shiftEnabled" | "sensorQuickMode";

  const p = $derived(app.profile);
  let draft = $state<string | null>(null);
  const enc = new TextEncoder();
  const nameBytes = $derived(enc.encode(draft ?? p?.name ?? "").length);

  type Byte = "stickResolution" | "lsAntiJitter" | "rsAntiJitter";

  /** u8 fields: clamp the typed value and show the stored one back. Empty/invalid keeps the old value. */
  function setByte(k: Byte, input: HTMLInputElement) {
    if (!p) return;
    const n = Math.round(input.valueAsNumber);
    if (Number.isFinite(n)) {
      p.fun[k] = Math.min(255, Math.max(0, n));
      app.saveProfile();
    }
    input.value = String(p.fun[k]);
  }

  function setFlag(k: Flag, on: boolean) {
    if (!p) return;
    p.fun[k] = on ? 1 : 0;
    app.saveProfile();
  }

  function setRate(v: number) {
    if (!p) return;
    p.fun.reportRateGears = v;
    app.saveProfile();
  }

  function setShiftValue(input: HTMLInputElement) {
    if (!p) return;
    // u8 on the wire: clamp the typed value and show the stored one back. Empty/invalid keeps the old value.
    const n = Math.round(input.valueAsNumber);
    if (Number.isFinite(n)) {
      p.fun.shiftValue = Math.min(255, Math.max(0, n));
      app.saveProfile();
    }
    input.value = String(p.fun.shiftValue);
  }

  const RATES: [number, string][] = [
    [0, "rate0"],
    [1, "rate1"],
    [2, "rate2"],
  ];
</script>

{#if p && app.snapshot}
  <section class="card settings">
    <label class="field-row">
      <span>{tr("name")}</span>
      <input
        class="name"
        type="text"
        value={p.name}
        maxlength="32"
        oninput={(e) => (draft = e.currentTarget.value)}
        onchange={(e) => { app.setName(e.currentTarget.value); draft = null; }}
      />
    </label>
    <p class="muted note" aria-live="polite">{nameBytes} / 32 {tr("bytesUsed")}</p>

    <div class="field-row">
      <span>{tr("reportRate")}</span>
      <div class="seg">
        {#each RATES as [v, key] (v)}
          <button
            class:active={p.fun.reportRateGears === v}
            aria-pressed={p.fun.reportRateGears === v}
            onclick={() => setRate(v)}>{tr(key as "rate0")}</button
          >
        {/each}
      </div>
    </div>
    <p class="hint note">{tr("hintReportRate")}</p>

    <label class="field-row">
      <span>{tr("dpadLock")}</span>
      <input
        type="checkbox"
        checked={p.fun.dpadDiagonalLock === 1}
        onchange={(e) => setFlag("dpadDiagonalLock", e.currentTarget.checked)}
      />
    </label>
    <p class="hint note">{tr("hintDpadLock")}</p>
    <label class="field-row">
      <span>{tr("abxyXinput")}</span>
      <input
        type="checkbox"
        checked={p.fun.xinputAbxyChange === 1}
        onchange={(e) => setFlag("xinputAbxyChange", e.currentTarget.checked)}
      />
    </label>
    <p class="hint note">{tr("hintAbxyXinput")}</p>
    <label class="field-row">
      <span>{tr("abxySwitch")}</span>
      <input
        type="checkbox"
        checked={p.fun.switchAbxyChange === 1}
        onchange={(e) => setFlag("switchAbxyChange", e.currentTarget.checked)}
      />
    </label>
    <p class="hint note">{tr("hintAbxySwitch")}</p>
    {#if app.model.family === "V2"}
      <label class="field-row">
        <span>{tr("quickMode")}</span>
        <input type="checkbox" checked={p.fun.sensorQuickMode === 1} onchange={(e) => setFlag("sensorQuickMode", e.currentTarget.checked)} />
      </label>
      {#each (app.model.id === "T3CE" ? ["stickResolution"] : ["stickResolution", "lsAntiJitter", "rsAntiJitter"]) as Byte[] as k (k)}
        <label class="field-row">
          <span>{tr(k === "lsAntiJitter" ? "antiJitterLeft" : k === "rsAntiJitter" ? "antiJitterRight" : "stickResolution")}</span>
          <input class="num" type="number" min="0" max="255" value={p.fun[k]} onchange={(e) => setByte(k, e.currentTarget)} />
        </label>
      {/each}
    {/if}
    <label class="field-row">
      <span>{tr("shiftEnabled")}</span>
      <input
        type="checkbox"
        checked={p.fun.shiftEnabled === 1}
        onchange={(e) => setFlag("shiftEnabled", e.currentTarget.checked)}
      />
    </label>
    <fieldset class="dep" disabled={p.fun.shiftEnabled !== 1}>
      <label class="field-row">
        <span>{tr("shiftValue")}</span>
        <input
          class="num"
          type="number"
          min="0"
          max="255"
          value={p.fun.shiftValue}
          aria-describedby="shift-value-hint"
          onchange={(e) => setShiftValue(e.currentTarget)}
        />
      </label>
      <p class="hint note" id="shift-value-hint">{tr("hintShift")}</p>
    </fieldset>
  </section>
{/if}

<style>
  .settings {
    display: grid;
    max-width: 640px;
  }
  .name {
    width: 240px;
    max-width: 100%;
  }
  .num {
    width: 96px;
  }
  .note {
    margin: 0;
    padding-bottom: 8px;
    font-size: 12px;
  }
  .field-row + .dep,
  .note + .field-row {
    border-top: 1px solid var(--line);
  }
</style>
