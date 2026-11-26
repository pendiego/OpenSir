<script lang="ts">
  import "@fontsource/ibm-plex-sans/latin-400.css";
  import "@fontsource/ibm-plex-sans/latin-500.css";
  import "@fontsource/ibm-plex-sans/latin-600.css";
  import "@fontsource/ibm-plex-mono/latin-400.css";
  import "@fontsource/ibm-plex-mono/latin-500.css";
  import "../app.css";
  import { onMount } from "svelte";
  import { app } from "$lib/app.svelte";
  import { isTauri } from "$lib/api";
  import PrefToggles from "$lib/components/PrefToggles.svelte";
  import BrandMark from "$lib/components/BrandMark.svelte";
  import Intro from "$lib/components/Intro.svelte";
  import { tr } from "$lib/i18n";
  import { withViewTransition } from "$lib/motion";

  // Launch intro: once per window session (dev reloads don't replay it), never with reduced motion.
  function wantsIntro(): boolean {
    if (matchMedia("(prefers-reduced-motion: reduce)").matches || sessionStorage.getItem("opensir-intro")) return false;
    sessionStorage.setItem("opensir-intro", "1");
    return true;
  }
  let intro = $state(wantsIntro());

  let { children } = $props();

  onMount(() => {
    app.init();
    // Save-first close: hold each close request until the open device has flushed its queued writes, then destroy.
    let unlisten: (() => void) | undefined;
    win().then(async (w) => {
      if (!w) return;
      unlisten = await w.onCloseRequested(async (event) => {
        event.preventDefault();
        try {
          await app.close();
        } finally {
          await w.destroy();
        }
      });
    });
    return () => unlisten?.();
  });

  $effect(() => {
    document.documentElement.dataset.theme = app.theme;
    document.documentElement.lang = app.locale;
  });

  // Heartbeat lives here, not in DeviceView, so it keeps running on the /buttons page.
  $effect(() => (app.openId ? app.keepAlive() : undefined));

  async function win() {
    if (!isTauri) return null;
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    return getCurrentWindow();
  }

  async function minimize() {
    (await win())?.minimize();
  }

  async function toggleMax() {
    (await win())?.toggleMaximize();
  }

  async function close() {
    (await win())?.close();
  }
</script>

<div class="app">
  <header class="titlebar" class:over={!!app.openId} data-tauri-drag-region="deep">
    {#if !app.openId}
      <span class="brand" class:hidden={intro}>
        <!-- Takes the intro mark's name once the intro is gone, so the mark flies here. -->
        <span style:view-transition-name={intro ? "none" : "brand-mark"}><BrandMark /></span>
        OpenSir
      </span>
      <PrefToggles />
    {/if}
    <!-- Like native caption buttons: out of the Tab order (Win+↓ / Alt+F4 work), still reachable by screen readers. -->
    <div class="winbtns">
      <button tabindex="-1" onclick={minimize} aria-label={tr("winMinimize")} title={tr("winMinimize")}>
        <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 8h10" /></svg>
      </button>
      <button tabindex="-1" onclick={toggleMax} aria-label={tr("winMaximize")} title={tr("winMaximize")}>
        <svg viewBox="0 0 16 16" aria-hidden="true"><rect x="3" y="3" width="10" height="10" rx="1" /></svg>
      </button>
      <button tabindex="-1" class="close" onclick={close} aria-label={tr("winClose")} title={tr("winClose")}>
        <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" /></svg>
      </button>
    </div>
  </header>
  {#if intro}<Intro ondone={() => withViewTransition(() => (intro = false))} />{/if}
  <div class="content" class:flush={!!app.openId}>
    {@render children()}
  </div>
</div>

<style>
  .app {
    display: grid;
    grid-template-rows: auto 1fr;
    height: 100%;
    background: var(--bg);
  }
  .titlebar {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 32px;
    justify-content: flex-end;
    padding: 0 0 0 16px;
    background: var(--bg);
    border-bottom: 1px solid var(--line);
  }
  .titlebar:has(.brand) { padding-left: 16px; }
  /* With a device open the shell's own top bar is the title bar: only the window buttons float over its right end. */
  .titlebar.over {
    position: fixed;
    top: 0;
    right: 0;
    z-index: 10;
    height: 52px;
    padding: 0;
    background: transparent;
    border-bottom: 0;
  }
  .titlebar.over + .content { grid-row: 1 / -1; }
  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    font-size: 14px;
    font-weight: 500;
    color: var(--fg);
  }
  /* While the intro plays its mark is the only one on screen; this one receives it. */
  .brand.hidden { visibility: hidden; }
  .winbtns {
    display: flex;
    align-items: center;
    height: 100%;
  }
  .winbtns button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 32px;
    padding: 0;
    border: 0;
    border-radius: 0;
    background: transparent;
    color: var(--fg-2);
    transition: background-color 150ms var(--ease), color 150ms var(--ease);
  }
  .winbtns svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
  }
  .winbtns button:hover { background: var(--hover); color: var(--fg); }
  .winbtns button:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
  .winbtns button.close:hover { background: var(--danger); color: var(--surface); }
  .content {
    overflow: auto;
    min-height: 0;
    padding: 24px;
  }
  .content.flush {
    padding: 0;
  }

  @media (prefers-reduced-motion: reduce) {
    .winbtns button { transition: none; }
  }
</style>
