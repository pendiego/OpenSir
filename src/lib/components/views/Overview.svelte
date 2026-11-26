<script lang="ts">
  import { app, type View } from "$lib/app.svelte";
  import { tr, type Key } from "$lib/i18n";
  import Controller from "$lib/components/Controller.svelte";
  import { lightSel } from "$lib/lightSel.svelte";
  import { KEYS, marked } from "$lib/keys";
  import { hairMode } from "$lib/bitfields";
  import type { Rgb } from "$lib/model";

  // Stored in fun.reportRateGears: 0=250 Hz, 1=500 Hz, 2=1000 Hz.
  const GEARS: { value: number; key: Key }[] = [
    { value: 0, key: "rate0" },
    { value: 1, key: "rate1" },
    { value: 2, key: "rate2" },
  ];
  const HAIR: Key[] = ["hairOff", "hairRapid", "hairFix"];
  const BEHAVIOR: Key[] = ["behOff", "behButton", "behToggle", "behAlways"];

  const p = $derived(app.profile);
  const leds = $derived(app.light?.animations[lightSel.slot]?.frames[0] as Rgb[] | undefined);
  const anim = $derived(app.light?.animations[lightSel.slot] ?? null);
  const battery = $derived(app.snapshot?.battery ?? null);
  const remapped = $derived(p ? KEYS.filter((k) => marked(p, k.target)).length : 0);

  function go(view: View) {
    app.view = view;
  }

  function setGear(v: number) {
    if (!p) return;
    p.fun.reportRateGears = v;
    app.saveProfile();
  }

  function toggleShift(on: boolean) {
    if (!p) return;
    p.fun.shiftEnabled = on ? 1 : 0;
    app.saveProfile();
  }
</script>

