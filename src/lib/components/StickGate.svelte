<script lang="ts">
  import { tr } from "$lib/i18n";
  import type { DeadZone } from "$lib/model";

  let { dead, square, label }: { dead: DeadZone; square: boolean; label: string } = $props();

  // Stick travel drawn as radius 64 around (80,80); percentages map onto that radius.
  const R = 64;
  const C = 80;
  const inner = $derived((Math.min(dead.front, dead.back) / 100) * R);
  const outer = $derived((Math.max(dead.front, dead.back) / 100) * R);
  // A square gate lets the stick reach the corners; draw its travel as a rounded square.
  const gate = $derived(
    square
      ? `M${C - R} ${C - R + 10}Q${C - R} ${C - R} ${C - R + 10} ${C - R}H${C + R - 10}Q${C + R} ${C - R} ${C + R} ${C - R + 10}V${C + R - 10}Q${C + R} ${C + R} ${C + R - 10} ${C + R}H${C - R + 10}Q${C - R} ${C + R} ${C - R} ${C + R - 10}Z`
      : `M${C - R} ${C}a${R} ${R} 0 1 0 ${2 * R} 0a${R} ${R} 0 1 0 ${-2 * R} 0Z`,
  );
</script>

<svg class="gate" viewBox="0 0 160 160" role="img" aria-label={`${label}. ${tr("gateSummary", { low: Math.min(dead.front, dead.back), high: Math.max(dead.front, dead.back) })}`}>
  <defs>
    <pattern id="gate-hatch" width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
      <path d="M0 0V6" class="hatch" />
    </pattern>
  </defs>
  <path d={gate} class="travel" />
  <!-- Beyond the outer deadzone the output is already at full deflection. -->
  <path d="{gate}M{C - outer} {C}a{outer} {outer} 0 1 0 {2 * outer} 0a{outer} {outer} 0 1 0 {-2 * outer} 0Z" class="clip" fill="url(#gate-hatch)" fill-rule="evenodd" />
  <path d="M{C - R} {C}H{C + R}M{C} {C - R}V{C + R}" class="axis" />
  <circle cx={C} cy={C} r={outer} class="ring" />
  <circle cx={C} cy={C} r={inner} class="dead" fill="url(#gate-hatch)" />
  <circle cx={C} cy={C} r={inner} class="ring inner" />
  <text x={C + outer * 0.71 + 4} y={C - outer * 0.71 - 4} class="num">{Math.max(dead.front, dead.back)}</text>
  <text x={C + 4} y={C - inner - 4} class="num">{Math.min(dead.front, dead.back)}</text>
</svg>

<style>
  .gate {
    display: block;
    width: 100%;
    max-width: 200px;
    height: auto;
    margin: 0 auto;
  }
  .travel {
    fill: var(--sidebar);
    stroke: var(--line-strong);
    stroke-width: 1;
  }
  .hatch {
    stroke: var(--line-strong);
    stroke-width: 1;
  }
  .clip {
    stroke: none;
  }
  .axis {
    stroke: var(--line);
    stroke-width: 1;
  }
  .ring {
    fill: none;
    stroke: var(--fg-2);
    stroke-width: 1;
    stroke-dasharray: 3 3;
  }
  .ring.inner {
    stroke: var(--fg);
    stroke-dasharray: none;
  }
  .dead {
    stroke: none;
  }
  .num {
    font: 9px var(--mono);
    fill: var(--fg-3);
  }
</style>
