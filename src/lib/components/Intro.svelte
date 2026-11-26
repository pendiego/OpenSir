<script lang="ts">
  import { onMount } from "svelte";
  import BrandMark from "./BrandMark.svelte";
  import { tr } from "$lib/i18n";

  // Launch intro, once per start: the bezel ring draws, the cap lights up like the pad's LED, the
  // wordmark settles, then the mark flies into the title bar (view transition `brand-mark`).
  // Any key or click skips it; it never blocks the device scan running underneath.
  let { ondone }: { ondone: () => void } = $props();

  const HOLD_MS = 1500;
  let finished = false;
  function finish() {
    if (finished) return;
    finished = true;
    ondone();
  }

  onMount(() => {
    const t = setTimeout(finish, HOLD_MS);
    return () => clearTimeout(t);
  });
</script>

<svelte:window onkeydown={finish} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="intro" onpointerdown={finish} aria-hidden="true">
  <div class="lockup">
    <div class="mark-slot"><BrandMark size={88} class="intro-mark" /></div>
    <div class="words">
      <span class="word">OpenSir</span>
      <span class="tag">{tr("introTagline")}</span>
    </div>
  </div>
</div>

<style>
  .intro {
    position: fixed;
    /* Below the 32px title bar, so the window buttons and dragging keep working. */
    inset: 32px 0 0;
    z-index: 50;
    display: grid;
    place-items: center;
    background: var(--bg);
    cursor: default;
  }
  /* Faint bench light behind the mark, the only "glow" in the identity. */
  .intro::before {
    content: "";
    position: absolute;
    inset: 0;
    background: radial-gradient(420px 300px at 50% 46%, var(--accent-soft), transparent 70%);
    opacity: 0;
    animation: fade 900ms var(--ease) 420ms forwards;
  }
  .lockup {
    position: relative;
    display: flex;
    align-items: center;
    gap: 24px;
  }
  .mark-slot {
    color: var(--fg);
    view-transition-name: brand-mark;
  }
  /* Lead: the ring draws clockwise from twelve o'clock while it settles from 92%. */
  .mark-slot :global(.ring) {
    /* 101 > pathLength 100: the finished dash overlaps itself, so no hairline gap at twelve o'clock. */
    stroke-dasharray: 101;
    stroke-dashoffset: 101;
    transform: rotate(-90deg);
    transform-origin: 24px 24px;
    animation: draw 700ms var(--ease) 60ms forwards;
  }
  .mark-slot :global(svg) { animation: settle 700ms var(--ease) both; }
  /* The cap lights as the ring closes: cause, then effect. */
  .mark-slot :global(.cap) {
    transform-origin: 24px 24px;
    animation: light 420ms var(--ease) 440ms both;
  }
  .words {
    display: grid;
    gap: 6px;
    animation: rise 520ms var(--ease) 560ms both;
  }
  .word {
    font-size: 44px;
    font-weight: 500;
    letter-spacing: -0.02em;
    line-height: 1;
    color: var(--fg);
  }
  .tag {
    font: 13px var(--mono);
    color: var(--fg-2);
    opacity: 0;
    animation: fade 420ms var(--ease) 780ms forwards;
  }

  @keyframes draw { to { stroke-dashoffset: 0; } }
  @keyframes settle { from { transform: scale(0.92); } }
  @keyframes light {
    from { opacity: 0; transform: scale(0.5); filter: drop-shadow(0 0 0 transparent); }
    60% { filter: drop-shadow(0 0 10px var(--accent)); }
    to { opacity: 1; transform: none; filter: drop-shadow(0 0 4px var(--accent)); }
  }
  @keyframes rise { from { opacity: 0; transform: translateY(10px); } }
  @keyframes fade { to { opacity: 1; } }
</style>
