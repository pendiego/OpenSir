<script lang="ts">
  import { app } from "$lib/app.svelte";
  import Controller from "../Controller.svelte";
  import MapEditor from "../MapEditor.svelte";
  import { KEYS, keyOf, type KeyTarget } from "$lib/keys";
  import { tr } from "$lib/i18n";

  let selected = $state<KeyTarget>(KEYS[0].target);

  const p = $derived(app.profile);
  const data = $derived(p ? keyOf(p, selected) : null);
  const spot = $derived(KEYS.find((k) => JSON.stringify(k.target) === JSON.stringify(selected)));
  const label = $derived(
    spot ? (spot.name ? tr(spot.name) : spot.label)
    : selected.kind === "extra" ? app.model.extraKeys[selected.index]
    : selected.kind === "touch" ? `${tr("touchKey")} ${selected.index ? "R" : "L"}`
    : "",
  );
</script>

{#if p}
  <div class="mapping fill">
    <section class="card stage">
      <Controller profile={p} model={app.model} {selected} onselect={(t) => (selected = t)} />
      <p class="muted">{tr("mapHint")}</p>
    </section>
    <aside class="card inspector">
      {#if data}
        <MapEditor {data} {label} onchange={() => app.saveProfile()} />
      {/if}
    </aside>
  </div>
{/if}

<style>
  /* Fills the panel (DeviceView .page:has(.fill)): the drawing stays in view and only the inspector scrolls. */
  .mapping {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(280px, 340px);
    grid-template-rows: minmax(0, 1fr);
    gap: 16px;
  }
  .stage {
    display: grid;
    grid-template-rows: minmax(0, 1fr) auto;
    gap: 12px;
    min-width: 0;
    min-height: 0;
  }
  /* The drawing shrinks to the height it gets (viewBox keeps its proportions). */
  .stage :global(svg.pad) {
    height: 100%;
    min-height: 0;
  }
  .inspector {
    min-height: 0;
    overflow: auto;
  }
  /* Narrow and tall: drawing above, inspector below, each half the height. */
  @media (max-width: 959px) and (min-height: 600px) {
    .mapping {
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: minmax(0, 1fr) minmax(0, 1fr);
    }
  }
  /* Narrow and short (zoomed in): stacking would leave a sliver for each, so stay side by side. */
  @media (max-width: 959px) and (max-height: 599px) {
    .mapping {
      grid-template-columns: minmax(0, 1fr) minmax(200px, 45%);
    }
  }
  /* Short: the drawing needs every pixel; the hint stays for screen readers. */
  @media (max-height: 599px) {
    .stage p {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip-path: inset(50%);
    }
  }
</style>
