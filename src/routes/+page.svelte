<script lang="ts">
  import { tick } from "svelte";
  import { app } from "$lib/app.svelte";
  import { isTauri } from "$lib/api";
  import DeviceCarousel from "$lib/components/DeviceCarousel.svelte";
  import DeviceView from "$lib/components/DeviceView.svelte";
  import PadThumb from "$lib/components/PadThumb.svelte";
  import { MODELS, deviceStatus, isWireless, modelOf, parkedDevices, pidHex, visibleDevices, type Support } from "$lib/models";
  import { SIMULATED, simulatePlug, simulateUnplug } from "$lib/mock";
  import { tr, type Key } from "$lib/i18n";
  import type { DeviceInfo } from "$lib/model";

  const SUPPORT: Record<Support, Key> = { full: "supportFull", codec: "supportCodec", other: "supportOther" };

  const all = $derived(visibleDevices(app.devices));
  // Game-mode C2 entries beside a ready C2 go in a collapsed section: usually the same pad.
  const parked = $derived(parkedDevices(all));
  const list = $derived(all.filter((d) => !parked.includes(d)));
  let simKey = $state(SIMULATED[0].key);
  let refreshing = $state(false);
  let opening = $state<string | null>(null);
  let refreshNote = $state("");
  // Front card of the carousel; kept here because the carousel remounts after a device is closed.
  let picked = $state<string | null>(null);
  let heading = $state<HTMLHeadingElement>();

  async function refresh() {
    refreshing = true;
    refreshNote = "";
    await app.refreshDevices();
    refreshing = false;
    if (!app.error) refreshNote = tr("refreshed", { n: visibleDevices(app.devices).length });
  }

  async function open(id: string) {
    picked = id;
    opening = id;
    await app.open(id);
    opening = null;
  }

  // Coming back from a device (Back, or it was unplugged): focus its card's Open button, or the
  // heading when that card is gone, instead of leaving focus on the removed device view.
  let wasOpen = false;
  // After a device was open, the list comes back by the art morph, not the card entrance.
  let cameBack = $state(false);
  $effect(() => {
    const isOpen = !!(app.openId && app.snapshot);
    if (isOpen) cameBack = true;
    if (wasOpen && !isOpen) {
      void tick().then(() => {
        const card = list.some((d) => d.id === picked) && document.querySelector<HTMLElement>(".slide.front .primary");
        (card || heading)?.focus();
      });
    }
    wasOpen = isOpen;
  });
</script>

