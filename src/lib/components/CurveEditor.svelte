<script lang="ts">
  import { tr } from "$lib/i18n";
  import { curvePath, setUserPoint, userPoints } from "$lib/curve";
  import type { DeadZone, Linear } from "$lib/model";

  let {
    dead,
    linear,
    onchange,
  }: {
    dead: DeadZone;
    linear: Linear;
    onchange: () => void;
  } = $props();

  // User points in 0-100 units; the wire holds them relative to the deadzone (see $lib/curve).
  const PRESETS: [number, number][] = [
    [25, 25],
    [50, 50],
    [75, 75],
  ];

  let svg: SVGSVGElement | undefined = $state();
  let drag = -1;

  const pts = $derived(userPoints(dead, linear));
  const poly = $derived(curvePath(dead, linear).map(([x, y]) => `${x * 10},${(100 - y) * 10}`).join(" "));

  // x of point i must stay between its neighbours; the outer ones are the deadzone bounds.
  function clampX(i: number, x: number): number {
    const lo = i === 0 ? dead.front : pts[i - 1][0];
    const hi = i === 2 ? dead.back : pts[i + 1][0];
    return Math.min(Math.max(x, lo), hi);
  }

  function setPoint(i: number, x: number, y: number) {
    const nx = clampX(i, Math.round(x));
    const ny = Math.min(dead.antiBack, Math.max(dead.antiFront, Math.round(y)));
    if (nx === pts[i][0] && ny === pts[i][1]) return;
    // A moved point makes the curve Custom, so the seg shows what is applied.
    linear.curveType = 3;
    setUserPoint(dead, linear, i, nx, ny);
    onchange();
  }

  function fromEvent(e: PointerEvent): [number, number] {
    const r = svg!.getBoundingClientRect();
    return [((e.clientX - r.left) / r.width) * 100, 100 - ((e.clientY - r.top) / r.height) * 100];
  }

  function down(i: number, e: PointerEvent) {
    drag = i;
    (e.target as Element).setPointerCapture(e.pointerId);
    e.preventDefault();
  }

  function move(e: PointerEvent) {
    if (drag < 0) return;
    const [x, y] = fromEvent(e);
    setPoint(drag, x, y);
  }

  function up() {
    drag = -1;
  }

  function key(i: number, e: KeyboardEvent) {
    // A disabled fieldset blocks pointers but not keyboard focus on the handles.
    if (svg?.closest("fieldset:disabled")) return;
    const pt = pts[i];
    if (e.key === "Home" || e.key === "End") {
      e.preventDefault();
      setPoint(i, clampX(i, e.key === "Home" ? 0 : 100), pt[1]);
      return;
    }
    const step = e.shiftKey ? 10 : 1;
    const moves: Record<string, [number, number]> = {
      ArrowLeft: [-step, 0],
      ArrowRight: [step, 0],
      ArrowUp: [0, step],
      ArrowDown: [0, -step],
      PageUp: [0, 10],
      PageDown: [0, -10],
    };
    const m = moves[e.key];
    if (!m) return;
    e.preventDefault();
    setPoint(i, pt[0] + m[0], pt[1] + m[1]);
  }

  // Typed value: setPoint clamps it; the field is then reset to the accepted value.
  function enter(i: number, which: 0 | 1, e: Event) {
    const el = e.currentTarget as HTMLInputElement;
    const n = Number(el.value);
    if (el.value !== "" && Number.isFinite(n)) {
      const [x, y] = pts[i];
      if (which === 0) setPoint(i, n, y);
      else setPoint(i, x, n);
    }
    el.value = String(pts[i][which]);
  }

  function preset(curveType: number) {
    linear.curveType = curveType;
    if (curveType === 0) PRESETS.forEach(([x, y], i) => setUserPoint(dead, linear, i, x, y));
    onchange();
  }
</script>

