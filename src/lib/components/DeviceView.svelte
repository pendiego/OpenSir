<script lang="ts">
  import type { ResetScope } from "$lib/api";
  import { app, type View } from "$lib/app.svelte";
  import { tr, type Key } from "$lib/i18n";
  import Overview from "./views/Overview.svelte";
  import Settings from "./views/Settings.svelte";
  import Mapping from "./views/Mapping.svelte";
  import Sticks from "./views/Sticks.svelte";
  import Triggers from "./views/Triggers.svelte";
  import Motion from "./views/Motion.svelte";
  import Motors from "./views/Motors.svelte";
  import Lights from "./views/Lights.svelte";
  import ButtonTest from "./views/ButtonTest.svelte";
  import PrefToggles from "./PrefToggles.svelte";

  // Icon rail: Lucide-style 24px paths, drawn at 20px.
  const NAV: { view: View; label: Key; d: string }[] = [
    { view: "overview", label: "overview", d: "M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h7v7h-7z" },
    { view: "mapping", label: "mapping", d: "M6 11h4M8 9v4M15 12h.01M18 10h.01M17.3 5H6.7a4 4 0 0 0-4 3.6c-.4 3.3-.7 6.8.2 9.3.5 1.3 2.2 1.6 3.2.6L8.9 16h6.2l2.8 2.5c1 1 2.7.7 3.2-.6.9-2.5.6-6-.2-9.3A4 4 0 0 0 17.3 5Z" },
    { view: "sticks", label: "sticks", d: "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18Zm0 5a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z" },
    { view: "triggers", label: "triggers", d: "M8 4h8v6a4 4 0 0 1-8 0V4Zm4 10v6" },
    { view: "motion", label: "motion", d: "M3 12h4l3-7 4 14 3-7h4" },
    { view: "motors", label: "motors", d: "M2 8v8M22 8v8M6 5v14M18 5v14M10 9h4v6h-4z" },
    { view: "lights", label: "lights", d: "M9 18h6M10 21h4M12 3a6 6 0 0 0-3.5 10.9c.6.5 1 1.2 1 2.1h5c0-.9.4-1.6 1-2.1A6 6 0 0 0 12 3Z" },
    { view: "buttons", label: "buttons", d: "M12 2v4M12 18v4M2 12h4M18 12h4M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z" },
    { view: "settings", label: "settings", d: "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6ZM19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1Z" },
  ];

  const SCOPES: Record<ResetScope, string> = {
    all: "scopeAll",
    mapping: "scopeMapping",
    sticks: "scopeSticks",
    motion: "scopeMotion",
    triggers: "scopeTriggers",
    motors: "scopeMotors",
  };

  const STATUS: Record<string, string> = {
    saving: "statusSaving",
    saved: "statusSaved",
    error: "statusError",
  };

  let scope = $state<ResetScope>("mapping");
  let resetDialog: HTMLDialogElement | undefined;
  let menu: HTMLDetailsElement | undefined;
  let importDialog: HTMLDialogElement | undefined;

  const device = $derived(app.devices.find((d) => d.id === app.openId));
  const profiles = $derived(app.snapshot?.profiles ?? []);
  const current = $derived(NAV.find((n) => n.view === app.view));
  const scopeLabel = $derived(tr(SCOPES[scope] as "scopeAll"));
  const canActivate = $derived(!!app.snapshot && app.snapshot.currentProfile !== app.viewIndex);
  const battery = $derived(app.snapshot?.battery ?? null);

  function confirmReset() {
    resetDialog?.close();
    void app.reset(scope);
  }

  function confirmImport() {
    importDialog?.close();
    void app.importInto();
  }

  function pick(action: () => void) {
    if (menu) menu.open = false;
    action();
  }

  function escMenu(e: KeyboardEvent) {
    if (e.key === "Escape" && menu?.open) {
      menu.open = false;
      menu.querySelector("summary")?.focus();
    }
  }