{#if app.openId && app.snapshot}
  <DeviceView />
{:else}
  <div class="picker">
    <header class="head">
      <div>
        <h1 bind:this={heading} tabindex="-1">{tr("devices")}</h1>
        <p class="lede">{tr("devicesHint")}</p>
        <p class="note" aria-live="polite">{refreshNote}</p>
      </div>
      <button onclick={refresh} disabled={refreshing}>
        <svg class:spin={refreshing} viewBox="0 0 24 24" aria-hidden="true"><path d="M20 12a8 8 0 1 1-2.3-5.7M20 4v5h-5" /></svg>
        {tr("refresh")}
      </button>
    </header>

    {#if app.error}
      <div class="notice" role="alert">
        <span>{app.error}</span>
        <button onclick={() => (app.error = "")}>{tr("dismiss")}</button>
      </div>
    {/if}

    {#if !isTauri}
      <section class="sim" aria-label={tr("simTitle")}>
        <div class="sim-text">
          <strong>{tr("simTitle")}</strong>
          <span>{tr("simHint")}</span>
        </div>
        <select bind:value={simKey} aria-label={tr("simTitle")}>
          {#each SIMULATED as s (s.key)}
            <option value={s.key}>{s.label}</option>
          {/each}
        </select>
        <button onclick={() => simulatePlug(simKey)}>{tr("simPlug")}</button>
      </section>
    {/if}

    {#snippet card(d: DeviceInfo)}
      {@const m = modelOf(d)}
      {@const s = deviceStatus(d)}
      <li class="dev" class:ready={d.configCapable}>
        <div class="art"><PadThumb model={m} ready={d.configCapable} /></div>
        <div class="info">
          <span class="name">{d.name}</span>
          <span class="conn">
            {#if isWireless(d)}
              <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12.5a10 10 0 0 1 14 0M8.5 16a5 5 0 0 1 7 0M12 19.5h.01" /></svg>
              {tr("connWireless")}
            {:else}
              <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3v12M9 6l3-3 3 3M8 11v4h8v-4M12 15v6" /></svg>
              {tr("connWired")}
            {/if}
            <span class="mono">{pidHex(d.pid)}</span>
          </span>
          <span class="status {s.tone}"><i></i>{tr(s.key)}</span>
        </div>
        <div class="act">
          {#if d.configCapable}
            <button class="primary" disabled={opening !== null} onclick={() => open(d.id)}>{opening === d.id ? tr("openingLabel") : tr("open")}</button>
          {/if}
          {#if !isTauri}
            <button class="ghost" aria-label={tr("simUnplug")} title={tr("simUnplug")} onclick={() => simulateUnplug(d.id)}>
              <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
            </button>
          {/if}
        </div>
      </li>
    {/snippet}

    {#if list.length === 0}
      <div class="empty">
        <div class="empty-art"><PadThumb model={MODELS[0]} /></div>
        <p class="empty-title">{tr("noDevices")}</p>
        <p class="muted">{tr("noDevicesHint")}</p>
        <p class="muted">{tr("noDevicesTip")}</p>
      </div>
    {:else}
      <DeviceCarousel devices={list} bind:picked enter={!cameBack} {opening} onopen={open} onunplug={isTauri ? undefined : simulateUnplug} />
      {#if parked.length}
        <details class="fold other">
          <summary>{tr("otherDetected")} <span class="mono">{parked.length}</span></summary>
          <div class="other-body">
            <p class="hint">{tr("otherDetectedHint")}</p>
            <ul class="grid">
              {#each parked as d (d.id)}
                {@render card(d)}
              {/each}
            </ul>
          </div>
        </details>
      {/if}
      <p class="hint vendor">{tr("closeVendorTip")}</p>
    {/if}

    <details class="fold models">
      <summary>{tr("knownModels")} <span class="mono">{MODELS.length}</span></summary>
      <ul>
        {#each MODELS as m (m.id)}
          <li>
            <span class="m-name">{m.name}</span>
            <span class="mono pids">{[...m.wired, ...m.wireless].map(pidHex).join(" ")}</span>
            <span class="tag {m.support}">{tr(SUPPORT[m.support])}</span>
          </li>
        {/each}
      </ul>
    </details>

    <a class="test" href="/buttons">{tr("buttonTest")}</a>
  </div>
{/if}

<style>
  .picker {
    display: grid;
    gap: 20px;
    max-width: 880px;
    margin: 0 auto;
    padding: 24px 0 40px;
  }
  .head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }
  /* Focus lands here (programmatically) when the device that was open is gone; it is not a control. */
  h1:focus { outline: none; }
  h1 {
    margin: 0 0 4px;
    font-size: 28px;
    font-weight: 500;
  }
  .lede {
    margin: 0;
    color: var(--fg-2);
  }
  .note {
    margin: 0;
    font-size: 13px;
    color: var(--fg-2);
  }
  .note:not(:empty) { margin-top: 4px; }
  .head button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .spin { animation: spin 700ms linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .sim {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    padding: 12px 16px;
    border: 1px dashed var(--line-strong);
    border-radius: var(--r-card);
  }
  .sim-text {
    display: grid;
    flex: 1;
    min-width: 0;
    font-size: 13px;
  }
  .sim-text span { color: var(--fg-3); }
  .sim select { max-width: 240px; }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
    gap: 12px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .dev {
    display: grid;
    grid-template-columns: 112px minmax(0, 1fr) auto;
    align-items: center;
    gap: 16px;
    padding: 16px;
    border: 1px solid var(--line);
    border-radius: var(--r-card);
    background: var(--surface);
  }
  .dev.ready { border-color: var(--line-strong); }
  .art {
    display: grid;
    place-items: center;
    height: 84px;
    padding: 8px;
    border-radius: var(--r-ctl);
    background: var(--sidebar);
  }
  .info {
    display: grid;
    gap: 4px;
    min-width: 0;
  }
  .name { font-size: 15px; font-weight: 500; }
  .conn {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--fg-2);
    flex-wrap: wrap;
  }
  .mono { font: 12px var(--mono); color: var(--fg-3); }
  .status {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-size: 13px;
    color: var(--fg-2);
  }
  .status i {
    flex: none;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--fg-3);
    transform: translateY(-1px);
  }
  .status.ok { color: var(--fg); }
  .status.ok i { background: var(--ok); }
  .status.warn i { background: var(--warn); }
  .act {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .ghost {
    display: grid;
    place-items: center;
    width: 32px;
    padding: 0;
    border-color: transparent;
    background: transparent;
    color: var(--fg-3);
  }

  .empty {
    display: grid;
    justify-items: center;
    gap: 8px;
    padding: 40px 24px;
    border: 1px solid var(--line);
    border-radius: var(--r-card);
    background: var(--surface);
    text-align: center;
  }
  .empty-art { width: 160px; opacity: 0.6; }
  .empty-title { margin: 8px 0 0; font-size: 16px; font-weight: 500; }
  .empty .muted { margin: 0; max-width: 420px; }

  .fold {
    border: 1px solid var(--line);
    border-radius: var(--r-card);
  }
  .fold summary {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    font-size: 14px;
    cursor: pointer;
    list-style: none;
  }
  .fold summary::-webkit-details-marker { display: none; }
  .fold summary::after {
    content: "";
    width: 6px;
    height: 6px;
    margin-left: auto;
    border-right: 1.5px solid var(--fg-3);
    border-bottom: 1.5px solid var(--fg-3);
    transform: rotate(45deg);
    transition: transform 160ms var(--ease);
  }
  .fold[open] summary::after { transform: rotate(-135deg); }
  /* Open/close glides instead of snapping (interpolate-size + ::details-content, Chromium 131+). */
  .fold { interpolate-size: allow-keywords; }
  .fold::details-content {
    height: 0;
    overflow: clip;
    opacity: 0;
    transition:
      height 240ms var(--ease),
      opacity 200ms var(--ease),
      content-visibility 240ms var(--ease) allow-discrete;
  }
  .fold[open]::details-content { height: auto; opacity: 1; }
  .other-body {
    display: grid;
    gap: 12px;
    padding: 0 16px 16px;
  }
  .models ul {
    margin: 0;
    padding: 0 16px 8px;
    list-style: none;
  }
  .models li {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    align-items: center;
    gap: 16px;
    min-height: 36px;
    border-top: 1px solid var(--line);
    font-size: 13px;
  }
  .pids { text-align: right; }
  .tag {
    min-width: 108px;
    font-size: 12px;
    text-align: right;
    color: var(--fg-3);
  }
  .tag.full { color: var(--fg); }
  .tag.codec { color: var(--fg-2); }
  .test {
    justify-self: center;
    font-size: 13px;
    color: var(--fg-2);
    padding: 8px 12px;
  }
  @media (max-width: 860px) {
    .grid { grid-template-columns: minmax(0, 1fr); }
    .pids { display: none; }
  }
  @media (prefers-reduced-motion: reduce) {
    .spin { animation: none; }
    .fold summary::after { transition: none; }
  }
</style>
