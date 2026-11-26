<script lang="ts">
  import { tr } from "$lib/i18n";

  let {
    low = $bindable(),
    high = $bindable(),
    min = 0,
    max = 100,
    label,
    unit,
    onchange,
  }: {
    low: number;
    high: number;
    min?: number;
    max?: number;
    label: string;
    /** Unit shown after the number fields, e.g. "%". */
    unit?: string;
    onchange?: () => void;
  } = $props();

  const lowLabel = $derived(`${label}: ${tr("begin")}`);
  const highLabel = $derived(`${label}: ${tr("end")}`);
  const TICKS = Array.from({ length: 11 }, (_, i) => i * 10);
  // Thumbs have a 24px hit area and centre 12px in from each end, so ticks and fill line up with them.
  const at = (t: number) => `calc(12px + (100% - 24px) * ${t / 100})`;
  const pct = (v: number) => ((v - min) / (max - min)) * 100;
  // The thumb nearer the top end is stacked above, so it can still move back when both sit at max (and vice versa at min).
  const lowOnTop = $derived(low + high >= min + max);

  // The two thumbs can never cross: each clamps against the other and the DOM value follows the clamp.
  function setLow(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    low = Math.min(Number(input.value), high);
    input.value = String(low);
  }

  function setHigh(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    high = Math.max(Number(input.value), low);
    input.value = String(high);
  }

  // Typed values follow the same rule: clamp to range, then keep low <= high.
  // Empty or invalid input restores the current value and saves nothing (Number("") would be 0).
  function enter(e: Event, which: "low" | "high") {
    const el = e.currentTarget as HTMLInputElement;
    const n = Math.round(el.valueAsNumber);
    if (Number.isFinite(n)) {
      const v = Math.min(max, Math.max(min, n));
      if (which === "low") low = Math.min(v, high);
      else high = Math.max(v, low);
    }
    el.value = String(which === "low" ? low : high);
    if (Number.isFinite(n)) onchange?.();
  }
</script>

<div class="dual" style:--lo={pct(low) / 100} style:--hi={pct(high) / 100}>
  <div class="head">
    <span class="name">{label}</span>
    <span class="readout">
      <input class="num" type="number" {min} {max} step="1" value={low} aria-label={`${lowLabel}: ${tr("exactValue")}`} onchange={(e) => enter(e, "low")} />
      <span class="sep">–</span>
      <input class="num" type="number" {min} {max} step="1" value={high} aria-label={`${highLabel}: ${tr("exactValue")}`} onchange={(e) => enter(e, "high")} />
      {#if unit}<span class="unit">{unit}</span>{/if}
    </span>
  </div>
  <div class="slot">
    <span class="rail"></span>
    <span class="fill"></span>
    <input
      class="lo"
      class:top={lowOnTop}
      type="range"
      aria-label={lowLabel}
      aria-valuetext={`${low}${unit ?? ""}`}
      {min}
      {max}
      step="1"
      value={low}
      oninput={setLow}
      onchange={() => onchange?.()}
    />
    <input
      class="hi"
      class:top={!lowOnTop}
      type="range"
      aria-label={highLabel}
      aria-valuetext={`${high}${unit ?? ""}`}
      {min}
      {max}
      step="1"
      value={high}
      oninput={setHigh}
      onchange={() => onchange?.()}
    />
  </div>
  <span class="scale" aria-hidden="true">
    {#each TICKS as t (t)}
      <i class:major={t % 50 === 0} style:left={at(t)}></i>
    {/each}
  </span>
</div>

<style>
  .dual {
    display: grid;
    gap: 4px;
    min-width: 0;
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    font-size: 13px;
    color: var(--fg-2);
    min-width: 0;
  }
  .name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .readout {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    flex: none;
  }
  .num {
    width: 4.5em;
    height: 28px;
    padding: 0 6px;
    border: 1px solid var(--line);
    border-radius: 4px;
    background: transparent;
    color: var(--fg);
    font: inherit;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .sep,
  .unit {
    color: var(--fg-3);
  }
  .slot {
    position: relative;
    height: 24px;
  }
  .rail,
  .fill {
    position: absolute;
    top: 50%;
    height: 4px;
    margin-top: -2px;
    border-radius: 2px;
  }
  .rail {
    left: 12px;
    right: 12px;
    background: var(--track);
  }
  .fill {
    left: calc(12px + (100% - 24px) * var(--lo));
    right: calc(12px + (100% - 24px) * (1 - var(--hi)));
    background: var(--fill);
  }
  input[type="range"] {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 24px;
    margin: 0;
    background: none;
    pointer-events: none;
    appearance: none;
  }
  input[type="range"]::-webkit-slider-runnable-track {
    background: transparent;
  }
  input[type="range"]::-moz-range-track,
  input[type="range"]::-moz-range-progress {
    background: transparent;
  }
  /* 24px hit area; the visible dot stays 14px. Separate rules: in one selector list the engine's unknown pseudo drops the whole rule,
     leaving both thumbs with pointer-events: none (no mouse/touch drag). */
  input[type="range"]::-webkit-slider-thumb {
    pointer-events: auto;
    appearance: none;
    box-sizing: border-box;
    width: 24px;
    height: 24px;
    margin-top: -10px;
    padding: 5px;
    border: 0;
    border-radius: 50%;
    box-shadow: none;
    background: var(--fg);
    background-clip: content-box;
  }
  input[type="range"]::-moz-range-thumb {
    pointer-events: auto;
    box-sizing: border-box;
    width: 24px;
    height: 24px;
    padding: 7px;
    border: 0;
    border-radius: 50%;
    background: var(--fg);
    background-clip: content-box;
  }
  .top {
    z-index: 1;
  }
  .scale {
    position: relative;
    display: block;
    height: 6px;
  }
  .scale i {
    position: absolute;
    top: 0;
    width: 1px;
    height: 4px;
    background: var(--line-strong);
  }
  .scale i.major {
    height: 6px;
  }
</style>