</script>

<svelte:window onkeydown={escMenu} />
<div class="frame">
  <header class="top" data-tauri-drag-region="deep">
    <!-- Back to the controller list (closes the device after pending saves are written). -->
    <button class="back" aria-label={tr("backToControllers")} data-tip={tr("backToControllers")} onclick={() => app.close()}>
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6" /></svg>
    </button>

    <nav class="tabs" aria-label={tr("profile")}>
      <!-- The firmware's default name is English "Profile n": tabs show it in the UI language. -->
      {#each profiles as p, i (i)}
        {@const n = i + 1}
        {@const slot = `${tr("profile")} ${n}`}
        {@const named = !!p.name && p.name !== slot && p.name !== `Profile ${n}`}
        {@const full = [slot, named && p.name, app.snapshot?.currentProfile === n && tr("activeOnController")].filter(Boolean).join(" · ")}
        <button
          class="tab"
          class:active={app.viewIndex === n}
          aria-current={app.viewIndex === n ? "page" : undefined}
          aria-label={full}
          title={full}
          onclick={() => (app.viewIndex = n)}
        >
          <!-- Fixed slot number: names can repeat or share a prefix that the ellipsis hides. -->
          <span class="slot" aria-hidden="true">{n}</span>
          <span class="tab-label" aria-hidden="true">{named ? p.name : slot}</span>
          {#if app.snapshot?.currentProfile === n}
            <span class="on" title={tr("activeOnController")} aria-label={tr("activeOnController")}></span>
          {/if}
        </button>
      {/each}
    </nav>

    <div class="actions">
      {#if app.status !== "idle"}
        <span class="save" class:err={app.status === "error"} role="status">{tr(STATUS[app.status] as "statusSaved")}</span>
      {/if}
      <button class="use" class:primary={canActivate} disabled={!canActivate || !app.writable} onclick={() => app.activate()}>
        {canActivate ? tr("useOnController") : tr("activeOnController")}
      </button>
      <details class="menu" bind:this={menu} data-tauri-drag-region="false">
        <summary aria-label={tr("more")} title={tr("more")}>
          <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true"><circle cx="5" cy="12" r="1.6" fill="currentColor" /><circle cx="12" cy="12" r="1.6" fill="currentColor" /><circle cx="19" cy="12" r="1.6" fill="currentColor" /></svg>
        </summary>
        <div class="pop">
          <button disabled={!app.writable} onclick={() => pick(() => importDialog?.showModal())}>{tr("importJson")}</button>
          <button onclick={() => pick(() => app.exportCurrent())}>{tr("exportJson")}</button>
          <button disabled={!app.writable} onclick={() => pick(() => resetDialog?.showModal())}>{tr("reset")}</button>
        </div>
      </details>
      <PrefToggles />
    </div>
  </header>

  <div class="body">
    <aside class="side">
      <nav class="rail" aria-label={tr("controller")}>
        {#each NAV.filter((n) => !(app.restricted && n.view === "motion")) as n (n.view)}
          <button
            class="rail-item"
            class:current={app.view === n.view}
            aria-current={app.view === n.view ? "page" : undefined}
            aria-label={tr(n.label)}
            data-tip={tr(n.label)}
            onclick={() => (app.view = n.view)}
          >
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d={n.d} /></svg>
          </button>
        {/each}
      </nav>

      {#if device && app.snapshot}
        <div
          class="status"
          role="img"
          aria-label="{device.name} · {tr(device.mode.startsWith('wireless') ? 'connWireless' : 'connWired')}{battery != null ? ` · ${tr('battery')} ${battery}%` : ''}"
          data-tip="{device.name} · {tr('firmware')} {app.snapshot.firmware}"
        >
          <i class="dot" class:warn={battery != null && battery < 20}></i>
          <span class="pct">{battery != null ? `${battery}%` : "–"}</span>
        </div>
      {/if}
    </aside>

    <main class="sheet">
      <div class="page">
        <header class="page-head">
          <h1>{current ? tr(current.label) : ""}</h1>
        </header>

        {#if app.error}
          <div class="notice" role="alert">
            <span>{app.error}</span>
            <button onclick={() => (app.error = "")}>{tr("dismiss")}</button>
          </div>
        {/if}

        {#if app.snapshot && !app.writable}
          <p class="muted" role="note">{tr("readOnly")}</p>
        {/if}
        {#if app.snapshot}
          <!-- Keyed: each page fades in when the rail switches view, so content does not just snap. -->
          {#key app.view}
            <div class="view" inert={!app.writable && app.view !== "buttons" && app.view !== "motors"}>
              {#if app.view === "overview"}<Overview />
              {:else if app.view === "settings"}<Settings />
              {:else if app.view === "mapping"}<Mapping />
              {:else if app.view === "sticks"}<Sticks />
              {:else if app.view === "triggers"}<Triggers />
              {:else if app.view === "motion"}<Motion />
              {:else if app.view === "motors"}<Motors />
              {:else if app.view === "lights"}<Lights />
              {:else if app.view === "buttons"}<ButtonTest />
              {/if}
            </div>
          {/key}
        {/if}
      </div>
    </main>
  </div>

  <dialog bind:this={resetDialog} class="reset" aria-labelledby="reset-title">
    <h2 id="reset-title" class="title">{tr("reset")}</h2>
    <p class="muted">{tr("resetConfirm", { scope: scopeLabel })}</p>
    <label class="field">
      <span>{tr("resetScope")}</span>
      <select bind:value={scope}>
        {#each Object.keys(SCOPES) as s (s)}
          <option value={s}>{tr(SCOPES[s as ResetScope] as "scopeAll")}</option>
        {/each}
      </select>
    </label>
    <div class="row end">
      <button onclick={() => resetDialog?.close()}>{tr("cancel")}</button>
      <button class="danger" onclick={confirmReset}>{tr("reset")}</button>
    </div>
  </dialog>

  <dialog bind:this={importDialog} class="reset" aria-labelledby="import-title">
    <h2 id="import-title" class="title">{tr("importJson")}</h2>
    <p class="muted">{tr("importConfirm")}</p>
    <div class="row end">
      <button onclick={() => importDialog?.close()}>{tr("cancel")}</button>
      <button class="danger" onclick={confirmImport}>{tr("importReplace")}</button>
    </div>
  </dialog>
</div>

<style>
  /* The frame is the darkest layer; the panel (and the active profile tab, which is cut from it)
     is one step lighter, so the tab reads as the top of the page it opens. */
  .frame {
    --rail: 72px;
    --tab-r: 10px;
    --sheet: var(--surface);
    display: grid;
    grid-template-rows: 52px minmax(0, 1fr);
    height: 100%;
    min-height: 0;
    background: var(--sidebar);
    color: var(--fg);
  }

  /* Top bar */
  .top {
    display: grid;
    grid-template-columns: var(--rail) minmax(0, 1fr) auto;
    align-items: end;
    /* Right side stays clear for the window buttons the layout draws over this bar. */
    padding-right: 132px;
  }
  .back {
    position: relative;
    align-self: center;
    justify-self: center;
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    min-height: 0;
    padding: 0;
    border-radius: 10px;
    color: var(--fg-2);
  }
  .back svg {
    width: 20px;
    height: 20px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  @media (hover: hover) and (pointer: fine) {
    .back:hover:not(:disabled) { color: var(--fg); }
  }

  .tabs {
    display: flex;
    align-items: flex-end;
    gap: 4px;
    min-width: 0;
    height: 100%;
    /* Keeps the first tab's concave corner clear of the panel's rounded corner. */
    padding-left: 24px;
  }
  .tab {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 44px;
    min-width: 0;
    padding: 0 16px;
    border: 0;
    border-radius: var(--tab-r) var(--tab-r) 0 0;
    background: transparent;
    color: var(--fg-3);
    font-size: 14px;
    font-weight: 500;
    box-shadow: none;
  }
  /* Hover: a floating pill inside the tab, like the reference; the active tab never shows it. */
  .tab::before {
    content: "";
    position: absolute;
    inset: 4px 2px 6px;
    border-radius: var(--r-ctl);
    background: var(--raised);
    opacity: 0;
    transition: opacity 140ms var(--ease);
  }
  .tab > * { position: relative; }
  @media (hover: hover) and (pointer: fine) {
    .tab:hover:not(.active) { background: transparent; color: var(--fg); }
    .tab:hover:not(.active)::before { opacity: 1; }
  }
  .tab:active:not(:disabled) { transform: none; }
  .tab.active {
    background: var(--sheet);
    color: var(--fg);
  }
  /* Concave corners where the active tab meets the panel. */
  .tab.active::before,
  .tab.active::after {
    content: "";
    position: absolute;
    inset: auto;
    bottom: 0;
    width: var(--tab-r);
    height: var(--tab-r);
    border-radius: 0;
    opacity: 1;
  }
  .tab.active::before {
    left: calc(-1 * var(--tab-r));
    background: radial-gradient(circle at 0 0, transparent calc(var(--tab-r) - 0.5px), var(--sheet) var(--tab-r));
  }
  .tab.active::after {
    right: calc(-1 * var(--tab-r));
    background: radial-gradient(circle at 100% 0, transparent calc(var(--tab-r) - 0.5px), var(--sheet) var(--tab-r));
  }
  .tab:focus-visible { outline: 2px solid var(--accent); outline-offset: -4px; }
  .slot {
    flex: none;
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    border: 1px solid var(--line-strong);
    border-radius: 5px;
    font: 500 11px var(--mono);
    color: inherit;
  }
  .tab-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 160px;
  }
  .on {
    flex: none;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
  }

  .actions {
    align-self: center;
    display: flex;
    align-items: center;
    gap: 8px;
    padding-left: 12px;
  }
  .save { font-size: 12px; color: var(--fg-3); white-space: nowrap; }
  .save.err { color: var(--danger); }
  .use { white-space: nowrap; }
  .use:disabled { opacity: 1; color: var(--fg-3); background: transparent; border-color: transparent; }

  .menu { position: relative; }
  .menu > summary {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--r-ctl);
    color: var(--fg-2);
    list-style: none;
    cursor: pointer;
    transition: background-color 150ms var(--ease), color 150ms var(--ease);
  }
  .menu > summary::-webkit-details-marker { display: none; }
  .menu > summary:hover { background: var(--hover); color: var(--fg); }
  .menu > summary:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
  .pop {
    position: absolute;
    right: 0;
    top: calc(100% + 4px);
    z-index: 20;
    display: grid;
    min-width: 180px;
    padding: 4px;
    border: 1px solid var(--line);
    border-radius: var(--r-ctl);
    background: var(--surface);
    box-shadow: var(--shadow);
    /* Grows out of the ⋯ button. */
    transform-origin: top right;
    transition:
      opacity 150ms var(--ease),
      transform 150ms var(--ease);
  }
  @starting-style {
    .menu[open] .pop {
      opacity: 0;
      transform: scale(0.96) translateY(-4px);
    }
  }
  .view { animation: view-in 180ms var(--ease); }
  @keyframes view-in {
    from { opacity: 0; transform: translateY(4px); }
  }
  .pop button {
    height: 32px;
    padding: 0 10px;
    border: 0;
    background: transparent;
    color: var(--fg);
    font: 14px var(--font);
    text-align: left;
  }
  .pop button:hover { background: var(--hover); }

  /* Body: icon rail + panel */
  .body {
    display: grid;
    grid-template-columns: var(--rail) minmax(0, 1fr);
    /* A fixed row: a rail taller than the window must not stretch the panel past it. */
    grid-template-rows: minmax(0, 1fr);
    min-height: 0;
  }
  .side {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    min-height: 0;
    padding: 4px 0 12px;
  }
  /* Zoomed in on a short window the rail no longer fits: it scrolls (its hover tips get clipped; aria-labels stay). */
  @media (max-height: 559px) {
    .side { overflow-y: auto; scrollbar-width: none; }
  }
  /* The rail is a capsule that hugs its items, not a full-height column. */
  .rail {
    display: grid;
    gap: 6px;
    padding: 6px;
    border: 1px solid var(--line);
    border-radius: 16px;
    background: var(--bg);
  }
  .rail-item {
    position: relative;
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    min-height: 0;
    padding: 0;
    border: 1px solid transparent;
    border-radius: 10px;
    background: transparent;
    color: var(--fg-3);
  }
  .rail-item svg {
    width: 20px;
    height: 20px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  @media (hover: hover) and (pointer: fine) {
    .rail-item:hover:not(.current) { background: var(--raised); color: var(--fg); }
  }
  .rail-item.current {
    background: var(--hover);
    border-color: var(--line-strong);
    box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.06);
    color: var(--fg);
  }
  .rail-item:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }

  /* Tooltips for the icon-only rail and the status chip. */
  [data-tip]::after {
    content: attr(data-tip);
    position: absolute;
    left: calc(100% + 10px);
    top: 50%;
    z-index: 30;
    padding: 4px 8px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--surface);
    box-shadow: var(--shadow);
    color: var(--fg);
    font: 12px var(--font);
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transform: translate(-4px, -50%);
    transition: opacity 120ms var(--ease), transform 120ms var(--ease);
  }
  [data-tip]:hover::after,
  [data-tip]:focus-visible::after {
    opacity: 1;
    transform: translate(0, -50%);
  }

  .status {
    position: relative;
    margin-top: auto;
    display: grid;
    justify-items: center;
    gap: 4px;
    width: 40px;
    padding: 8px 0;
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--bg);
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--ok);
  }
  .dot.warn { background: var(--warn); }
  .pct { font: 11px var(--mono); color: var(--fg-2); font-variant-numeric: tabular-nums; }

  .sheet {
    min-height: 0;
    overflow: auto;
    margin: 0 8px 8px 0;
    border-radius: 12px;
    background: var(--sheet);
  }
  .page {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px 24px 24px;
  }
  /* A view whose root is .fill (Mapping) gets exactly the panel's height and scrolls inside its own panes. */
  .page:has(> .view > :global(.fill)) {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .page:has(> .view > :global(.fill)) > :not(.view) { flex: none; }
  .view:has(> :global(.fill)) {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-height: 0;
  }
  .page-head {
    display: flex;
    align-items: baseline;
    gap: 12px;
    margin-bottom: 16px;
  }
  .page-head h1 {
    margin: 0;
    font-size: 20px;
    font-weight: 500;
  }

  .reset { width: min(420px, calc(100vw - 48px)); }
  .reset .title { font-size: 18px; }

  @media (max-width: 1180px) {
    .tab { padding: 0 12px; }
    .tab-label { max-width: 110px; }
  }
  /* Zoomed in (Ctrl +) or very narrow: the slot keycap and the tab's aria-label/title carry the name. */
  @media (max-width: 760px) {
    .tabs { padding-left: 12px; }
    .tab { padding: 0 10px; gap: 4px; }
    .tab-label { display: none; }
    /* Still announced (role=status); the tab dot and menu keep their places. */
    .save { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); }
  }
  /* 200% zoom on the minimum window: tabs scroll instead of sliding under the actions. */
  @media (max-width: 560px) {
    .tabs { overflow-x: auto; scrollbar-width: none; }
  }

  @media (prefers-reduced-motion: reduce) {
    .tab::before, .menu > summary, [data-tip]::after { transition: none; }
  }
</style>
