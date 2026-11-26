<script module lang="ts">
  import { KEY_CODES } from "$lib/keycodes";
  import { tr, type Key } from "$lib/i18n";

  export type KeyGroup = "gamepad" | "keyboard" | "mouse" | "media" | "other";

  export const GROUP_LABEL: Record<KeyGroup, Key> = {
    gamepad: "groupGamepad",
    keyboard: "groupKeyboard",
    mouse: "groupMouse",
    media: "groupMedia",
    other: "groupOther",
  };

  const MEDIA = new Set(["Mute", "KB_NUM_PLAY", "KB_NUM_STOP", "KB_NUM_NEXT", "KB_NUM_PREV", "KB_NUM_VOL_UP", "KB_NUM_VOL_DOWN", "KB_NUM_MUTE"]);

  /** Group of a device key name (see `keycodes.ts`). */
  export function groupOf(name: string): KeyGroup {
    if (name === "NC" || name === "Shift") return "other";
    if (name.startsWith("MS_")) return "mouse";
    if (MEDIA.has(name)) return "media";
    if (name.startsWith("KB_")) return "keyboard";
    return "gamepad";
  }

  /** Device names that read as words in the UI: translated, never shown raw. */
  const WORD: Record<string, Key> = {
    NC: "none", PS: "spotHome", Sel: "spotView", Sta: "spotMenu", Camera: "spotShare", FL1: "spotPaddleL", FR1: "spotPaddleR",
    KB_SPACE: "keySpace",
    KB_NUM_PLAY: "mediaPlay", KB_NUM_STOP: "mediaStop", KB_NUM_NEXT: "mediaNext", KB_NUM_PREV: "mediaPrev",
    KB_NUM_VOL_UP: "mediaVolUp", KB_NUM_VOL_DOWN: "mediaVolDown", KB_NUM_MUTE: "mediaMute",
    MS_LEFT_CLICK: "mouseLeft", MS_MID_CLICK: "mouseMiddle", MS_RIGHT_CLICK: "mouseRight",
    MS_BUTTON4: "mouseButton4", MS_BUTTON5: "mouseButton5", MS_WHEEL_UP: "mouseWheelUp", MS_WHEEL_DOWN: "mouseWheelDown",
  };

  const PAD: Record<string, string> = {
    CRO: "A", CIR: "B", SQU: "X", TRI: "Y", L1: "LB", R1: "RB", L2: "LT", R2: "RT", L3: "LS", R3: "RS",
    Up: "D-pad ↑", Under: "D-pad ↓", Left: "D-pad ←", Right: "D-pad →",
    LSUp: "LS ↑", LSUnder: "LS ↓", LSLeft: "LS ←", LSRight: "LS →",
    RSUp: "RS ↑", RSUnder: "RS ↓", RSLeft: "RS ←", RSRight: "RS →",
  };
  // Keycap legends. The vendor prefixes the navigation cluster with NUM_ (KB_NUM_INSERT is plain Insert,
  // docs/protocol/cyclone2-keycodes.md), so only NUM_LOCK, the operators and the digits are numpad keys.
  const KB: Record<string, string> = {
    ESC: "Esc", BACKQUOTE: "`", DEC: "-", ADD: "=", BACKSPACE: "Backspace", TAB: "Tab", LBRACKETS: "[", RBRACKETS: "]",
    BACKSLASH: "\\", CAPSLOCK: "Caps Lock", SEMI: ";", QUOTE: "'", ENTER: "Enter", COMMA: ",", DOT: ".", SLASH: "/",
    LSHIFT: "L Shift", RSHIFT: "R Shift", LCTRL: "L Ctrl", RCTRL: "R Ctrl", LALT: "L Alt", RALT: "R Alt", WIN: "Win",
    NUM_INSERT: "Insert", NUM_HOME: "Home", NUM_PGUP: "Page Up", NUM_DELETE: "Delete", NUM_END: "End", NUM_PGDN: "Page Down",
    NUM_UP: "↑", NUM_DOWN: "↓", NUM_LEFT: "←", NUM_RIGHT: "→",
    NUM_LOCK: "Num Lock", NUM_DIV: "Num /", NUM_MUL: "Num *", NUM_DEC: "Num -", NUM_ADD: "Num +", NUM_DOT: "Num .", NUM_ENTER: "Num Enter",
  };
  /** Readable name for a device key name: `KB_LCTRL` -> `L Ctrl`, `KB_NUM_7` -> `Num 7`, `MS_WHEEL_UP` -> translated. */
  export function keyLabel(name: string): string {
    if (name in WORD) return tr(WORD[name]);
    if (name in PAD) return PAD[name];
    if (!name.startsWith("KB_")) return name;
    const k = name.slice(3);
    if (k in KB) return KB[k];
    if (k.startsWith("NUM_")) return `Num ${k.slice(4)}`;
    return k;
  }

  const ORDER: KeyGroup[] = ["gamepad", "keyboard", "mouse", "media", "other"];

  /** Every device key, grouped in picker order (computed once). */
  export const GROUPED_KEYS: { group: KeyGroup; keys: (readonly [number, string])[] }[] = ORDER.map((group) => ({
    group,
    keys: KEY_CODES.filter(([, name]) => groupOf(name) === group),
  })).filter((g) => g.keys.length > 0);
