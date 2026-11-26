<script lang="ts">
  import { app } from "$lib/app.svelte";
  import Range from "../Range.svelte";
  import { tr } from "$lib/i18n";

  const p = $derived(app.profile);

  const LABELS = ["motorLeft", "motorRight", "motorLt", "motorRt"] as const;

  function rumble() {
    if (!p) return;
    const [left, right] = p.fun.motors;
    app.vibrate(left, right, 100);
  }
</script>

{#if p}
  <section class="card">
    <div class="head">
      <span class="section-label">{tr("levels")}</span>
      <button class="primary" onclick={rumble}>{tr("rumble")}</button>
    </div>
    <p class="muted note">{tr("hintMotors")}</p>
    <fieldset class="levels" disabled={!app.writable}>
      {#each LABELS as key, i (key)}
        <Range label={tr(key)} bind:value={p.fun.motors[i]} max={4} onchange={() => app.saveProfile()} />
      {/each}
    </fieldset>
  </section>
{/if}

<style>
  .card {
    display: grid;
    gap: 16px;
    align-content: start;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .head .section-label {
    margin: 0;
  }
  .levels {
    display: grid;
    gap: 16px;
    min-width: 0;
    margin: 0;
    padding: 0;
    border: 0;
  }
</style>
