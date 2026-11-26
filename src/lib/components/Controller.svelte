<script lang="ts">
  import { KEYS, keyOf, marked, sameTarget, type KeyInfo, type KeyTarget } from "$lib/keys";
  import { KEY_CODES } from "$lib/keycodes";
  import type { GamepadState, Profile } from "$lib/model";
  import type { ModelInfo, PadSpot } from "$lib/models";
  import { tr } from "$lib/i18n";

  let {
    profile,
    model,
    selected = null,
    onselect,
    live = null,
    leds,
  }: {
    profile?: Profile;
    model: ModelInfo;
    selected?: KeyTarget | null;
    onselect?: (t: KeyTarget) => void;
    /** Live input: highlights pressed keys and moves sticks and triggers. */
    live?: GamepadState | null;
    /** RGB per LED slot in device order: 0 lower-left grip, 1 lower-right grip, 3 home ring, 4 back. */
    leds?: [number, number, number][];
  } = $props();

  type Spot = KeyInfo & PadSpot;
  // Hotspots come from the model's measured pad; a key without a spot on this cover is not drawn.
  const spots = $derived<Spot[]>(KEYS.filter((k) => model.pad.spots[k.id]).map((k) => ({ ...k, ...model.pad.spots[k.id] })));
  const extraSpots = $derived<Spot[]>([
    ...model.pad.extra.map((e, i) => ({ ...e, id: `X${i}`, label: model.extraKeys[i] ?? "", target: { kind: "extra" as const, index: i } })),
    ...model.pad.touchKeys.map((e, i) => ({ ...e, id: `T${i}`, label: `Ext${8 + i}`, target: { kind: "touch" as const, index: i } })),
  ]);
  // shortcut: stick cap travel is in cover units; upgrade if sticks need full range
  const TRAVEL = 9;
  const pressed = $derived(new Set(live?.buttons ?? []));
  const ps = $derived(model.pad.spots.PS);

  // ABXY legends as strokes centred on (0,0): font text sat off-centre (its baseline depends on the
  // font the webview picks), and the box of each glyph here is symmetric about the origin.
  const GLYPH: Record<string, string> = {
    A: "M-10 11L0 -11L10 11M-6.2 3.5H6.2",
    B: "M-8 -11V11H1.5C6.5 11 9 8.3 9 5.5C9 2.7 6.5 0 1.5 0H-8M-8 0H0.5C5 0 7.5 -2.5 7.5 -5.5C7.5 -8.5 5 -11 0.5 -11H-8",
    X: "M-9 -11L9 11M9 -11L-9 11",
    Y: "M-9.5 -11L0 0.5L9.5 -11M0 0.5V11",
  };

  const led = (i: number) => {
    const c = leds?.[i];
    return c ? `rgb(${c[0]} ${c[1]} ${c[2]})` : "#ffa31a";
  };
  const rot = (k: Spot) => (k.shape.kind === "rect" && k.shape.rot ? `rotate(${k.shape.rot} ${k.x} ${k.y})` : undefined);
  // HID axes: y grows downward (stick up reads -1), the same as SVG y, so no flip.
  const travel = (k: Spot) =>
    k.axes && live ? `translate(${live[k.axes[0]] * TRAVEL} ${live[k.axes[1]] * TRAVEL})` : undefined;
  const dot = (k: Spot) =>
    k.shape.kind === "circle"
      ? { x: k.x + k.shape.r * 0.72, y: k.y - k.shape.r * 0.72 }
      : { x: k.x + k.shape.w / 2 - 6, y: k.y - k.shape.h / 2 + 6 };
  /** Live trigger fill: a bar of the key's shape, `frac` of its width from the left. */
  function bar(k: Spot, frac: number) {
    if (k.shape.kind !== "rect") return null;
    const { w, h, rx } = k.shape;
    return { x: k.x - w / 2, y: k.y - h / 2, width: w * frac, height: h, rx, transform: rot(k) };
  }

  function names(k: Spot): string {
    if (!profile) return "";
    return keyOf(profile, k.target)
      .map.filter((v) => v > 0)
      .map((v) => KEY_CODES.find(([c]) => c === v)?.[1] ?? String(v))
      .join(" + ");
  }

  function spotName(k: Spot): string {
    return k.name ? tr(k.name) : k.label;
  }

  function press(k: Spot, e: KeyboardEvent) {
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    onselect?.(k.target);
  }
</script>