<div class="curve">
  <div class="head">
    <div class="section-label">{tr("curve")}</div>
    <p class="hint">{tr("curveHint")}</p>
  </div>

  <div class="seg presets" role="group" aria-label={tr("curve")}>
    <button type="button" class:active={linear.curveType === 0} onclick={() => preset(0)}>{tr("curveLinear")}</button>
    <button type="button" class:active={linear.curveType === 1} onclick={() => preset(1)}>{tr("curveConcave")}</button>
    <button type="button" class:active={linear.curveType === 2} onclick={() => preset(2)}>{tr("curveS")}</button>
    <button type="button" class:active={linear.curveType === 3} onclick={() => preset(3)}>{tr("curveCustom")}</button>
  </div>

  <div class="plot-wrap">
    <svg
      bind:this={svg}
      class="plot"
      viewBox="0 0 1000 1000"
      role="group"
      aria-label={tr("points")}
      onpointermove={move}
      onpointerup={up}
      onpointercancel={up}
    >
      <defs>
        <pattern id="curve-hatch" width="60" height="60" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
          <line x1="0" y1="0" x2="0" y2="60" />
        </pattern>
      </defs>
      <rect class="frame" x="0" y="0" width="1000" height="1000" />
      <path class="grid" d="M250 0V1000M500 0V1000M750 0V1000M0 250H1000M0 500H1000M0 750H1000" />
      <!-- Shade what the curve cannot reach: input below/above the deadzone, output outside the anti-deadzone. -->
      <rect class="dead" x="0" y="0" width={dead.front * 10} height="1000" />
      <rect class="dead" x={dead.back * 10} y="0" width={(100 - dead.back) * 10} height="1000" />
      <rect class="dead" x="0" y={(100 - dead.antiFront) * 10} width="1000" height={dead.antiFront * 10} />
      <rect class="dead" x="0" y="0" width="1000" height={(100 - dead.antiBack) * 10} />
      <polyline class="curve-line" points={poly} />
      {#each pts as pt, i (i)}
        <!-- The invisible hit circle is the target (about 28px at full size); the drawn handle stays small. -->
        <circle
          class="hit"
          cx={pt[0] * 10}
          cy={(100 - pt[1]) * 10}
          r="60"
          tabindex="0"
          role="slider"
          aria-label={`${tr("points")} ${i + 1}`}
          aria-valuemin={clampX(i, 0)}
          aria-valuemax={clampX(i, 100)}
          aria-valuenow={pt[0]}
          aria-valuetext={`${tr("input")} ${pt[0]}, ${tr("output")} ${pt[1]}`}
          onpointerdown={(e) => down(i, e)}
          onkeydown={(e) => key(i, e)}
        />
        <circle class="handle" cx={pt[0] * 10} cy={(100 - pt[1]) * 10} r="30" />
      {/each}
    </svg>
    <p class="axes"><span>{tr("output")} ↑</span><span>{tr("input")} →</span></p>
  </div>

  <div class="points">
    {#each pts as pt, i (i)}
      <div class="pt">
        <span>{tr("points")} {i + 1}</span>
        <span class="pair">
          <input type="number" min="0" max="100" step="1" value={pt[0]} aria-label={`${tr("points")} ${i + 1} ${tr("input")}`} onchange={(e) => enter(i, 0, e)} />
          <input type="number" min="0" max="100" step="1" value={pt[1]} aria-label={`${tr("points")} ${i + 1} ${tr("output")}`} onchange={(e) => enter(i, 1, e)} />
        </span>
      </div>
    {/each}
  </div>
</div>

<style>
  .curve {
    display: grid;
    gap: 12px;
    align-content: start;
  }
  .head {
    display: grid;
    gap: 4px;
  }
  .head .section-label {
    margin: 0;
  }
  .presets {
    display: grid;
  }
  .plot-wrap {
    display: grid;
    gap: 4px;
    width: min(100%, 232px);
    justify-self: center;
    align-self: start;
  }
  .plot {
    display: block;
    width: 100%;
    aspect-ratio: 1;
    overflow: visible;
    touch-action: none;
    user-select: none;
  }
  .frame {
    fill: var(--sidebar);
    stroke: var(--line-strong);
    stroke-width: 1;
    vector-effect: non-scaling-stroke;
  }
  .grid {
    fill: none;
    stroke: var(--line);
    stroke-width: 1;
    vector-effect: non-scaling-stroke;
  }
  pattern line {
    stroke: var(--line-strong);
    stroke-width: 16;
  }
  .dead {
    fill: url(#curve-hatch);
  }
  .curve-line {
    fill: none;
    stroke: var(--fg);
    stroke-width: 2;
    stroke-linejoin: round;
    vector-effect: non-scaling-stroke;
  }
  .hit {
    fill: transparent;
    outline: none;
    cursor: grab;
  }
  .handle {
    fill: var(--fg);
    stroke: var(--bg);
    stroke-width: 2;
    vector-effect: non-scaling-stroke;
    pointer-events: none;
  }
  .hit:focus-visible + .handle {
    stroke: var(--accent);
    stroke-width: 3;
  }
  .axes {
    display: flex;
    justify-content: space-between;
    margin: 0;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--fg-3);
  }
  .points {
    display: grid;
    gap: 4px;
    font-size: 13px;
    color: var(--fg-2);
  }
  .pt {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }
  .pair {
    display: inline-flex;
    gap: 4px;
  }
  .pt input {
    width: 4em;
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
  @container shaping (min-width: 520px) {
    .curve {
      grid-template-columns: minmax(0, 1fr) 232px;
      grid-template-areas:
        "head plot"
        "presets plot"
        "points plot";
    }
    .head {
      grid-area: head;
    }
    .presets {
      grid-area: presets;
    }
    .points {
      grid-area: points;
    }
    .plot-wrap {
      grid-area: plot;
    }
  }
</style>
