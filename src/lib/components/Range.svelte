<script lang="ts">
  import { tr } from "$lib/i18n";

  let {
    label,
    value = $bindable(),
    min = 0,
    max,
    step = 1,
    unit,
    format,
    onchange,
  }: {
    label: string;
    value: number;
    min?: number;
    max: number;
    step?: number;
    /** Unit shown after the number field, e.g. "%". */
    unit?: string;
    /** Spoken value text; defaults to the number plus unit. */
    format?: (v: number) => string;
    onchange?: () => void;
  } = $props();

  const uid = $props.id();

  // Short ranges (motor levels 0-4) get a tick per step; long ones a tick every 10%.
  const ticks = $derived.by(() => {
    const steps = (max - min) / step;
    const n = steps <= 20 ? steps : 10;
    return Array.from({ length: n + 1 }, (_, i) => ({ t: (i / n) * 100, major: i === 0 || i === n || i * 2 === n }));
  });
  // Thumbs have a 24px hit area and centre 12px in from each end, so ticks sit under the thumb path.
  const at = (t: number) => `calc(12px + (100% - 24px) * ${t / 100})`;

  // Typed value: snap to step, clamp to range, then write the accepted value back to the field.
  // Empty or invalid input restores the current value and saves nothing (Number("") would be 0).
  function commit(e: Event) {
    const el = e.currentTarget as HTMLInputElement;
    const n = el.valueAsNumber;
    if (Number.isFinite(n)) value = Math.min(max, Math.max(min, min + Math.round((n - min) / step) * step));
    el.value = String(value);
    if (Number.isFinite(n)) onchange?.();
  }
</script>

<div class="rng">
  <div class="head">
    <span class="name" id={uid}>{label}</span>
    <span class="readout">
      <input class="num" type="number" {min} {max} {step} {value} aria-label={`${label}: ${tr("exactValue")}`} onchange={commit} />
      {#if unit}<span class="unit">{unit}</span>{/if}
    </span>
  </div>
  <input
    type="range"
    {min}
    {max}
    {step}
    bind:value
    aria-labelledby={uid}
    aria-valuetext={format ? format(value) : `${value}${unit ?? ""}`}
    style:--p="{((value - min) / (max - min)) * 100}%"
    onchange={() => onchange?.()}
  />
  <span class="scale" aria-hidden="true">
    {#each ticks as { t, major } (t)}
      <i class:major style:left={at(t)}></i>
    {/each}
  </span>
</div>

<style>
  .rng {
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
    width: 5em;
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
  .unit {
    color: var(--fg-3);
  }
  input[type="range"] {
    width: 100%;
    height: 24px;
  }
  /* 24px hit area; the visible dot stays about 14px. */
  input[type="range"]::-webkit-slider-thumb {
    box-sizing: border-box;
    width: 24px;
    height: 24px;
    margin-top: -10px;
    padding: 5px;
    border: 0;
    box-shadow: none;
    background: var(--fg);
    background-clip: content-box;
  }
  input[type="range"]::-moz-range-thumb {
    box-sizing: border-box;
    width: 24px;
    height: 24px;
    padding: 7px;
    border: 0;
    background: var(--fg);
    background-clip: content-box;
  }
  .scale {
    position: relative;
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
