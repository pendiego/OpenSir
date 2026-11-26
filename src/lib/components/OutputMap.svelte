<script lang="ts">
  import Range from "./Range.svelte";
  import { tr } from "$lib/i18n";
  import type { MapModule } from "$lib/model";
  import type { Key } from "$lib/i18n";
  import { GROUPED_KEYS, GROUP_LABEL, keyLabel } from "./KeyPicker.svelte";

  let {
    map,
    withZ = false,
    onchange,
  }: {
    map: MapModule;
    /** V2 motion map module has a third (Z) flip. */
    withZ?: boolean;
    onchange: () => void;
  } = $props();

  const OUTPUTS: [number, string][] = [
    [1, "outLeft"],
    [2, "outRight"],
    [3, "outWheel"],
    [4, "outMouse"],
  ];

  type WheelField = "mapUp" | "mapDown" | "mapLeft" | "mapRight" | "mapDead";
  const WHEEL: [WheelField, Key][] = [
    ["mapUp", "wheelUp"],
    ["mapDown", "wheelUnder"],
    ["mapLeft", "wheelLeft"],
    ["mapRight", "wheelRight"],
    ["mapDead", "wheelDead"],
  ];
</script>

<div class="output">
  <label class="field-row">
    <span>{tr("output")}</span>
    <select bind:value={map.mapIndex} onchange={onchange}>
      {#each OUTPUTS as [v, key] (v)}
        <option value={v}>{tr(key as "outLeft")}</option>
      {/each}
    </select>
  </label>

  <label class="field-row">
    <span>{tr("flipX")}</span>
    <input type="checkbox" bind:checked={map.xFlip} onchange={onchange} />
  </label>
  <label class="field-row">
    <span>{tr("flipY")}</span>
    <input type="checkbox" bind:checked={map.yFlip} onchange={onchange} />
  </label>
  {#if withZ}
    <label class="field-row">
      <span>{tr("zFlip")}</span>
      <input type="checkbox" bind:checked={map.zFlip} onchange={onchange} />
    </label>
  {/if}

  <Range label={tr("axisRatio")} bind:value={map.axisRatio} max={100} {onchange} />
  <Range label={tr("cross")} bind:value={map.mapCross} max={100} {onchange} />
  {#if map.mapIndex === 4}
    <Range label={tr("dpi")} bind:value={map.mouseDpi} max={100} {onchange} />
  {/if}

  {#if map.mapIndex === 3}
    <div class="wheel">
      {#each WHEEL as [field, label] (field)}
        <label class="field-row">
          <span>{tr(label)}</span>
          <select
            value={map[field]}
            onchange={(e) => {
              map[field] = Number(e.currentTarget.value);
              onchange();
            }}
          >
            <option value={0}>{tr("none")}</option>
            {#each GROUPED_KEYS as { group, keys } (group)}
              <optgroup label={tr(GROUP_LABEL[group])}>
                {#each keys as [code, name] (code)}
                  <option value={code}>{keyLabel(name)}</option>
                {/each}
              </optgroup>
            {/each}
          </select>
        </label>
      {/each}
    </div>
  {/if}
</div>

<style>
  .output {
    display: grid;
    gap: 12px;
  }
  .wheel {
    display: grid;
    gap: 4px;
  }
</style>
