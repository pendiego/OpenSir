<script lang="ts">
  // OpenSir mark: a thumbstick seen from above — the bezel ring and the cap, lit like the pad's LED.
  // One drawing for every size (titlebar 16px, intro 96px); the app icon is built from the same geometry.
  let { size = 16, lit = true, class: cls = "" }: { size?: number; lit?: boolean; class?: string } = $props();
</script>

<svg class="mark {cls}" width={size} height={size} viewBox="0 0 48 48" aria-hidden="true">
  <circle class="ring" cx="24" cy="24" r="18" pathLength="100" />
  <circle class="cap" class:lit cx="24" cy="24" r="6" />
</svg>

<style>
  .mark { display: block; overflow: visible; }
  .ring {
    fill: none;
    stroke: currentColor;
    stroke-width: 4.5;
  }
  .cap { fill: currentColor; }
  .cap.lit { fill: var(--accent); }
</style>
