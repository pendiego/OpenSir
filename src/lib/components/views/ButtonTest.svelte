<script lang="ts">
  import { app } from "$lib/app.svelte";
  import Controller from "$lib/components/Controller.svelte";
  import { KEYS } from "$lib/keys";
  import { tr } from "$lib/i18n";

  // Test mode needs an open session. Coming from the device list there is none, and the page
  // used to sit on "No input" forever: open the first configurable controller here.
  $effect(() => {
    const id = app.openId;
    if (!id) {
      const d = app.devices.find((d) => d.configCapable);
      if (d) void app.open(d.id);
      return;
    }
    app.gamepad = null;
    app.setTestMode(true);
    return () => app.setTestMode(false);
  });

  const pct = (v: number) => Math.round(v * 100);
  const trig = (v: number) => Math.round((v / 255) * 100);
  // Live names are device key ids ("CRO"); show the drawing's labels ("A") where there is one.
  const label = (id: string) => KEYS.find((k) => k.id === id)?.label ?? id;

  const g = $derived(app.gamepad);
  const sticks = $derived(
    g
      ? [
          { name: "L", x: g.lx, y: g.ly },
          { name: "R", x: g.rx, y: g.ry },
        ]
      : [],
  );
  const triggers = $derived(
    g
      ? [
          { name: "LT", v: g.lt },
          { name: "RT", v: g.rt },
        ]
      : [],
  );
</script>

<div class="test">
  <section class="bench">
    <Controller live={g} model={app.model} />
  </section>

  {#if !app.openId && !app.devices.some((d) => d.configCapable)}
    <p class="msg">{tr(app.devices.length ? "notConfigCapable" : "noDevices")}</p>
    {#if !app.devices.length}<p class="msg">{tr("noDevicesHint")}</p>{/if}
  {:else if g}
    <section class="readout" aria-label={tr("live")}>
      {#each sticks as s (s.name)}
        <div class="card stick">
          <!-- Device y grows downward, like SVG y: no flip for the dot; the readout shows up as positive. -->
          <svg viewBox="-50 -50 100 100" aria-hidden="true">
            <circle r="46" class="ring" />
            <path d="M-46 0H46M0 -46V46" class="axis" />
            <circle cx={s.x * 42} cy={s.y * 42} r="5" class="dot" />
          </svg>
          <div class="vals">
            <span class="name">{s.name}</span>
            <span class="value">x {pct(s.x)}</span>
            <span class="value">y {pct(-s.y)}</span>
          </div>
        </div>
      {/each}

      <div class="card trig">
        {#each triggers as t (t.name)}
          <div class="tr-row">
            <span class="name">{t.name}</span>
            <span class="track"><span class="fill" style:width="{trig(t.v)}%"></span></span>
            <span class="value">{trig(t.v)}%</span>
          </div>
        {/each}
      </div>

      <div class="card keys">
        <span class="name">{tr("pressed")}</span>
        <div class="caps">
          {#each g.buttons as b (b)}
            <kbd>{label(b)}</kbd>
          {:else}
            <span class="none">{tr("nonePressed")}</span>
          {/each}
        </div>
      </div>
    </section>
    <p class="msg hint">{tr("stickHint")}</p>
  {:else if app.error}
    <p class="msg error" role="alert">{app.error}</p>
  {:else}
    <p class="msg">{tr("noSignal")}</p>
  {/if}
</div>

<style>
  .test {
    display: grid;
    align-content: start;
    gap: 16px;
  }
  .bench {
    display: grid;
    place-items: center;
    padding: 24px;
    background: var(--sidebar);
    border: 1px solid var(--line);
    border-radius: var(--r-panel);
  }
  .bench :global(svg.pad) {
    width: min(100%, 560px);
    height: auto;
  }
  .readout {
    display: grid;
    grid-template-columns: auto auto minmax(0, 1fr) minmax(0, 1fr);
    gap: 16px;
  }
  @media (max-width: 1099px) {
    .readout { grid-template-columns: auto auto minmax(0, 1fr); }
    .keys { grid-column: 1 / -1; }
  }
  .card { padding: 16px; }
  .stick {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .stick svg { width: 72px; height: 72px; }
  .ring { fill: var(--sidebar); stroke: var(--line-strong); stroke-width: 1; }
  .axis { stroke: var(--line); stroke-width: 1; }
  .dot { fill: var(--fg); }
  .vals { display: grid; gap: 2px; }
  /* Room for the widest reading ("y -100"): the stick cards must not resize as the stick moves. */
  .vals .value { min-width: 6ch; }
  .name { font-size: 13px; color: var(--fg-2); }
  .value {
    font: 13px var(--mono);
    font-variant-numeric: tabular-nums;
    color: var(--fg);
  }
  .trig { display: grid; align-content: center; gap: 12px; }
  .tr-row {
    display: grid;
    grid-template-columns: 24px minmax(0, 1fr) 40px;
    align-items: center;
    gap: 12px;
  }
  .tr-row .value { text-align: right; }
  .track {
    height: 6px;
    border-radius: 3px;
    background: var(--track);
    overflow: hidden;
  }
  .fill {
    display: block;
    height: 100%;
    background: var(--fill);
  }
  .keys { display: grid; align-content: start; gap: 8px; }
  .caps { display: flex; flex-wrap: wrap; gap: 4px; min-height: 24px; }
  kbd {
    padding: 2px 8px;
    border: 1px solid color-mix(in srgb, var(--accent) 55%, transparent);
    border-radius: 6px;
    background: var(--accent-soft);
    font: 12px var(--mono);
    color: var(--accent-text);
  }
  .none { font-size: 13px; color: var(--fg-3); }
  .msg { margin: 0; text-align: center; color: var(--fg-2); }
  .msg.error { color: var(--danger); }
  .hint { margin: 0; font-size: 13px; color: var(--fg-2); text-align: left; }
</style>