<div class="overview">
  <section class="bench panel">
    <button class="art" aria-label={tr("mapping")} title={tr("mapping")} onclick={() => go("mapping")}>
      <Controller {leds} model={app.model} />
    </button>
    <div class="caption">
      <strong>{app.model.name}</strong>
      {#if app.snapshot?.firmware}
        <span class="mono muted">{tr("firmware")} {app.snapshot.firmware}</span>
      {/if}
      {#if battery !== null}
        <span class="mono muted" class:low={battery < 20}>
          {tr("battery")} {battery}%{app.snapshot?.charging ? ` · ${tr("charging")}` : ""}
        </span>
      {/if}
    </div>
  </section>

  {#if p}
    <section class="summary card">
      <header class="head">
        <strong>{p.name && p.name !== `Profile ${app.viewIndex}` ? p.name : `${tr("profile")} ${app.viewIndex}`}</strong>
      </header>
      <div class="row plain">
        <span class="k">{tr("dashReportRate")}</span>
        <div class="seg" role="radiogroup" aria-label={tr("dashReportRate")} title={tr("hintReportRate")}>
          {#each GEARS as g (g.value)}
            <button role="radio" aria-checked={p.fun.reportRateGears === g.value} class:active={p.fun.reportRateGears === g.value} onclick={() => setGear(g.value)}>{tr(g.key)}</button>
          {/each}
        </div>
      </div>

      <button class="row nav" onclick={() => go("sticks")}>
        <span class="k">{tr("dashSticks")}</span>
        <span class="v mono">{tr("shortLeft")} {p.leftStick.dead.front}–{p.leftStick.dead.back} · {tr("shortRight")} {p.rightStick.dead.front}–{p.rightStick.dead.back}</span>
        <svg class="chev" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg>
      </button>

      <button class="row nav" onclick={() => go("triggers")}>
        <span class="k">{tr("dashTriggers")}</span>
        <span class="v mono">{tr(HAIR[hairMode(p.l2.quickTriggerStatus)])} · {tr(HAIR[hairMode(p.r2.quickTriggerStatus)])}</span>
        <svg class="chev" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg>
      </button>

      <button class="row nav" onclick={() => go("motion")}>
        <span class="k">{tr("dashMotion")}</span>
        <span class="v mono">{tr(BEHAVIOR[p.aim.sensorProfileStatus] ?? "behOff")}</span>
        <svg class="chev" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg>
      </button>

      <button class="row nav" onclick={() => go("mapping")}>
        <span class="k">{tr("dashButtons")}</span>
        <span class="v mono">{tr("remappedCount", { n: remapped })}</span>
        <svg class="chev" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg>
      </button>

      <button class="row nav" onclick={() => go("motors")}>
        <span class="k">{tr("motors")}</span>
        <span class="v mono">{tr("shortLeft")} {p.fun.motors[0]} · {tr("shortRight")} {p.fun.motors[1]}</span>
        <svg class="chev" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg>
      </button>

      <button class="row nav" onclick={() => go("lights")}>
        <span class="k">{tr("dashLights")}</span>
        <span class="v mono">{tr("lightConfig")} {lightSel.slot + 1}{anim ? ` · ${anim.brightness}%` : ""}</span>
        <svg class="chev" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg>
      </button>

      <label class="row plain">
        <span class="k">{tr("dashShift")}</span>
        <input class="switch" type="checkbox" role="switch" checked={p.fun.shiftEnabled === 1} onchange={(e) => toggleShift(e.currentTarget.checked)} aria-label={tr("dashShift")} title={tr("hintShift")} />
      </label>
    </section>
  {/if}
</div>

<style>
  .overview {
    display: grid;
    grid-template-columns: minmax(0, 1.6fr) minmax(0, 1fr);
    gap: 16px;
    align-items: start;
  }
  @media (max-width: 959px) {
    .overview { grid-template-columns: minmax(0, 1fr); }
  }

  .bench {
    display: grid;
    gap: 16px;
    padding: 24px;
    justify-items: center;
  }
  .art {
    background: none;
    border: 0;
    padding: 0;
    width: 100%;
    display: grid;
    place-items: center;
  }
  .art:hover:not(:disabled) { background: none; }
  .art, .row.nav { box-shadow: none; }
  .art :global(svg) { width: min(100%, 520px); height: auto; }
  /* Same name as the list card's art: opening a controller morphs one into the other. */
  .art :global(svg.pad) { view-transition-name: pad-art; }
  .caption {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: center;
    gap: 4px 16px;
  }
  .caption strong { font-size: 14px; font-weight: 500; }
  .mono { font-family: var(--mono); font-size: 12px; font-variant-numeric: tabular-nums; }
  .muted { color: var(--fg-2); }
  .low { color: var(--warn); }

  .summary { padding: 0 0 4px; }
  .head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    padding: 16px 16px 12px;
  }
  .head strong { font-size: 15px; font-weight: 500; }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: nowrap;
    min-height: 44px;
    padding: 0 16px;
    border-top: 1px solid var(--line);
    width: 100%;
    text-align: left;
    font-size: 14px;
  }
  .summary > :first-child { border-top: 0; }
  .row .k { flex: 0 1 auto; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--fg-2); }
  .row .v { margin-left: auto; min-width: 0; white-space: nowrap; text-align: right; color: var(--fg); }
  .row.nav { background: transparent; border-left: 0; border-right: 0; border-bottom: 0; border-radius: 0; }
  .row.nav .chev { flex: 0 0 auto; color: var(--fg-3); }
  .row.nav .v { margin-left: auto; }
  /* Wraps the seg under its label in a narrow column (PT label is long) rather than truncating it. */
  .row.plain { flex-wrap: wrap; row-gap: 6px; padding-block: 6px; }
  .row.plain .k { overflow: visible; }
  .row.plain .seg { margin-left: auto; min-width: 0; }
  .row.plain .seg button { padding: 0 8px; white-space: nowrap; }
  @media (hover: hover) and (pointer: fine) {
    .row.nav:hover:not(:disabled) { background: var(--raised); }
  }

  .switch {
    appearance: none;
    margin: 0 0 0 auto;
    width: 32px;
    height: 18px;
    border-radius: 9px;
    background: var(--track);
    position: relative;
    cursor: pointer;
    transition: background 160ms var(--ease);
  }
  .switch::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--fg-2);
    transition: transform 160ms var(--ease);
  }
  .switch:checked { background: var(--fill); }
  .switch:checked::after { transform: translateX(14px); background: var(--bg); }
</style>