</script>

<script lang="ts">
  import { app } from "$lib/app.svelte";

  /** The device stores at most three mapped keys per button (`map: [u8; 3]`). */
  const MAX_KEYS = 3;

  let {
    selected = $bindable(),
    onchange,
  }: {
    selected: number[];
    onchange: () => void;
  } = $props();

  let query = $state("");

  const visible = $derived.by(() => {
    const q = query.trim().toLowerCase();
    // Restricted PIDs map to gamepad keys only (no keyboard, mouse or media targets).
    const groups = app.restricted ? GROUPED_KEYS.filter((g) => g.group === "gamepad" || g.group === "other") : GROUPED_KEYS;
    return groups.map(({ group, keys }) => ({
      group,
      keys: keys.filter(([, name]) => !q || keyLabel(name).toLowerCase().includes(q) || name.toLowerCase().includes(q)),
    })).filter((g) => g.keys.length > 0);
  });
  const found = $derived(visible.reduce((n, g) => n + g.keys.length, 0));

  function toggle(code: number) {
    selected = selected.includes(code) ? selected.filter((c) => c !== code) : [...selected, code];
    onchange();
  }
</script>

<div class="picker">
  <div class="head">
    <input type="search" bind:value={query} placeholder={tr("keySearch")} aria-label={tr("keySearch")} />
    {#if query.trim() && found > 0}
      <span class="muted">{tr("keyResults", { n: found })}</span>
    {/if}
  </div>
  {#if selected.length >= MAX_KEYS}
    <p class="hint">{tr("keyMax", { n: MAX_KEYS })}</p>
  {/if}

  <div class="groups" role="group" aria-label={tr("keyPicker")}>
    {#each visible as { group, keys } (group)}
      <section>
        <h4 class="section-label">{tr(GROUP_LABEL[group])}</h4>
        <div class="grid">
          {#each keys as [code, name] (code)}
            {@const on = selected.includes(code)}
            <button
              type="button"
              class="key"
              class:active={on}
              aria-pressed={on}
              disabled={!on && selected.length >= MAX_KEYS}
              onclick={() => toggle(code)}
            >
              {keyLabel(name)}
            </button>
          {/each}
        </div>
      </section>
    {/each}
  </div>
  {#if found === 0}
    <p class="muted">{tr("keyNoMatch", { q: query.trim() })}</p>
  {/if}
</div>

<style>
  .picker {
    display: grid;
    gap: 12px;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .head input {
    flex: 1;
    min-width: 0;
    appearance: none;
    min-height: 32px;
    padding: 4px 10px;
    font: inherit;
    font-size: 13px;
    color: var(--fg);
    background: var(--raised);
    border: 1px solid var(--line);
    border-radius: var(--r-ctl);
  }
  .head .muted {
    white-space: nowrap;
  }
  .groups {
    display: grid;
    gap: 16px;
  }
  .section-label {
    margin: 0 0 6px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(84px, 1fr));
    gap: 6px;
  }
  .key {
    min-height: 32px;
    padding: 4px 8px;
    font-size: 13px;
  }
</style>