{#snippet shape(k: Spot, cls: string)}
  {#if k.shape.kind === "circle"}
    <circle class={cls} cx={k.x} cy={k.y} r={k.shape.r} />
  {:else}
    <rect
      class={cls}
      x={k.x - k.shape.w / 2}
      y={k.y - k.shape.h / 2}
      width={k.shape.w}
      height={k.shape.h}
      rx={k.shape.rx}
      transform={rot(k)}
    />
  {/if}
{/snippet}

<svg
  class="pad"
  class:interactive={!!onselect}
  viewBox="0 -60 1000 {model.pad.height + 145}"
  role="group"
  aria-label={tr("controller")}
>
  <image class="cover" class:mono={model.pad.mono} href={model.pad.cover} x="0" y="0" width="1000" height={model.pad.height} aria-hidden="true" />
  <g aria-hidden="true">
    {#if leds}
      {#each model.pad.leds as l (l.slot)}
        <ellipse class="led" cx={l.x} cy={l.y} rx={l.rx} ry={l.ry} transform="rotate({l.rot ?? 0} {l.x} {l.y})" fill={led(l.slot)} />
      {/each}
      {#if ps && ps.shape.kind === "circle"}
        <circle class="home-glow" cx={ps.x} cy={ps.y} r={ps.shape.r + 4} stroke={led(3)} />
      {/if}
    {/if}
    {#each [...spots, ...extraSpots].filter((k) => k.chip) as k (k.id)}
      <g class="chip-g">
        {@render shape(k, "chip")}
        <text class="chip-label" x={k.x} y={k.y}>{k.label}</text>
      </g>
    {/each}
    {#if live}
      {#each spots.filter((k) => k.target.kind === "trigger") as k (k.id)}
        {@const frac = (k.target.kind === "trigger" && k.target.name === "l2" ? live.lt : live.rt) / 255}
        {@const b = bar(k, frac)}
        {#if b}<rect class="live" {...b} />{/if}
      {/each}
      <!-- Stick position: the cover's cap is static, so a dot travels inside the stick ring. Axes are
           normalized independently: the vector is clamped so a full diagonal stays inside the ring. -->
      {#each spots.filter((k) => k.axes && k.shape.kind === "circle") as k (k.id)}
        {@const ax = live[k.axes![0]]}
        {@const ay = live[k.axes![1]]}
        {@const reach = (k.shape.kind === "circle" ? k.shape.r * 0.7 : 0) / Math.max(1, Math.hypot(ax, ay))}
        <circle class="stick-dot" cx={k.x + ax * reach} cy={k.y + ay * reach} r="12" />
      {/each}
    {/if}
  </g>
  <!-- hit layer: only when the drawing takes input or live state -->
  {#if onselect || live}
    {#each [...spots, ...extraSpots] as k (k.id)}
      {@const isSel = sameTarget(k.target, selected)}
      {@const isMarked = !!profile && marked(profile, k.target)}
      {@const d = dot(k)}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <g
        class="key"
        class:sel={isSel}
        class:marked={isMarked}
        class:on={!!live && pressed.has(k.id)}
        class:glyph={!!GLYPH[k.label]}
        class:interactive={!!onselect}
        role={onselect ? "button" : "img"}
        tabindex={onselect ? 0 : undefined}
        aria-label={spotName(k) + (names(k) ? `, ${tr("mappedTo")} ${names(k)}` : "")}
        aria-pressed={onselect ? isSel : undefined}
        transform={travel(k)}
        onclick={() => onselect?.(k.target)}
        onkeydown={(e) => onselect && press(k, e)}
      >
        <title>{names(k) ? `${spotName(k)}: ${names(k)}` : spotName(k)}</title>
        {@render shape(k, "hit")}
        {@render shape(k, "hl")}
        {@render shape(k, "tint")}
        {#if live && GLYPH[k.label] && pressed.has(k.id)}
          <path class="letter pressed-letter" d={GLYPH[k.label]} transform="translate({k.x} {k.y})" />
        {/if}
        {#if isMarked}<circle class="badge" cx={d.x} cy={d.y} r="6" />{/if}
      </g>
    {/each}
  {/if}
</svg>

<style>
  .pad {
    display: block;
    width: 100%;
    max-width: none;
    height: auto;
    margin: 0 auto;
    overflow: visible;
  }
  .mono {
    filter: grayscale(1) contrast(0.8);
  }
  .led {
    opacity: 0.8;
    filter: blur(8px);
    transition: fill 120ms linear;
  }
  .home-glow {
    fill: none;
    stroke-width: 12;
    opacity: 0.55;
    transition: stroke 120ms linear;
  }
  .chip {
    fill: var(--raised);
    stroke: var(--line-strong);
    stroke-width: 1.5;
  }
  .chip-label {
    fill: var(--fg-2);
    font: 600 18px var(--font);
    text-anchor: middle;
    dominant-baseline: central;
    pointer-events: none;
  }
  .live {
    fill: var(--accent);
    opacity: 0.7;
  }
  .stick-dot {
    fill: var(--accent);
    stroke: #1c1304;
    stroke-width: 3;
  }

  /* Invisible stroke widens small hotspots (D-pad, triggers, Share) toward a 24px target. */
  .hit {
    fill: transparent;
    stroke: transparent;
    stroke-width: 14;
  }
  .hl {
    fill: none;
    stroke: transparent;
    stroke-width: 3;
  }
  .tint {
    fill: var(--accent);
    opacity: 0;
    transition: opacity 150ms ease-out;
  }
  /* Pressed: instant on (taps are short), fades out on release. Tint stays light so the
     face letter under it reads; the outline + glow carry the state. */
  .key.on .tint {
    opacity: 0.5;
    transition-duration: 0ms;
  }
  /* ABXY: near-solid amber with the legend redrawn on top in dark ink, readable on every pad colour. */
  .key.on.glyph .tint {
    opacity: 0.9;
  }
  .letter {
    fill: none;
    stroke: var(--letter);
    stroke-width: 4;
    stroke-linecap: round;
    stroke-linejoin: round;
    pointer-events: none;
  }
  .pressed-letter {
    stroke: #1c1304;
  }
  .key.on .hl {
    stroke: var(--accent);
    stroke-width: 4;
    filter: drop-shadow(0 0 6px var(--accent));
  }
  .key.marked .hl {
    stroke: var(--accent);
  }
  .key.sel .hl {
    stroke: var(--accent);
    stroke-width: 5;
    filter: drop-shadow(0 0 6px var(--accent));
  }
  .key.interactive {
    cursor: pointer;
    outline: none;
  }
  .key.interactive:focus-visible .hl {
    stroke: var(--accent);
    stroke-width: 4;
  }
  @media (hover: hover) and (pointer: fine) {
    .key.interactive:hover .hl {
      stroke: var(--accent);
      stroke-opacity: 0.7;
    }
  }
  .badge {
    fill: var(--accent);
    stroke: #fff;
    stroke-width: 2;
    pointer-events: none;
  }
  @media (prefers-reduced-motion: reduce) {
    .tint {
      transition: none;
    }
  }
</style>
