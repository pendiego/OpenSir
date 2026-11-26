<script lang="ts">
  import { tick } from "svelte";
  import type { DeviceInfo } from "$lib/model";
  import { deviceStatus, isWireless, modelOf, pidHex } from "$lib/models";
  import { tr } from "$lib/i18n";

  let {
    devices,
    opening,
    onopen,
    onunplug,
    picked = $bindable(null),
    enter = true,
  }: {
    devices: DeviceInfo[];
    opening: string | null;
    onopen: (id: string) => void;
    /** Browser preview only: unplug a simulated controller. */
    onunplug?: (id: string) => void;
    /** Id of the front device. Owned by the page so it survives opening a device and coming back. */
    picked?: string | null;
    /** Play the card entrance. Off when coming back from a device, where the art morphs instead. */
    enter?: boolean;
  } = $props();

  // Selection follows the device id, so hotplug reordering keeps the same pad in front.
  const index = $derived(Math.max(0, devices.findIndex((d) => d.id === picked)));
  const current = $derived(devices[index]);

  let root: HTMLElement;

  async function go(i: number) {
    const n = Math.min(Math.max(i, 0), devices.length - 1);
    const had = root.contains(document.activeElement);
    picked = devices[n]?.id ?? null;
    await tick();
    // The focused Open/✕ or side card is gone after the swap: hand focus to the new front card.
    if (had && !root.contains(document.activeElement)) {
      (root.querySelector<HTMLElement>(".slide.front .act button") ?? root).focus();
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if ((e.target as HTMLElement).closest("input, select, textarea")) return;
    const to = { ArrowLeft: index - 1, ArrowRight: index + 1, Home: 0, End: devices.length - 1 }[e.key];
    if (to === undefined) return;
    e.preventDefault();
    go(to);
  }

  // Swipe: a horizontal drag of 48px moves one card.
  let downX: number | null = null;
  function pointerdown(e: PointerEvent) {
    if (e.pointerType !== "mouse") downX = e.clientX;
  }
  function pointerup(e: PointerEvent) {
    if (downX === null) return;
    const dx = e.clientX - downX;
    downX = null;
    if (Math.abs(dx) > 48) go(index + (dx < 0 ? 1 : -1));
  }

  // Neighbours shrink and step back; past two away they fade out.
  function place(off: number) {
    const a = Math.abs(off);
    const scale = a === 0 ? 1 : a === 1 ? 0.82 : 0.68;
    const x = off === 0 ? 0 : Math.sign(off) * (a === 1 ? 74 : 128);
    return { transform: `translateX(${x}%) scale(${scale})`, z: 10 - a, opacity: a > 2 ? 0 : 1 };
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<section
  bind:this={root}
  class="carousel"
  class:enter
  tabindex="-1"
  aria-roledescription="carousel"
  aria-label={tr("devices")}
  {onkeydown}
  onpointerdown={pointerdown}
  onpointerup={pointerup}
  onpointercancel={() => (downX = null)}
>
  <!-- Ambient light: the front controller's own art, blurred. -->
  <div class="glow" aria-hidden="true">
    {#each devices as d (d.id)}
      <img src={modelOf(d).pad.cover} alt="" class:on={d === current} class:dim={!d.configCapable} />
    {/each}
  </div>

  <ul class="track">
    {#each devices as d, k (d.id)}
      {@const m = modelOf(d)}
      {@const s = deviceStatus(d)}
      {@const p = place(k - index)}
      {@const front = k === index}
      <li
        class="slide"
        class:front
        style:transform={p.transform}
        style:z-index={p.z}
        style:--i={Math.abs(k - index)}
        style:opacity={p.opacity}
        aria-roledescription="slide"
        aria-label={tr("carouselPos", { i: k + 1, n: devices.length })}
        aria-current={front ? "true" : undefined}
        inert={Math.abs(k - index) > 2}
      >
        <article class="card" class:ready={d.configCapable}>
          <div class="backdrop" aria-hidden="true"><img src={m.pad.cover} alt="" /></div>
          <div class="top">
            <span class="chip">
              {#if isWireless(d)}
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12.5a10 10 0 0 1 14 0M8.5 16a5 5 0 0 1 7 0M12 19.5h.01" /></svg>
                {tr("connWireless")}
              {:else}
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3v12M9 6l3-3 3 3M8 11v4h8v-4M12 15v6" /></svg>
                {tr("connWired")}
              {/if}
            </span>
            <span class="pid">{pidHex(d.pid)}</span>
          </div>
          <img class="art" class:mono={m.pad.mono || !d.configCapable} src={m.pad.cover} alt="" />
          <div class="info">
            <h2 class="name">{d.name}</h2>
            <p class="status {s.tone}"><i></i>{tr(s.key)}</p>
          </div>
          {#if front}
            <div class="act">
              {#if d.configCapable}
                <button class="primary" disabled={opening !== null} onclick={() => onopen(d.id)}>
                  {opening === d.id ? tr("openingLabel") : tr("open")}
                </button>
              {/if}
              {#if onunplug}
                <button class="ghost" aria-label={tr("simUnplug")} title={tr("simUnplug")} onclick={() => onunplug(d.id)}>
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
                </button>
              {/if}
            </div>
          {:else}
            <!-- A card behind the front one brings itself forward. -->
            <button class="peek" aria-label={tr("carouselShow", { name: d.name })} onclick={() => go(k)}></button>
          {/if}
        </article>
      </li>
    {/each}
  </ul>

  {#if devices.length > 1}
    <div class="nav">
      <button class="arrow" aria-label={tr("carouselPrev")} disabled={index === 0} onclick={() => go(index - 1)}>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 6l-6 6 6 6" /></svg>
      </button>
      <div class="dots" aria-hidden="true">
        {#each devices as d, k (d.id)}<i class:on={k === index}></i>{/each}
      </div>
      <button class="arrow" aria-label={tr("carouselNext")} disabled={index === devices.length - 1} onclick={() => go(index + 1)}>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 6l6 6-6 6" /></svg>
      </button>
    </div>
  {/if}
</section>

<style>
  .carousel {
    position: relative;
    display: grid;
    gap: 16px;
    padding: 8px 0 0;
    touch-action: pan-y;
    /* Far cards and the glow must not widen the page (narrow windows, Ctrl+ zoom). */
    overflow-x: clip;
  }
  .glow {
    position: absolute;
    inset: -80px 0 -20px;
    pointer-events: none;
    overflow: hidden;
    /* Fade to nothing before the box edge, so the light has no visible border. */
    mask-image: radial-gradient(closest-side, #000 35%, transparent);
  }
  .glow img {
    position: absolute;
    left: 50%;
    top: 46%;
    width: 440px;
    transform: translate(-50%, -50%) scale(1.15);
    filter: blur(64px) saturate(1.6);
    opacity: 0;
    transition: opacity 600ms var(--ease);
  }
  .glow img.on { opacity: 0.5; }
  .glow img.on.dim { opacity: 0.22; filter: blur(64px) grayscale(1); }
  :global(:root[data-theme="light"]) .glow img.on { opacity: 0.35; }

  .track {
    position: relative;
    display: grid;
    place-items: center;
    height: 380px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .slide {
    grid-area: 1 / 1;
    width: 272px;
    transition:
      transform 360ms var(--ease),
      opacity 360ms var(--ease),
      filter 360ms var(--ease);
    filter: brightness(0.6) saturate(0.6);
  }
  .slide.front { filter: none; }
  :global(:root[data-theme="light"]) .slide:not(.front) { filter: saturate(0.5) opacity(0.6); }

  .card {
    position: relative;
    display: grid;
    grid-template-rows: auto 1fr auto auto;
    gap: 12px;
    height: 360px;
    padding: 16px;
    overflow: hidden;
    border: 1px solid var(--line);
    border-radius: var(--r-panel);
    background: var(--surface);
    box-shadow: var(--shadow);
  }
  /* Entrance (first visit, a controller plugged in): cards rise in from the centre outward. */
  .carousel.enter .card {
    transition:
      opacity 320ms var(--ease),
      transform 320ms var(--ease);
    transition-delay: calc(var(--i) * 60ms);
  }
  @starting-style {
    .carousel.enter .card {
      opacity: 0;
      transform: translateY(16px) scale(0.97);
    }
  }
  .front .card { border-color: var(--line-strong); }
  .card > :not(.backdrop, .peek) { position: relative; }
  /* The card takes its colour from its own controller, softly. */
  .backdrop {
    position: absolute;
    inset: 0;
    opacity: 0.32;
    pointer-events: none;
  }
  .backdrop img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    filter: blur(28px) saturate(1.4);
    transform: scale(1.4);
  }
  .card:not(.ready) .backdrop { opacity: 0.14; filter: grayscale(1); }

  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 8px 2px 6px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: var(--bg);
    font-size: 12px;
    color: var(--fg-2);
  }
  .pid { font: 12px var(--mono); color: var(--fg-2); }
  .art {
    align-self: center;
    justify-self: center;
    width: 100%;
    max-height: 168px;
    object-fit: contain;
    filter: drop-shadow(0 12px 18px rgb(0 0 0 / 0.35));
  }
  /* Opening a controller morphs this art into the Overview drawing (src/lib/motion.ts). */
  .front .art { view-transition-name: pad-art; }
  .art.mono { filter: grayscale(1) drop-shadow(0 12px 18px rgb(0 0 0 / 0.35)); opacity: 0.7; }
  .info { display: grid; gap: 4px; }
  .name {
    margin: 0;
    font-size: 18px;
    font-weight: 500;
    line-height: 1.25;
  }
  .status {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin: 0;
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
  .status.ok i { background: var(--ok); }
  .status.warn i { background: var(--warn); }
  .act {
    display: flex;
    gap: 8px;
  }
  .act .primary { flex: 1; min-height: 36px; }
  .ghost {
    display: inline-grid;
    place-items: center;
    width: 36px;
    min-height: 36px;
    padding: 0;
    background: transparent;
  }
  .peek {
    position: absolute;
    inset: 0;
    min-height: 0;
    border: 0;
    border-radius: inherit;
    background: transparent;
    cursor: pointer;
  }
  @media (hover: hover) and (pointer: fine) {
    .peek:hover:not(:disabled) { background: rgb(255 255 255 / 0.04); }
  }
  .peek:active:not(:disabled) { transform: none; }

  .nav {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
  }
  .arrow {
    display: inline-grid;
    place-items: center;
    width: 36px;
    min-height: 36px;
    padding: 0;
    border-radius: 50%;
  }
  .dots { display: flex; gap: 6px; }
  .dots i {
    width: 6px;
    height: 6px;
    border-radius: 3px;
    background: var(--line-strong);
    transition: width 300ms var(--ease), background-color 300ms var(--ease);
  }
  .dots i.on { width: 18px; background: var(--fg-2); }

  svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .chip svg { width: 14px; height: 14px; }

  @media (prefers-reduced-motion: reduce) {
    .slide,
    .glow img,
    .dots i { transition: none; }
  }
</style>
