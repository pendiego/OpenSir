<script lang="ts">
  import type { MacroStep } from "$lib/model";
  import { tr } from "$lib/i18n";
  import { GROUPED_KEYS, GROUP_LABEL, keyLabel } from "./KeyPicker.svelte";

  /** The device accepts at most 30 macro steps (`profile.rs` MAX_STEPS). */
  const MAX_STEPS = 30;
  const DEFAULT_GAP_MS = 50;
  /** Wire fields are u16; an emptied number input binds null. */
  const u16 = (v: number | null) => Math.min(65535, Math.max(0, Math.round(v ?? 0)));

  let {
    steps = $bindable(),
    macroEnabled = $bindable(),
    macroPressToTrigger = $bindable(),
    macroCycle = $bindable(),
    cycleMs = $bindable(),
    onchange,
  }: {
    steps: MacroStep[];
    macroEnabled: boolean;
    macroPressToTrigger: boolean;
    macroCycle: boolean;
    cycleMs: number;
    onchange: () => void;
  } = $props();

  // The delay follows a step; the last step has nothing after it, so its delay is always 0.
  function commit() {
    for (const s of steps) {
      s.holdMs = u16(s.holdMs);
      s.delayMs = u16(s.delayMs);
    }
    const last = steps[steps.length - 1];
    if (last) last.delayMs = 0;
    onchange();
  }

  function commitCycle() {
    cycleMs = u16(cycleMs);
    onchange();
  }

  function addStep() {
    if (steps.length >= MAX_STEPS) return;
    const prev = steps[steps.length - 1];
    if (prev && prev.delayMs === 0) prev.delayMs = DEFAULT_GAP_MS;
    steps = [...steps, { key: 0, holdMs: DEFAULT_GAP_MS, delayMs: 0 }];
    commit();
  }

  function removeStep(i: number) {
    steps = steps.filter((_, j) => j !== i);
    commit();
  }

  function move(i: number, dir: -1 | 1) {
    const j = i + dir;
    if (j < 0 || j >= steps.length) return;
    const next = [...steps];
    [next[i], next[j]] = [next[j], next[i]];
    steps = next;
    commit();
  }
</script>

<section class="macro">
  <label class="field-row">
    <span>{tr("macroEnabled")}</span>
    <input type="checkbox" bind:checked={macroEnabled} onchange={onchange} />
  </label>
  <label class="field-row">
    <span>{tr("pressToTrigger")}</span>
    <input type="checkbox" bind:checked={macroPressToTrigger} onchange={onchange} />
  </label>
  <label class="field-row">
    <span>{tr("loop")}</span>
    <input type="checkbox" bind:checked={macroCycle} onchange={onchange} />
  </label>
  <label class="field-row">
    <span>{tr("cycleTime")}</span>
    <input type="number" min="0" max="65535" bind:value={cycleMs} onchange={commitCycle} />
  </label>

  <p class="muted">{tr("macroUnitsHint")}</p>
  <ol class="steps">
    {#each steps as step, i (i)}
      <li class="step">
        <div class="order">
          <button type="button" aria-label={`${tr("moveUpStep")} ${i + 1}`} disabled={i === 0} onclick={() => move(i, -1)}>↑</button>
          <button type="button" aria-label={`${tr("moveDownStep")} ${i + 1}`} disabled={i === steps.length - 1} onclick={() => move(i, 1)}>↓</button>
        </div>
        <label class="cell">
          <span>{tr("stepKey", { n: i + 1 })}</span>
          <select bind:value={step.key} onchange={commit}>
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
        <label class="cell">
          <span>{tr("hold")}</span>
          <input type="number" min="0" max="65535" bind:value={step.holdMs} onchange={commit} />
        </label>
        {#if i < steps.length - 1}
          <label class="cell">
            <span>{tr("delay")}</span>
            <input type="number" min="0" max="65535" bind:value={step.delayMs} onchange={commit} />
          </label>
        {:else}
          <p class="cell">
            <span>{tr("delay")}</span>
            <small>{tr("lastStepNoDelay")}</small>
          </p>
        {/if}
        <button type="button" aria-label={`${tr("removeStep")} ${i + 1}`} onclick={() => removeStep(i)}>{tr("removeStep")}</button>
      </li>
    {/each}
  </ol>

  <div class="foot">
    <button type="button" onclick={addStep} disabled={steps.length >= MAX_STEPS}>
      {tr("addStep")}
    </button>
    <span class="muted">{tr("stepsUsed", { n: steps.length, max: MAX_STEPS })}</span>
  </div>
</section>

<style>
  .macro {
    display: grid;
    gap: 4px;
  }
  .field-row + .field-row {
    border-top: 1px solid var(--line);
  }
  .steps {
    display: grid;
    margin: 8px 0 0;
    padding: 0;
    list-style: none;
  }
  .step {
    display: grid;
    grid-template-columns: auto minmax(0, 2fr) minmax(0, 1fr) minmax(0, 1fr) auto;
    gap: 8px;
    align-items: end;
    padding: 8px 0;
    border-top: 1px solid var(--line);
  }
  .order {
    display: flex;
    gap: 4px;
  }
  .cell {
    display: grid;
    gap: 4px;
    min-width: 0;
    margin: 0;
    font-size: 12px;
    color: var(--fg-2);
  }
  .cell select,
  .cell input {
    width: 100%;
    min-width: 0;
  }
  .foot {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 8px;
  }
</style>
