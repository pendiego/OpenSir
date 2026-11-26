<script lang="ts">
  import { tr } from "$lib/i18n";
  import Range from "./Range.svelte";

  // The device caps at 42; the UI stops at 30.
  const MAX_TURBO = 30;

  let {
    turbo = $bindable(),
    toggleOnPress = $bindable(),
    onchange,
  }: {
    turbo: number;
    toggleOnPress: boolean;
    onchange: () => void;
  } = $props();
</script>

<div class="turbo-toggle">
  <Range
    label={tr("turbo")}
    bind:value={turbo}
    max={MAX_TURBO}
    unit="Hz"
    format={(v) => (v === 0 ? tr("turboOff") : tr("turboValue", { n: v }))}
    {onchange}
  />
  <p class="muted hint">{tr("turboHint")}</p>
  <label class="field-row">
    <span>{tr("toggleOnPress")}</span>
    <input type="checkbox" role="switch" bind:checked={toggleOnPress} onchange={onchange} />
  </label>
  <p class="muted hint">{tr("toggleOnPressHint")}</p>
</div>

<style>
  .turbo-toggle {
    display: grid;
    gap: 4px;
  }
</style>
