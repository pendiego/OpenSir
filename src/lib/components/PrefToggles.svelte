<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { tr } from "$lib/i18n";
  import { withViewTransition } from "$lib/motion";
</script>

<div class="prefs">
  <button
    class="pref"
    title={tr(app.theme === "dark" ? "light" : "dark")}
    aria-label={tr(app.theme === "dark" ? "light" : "dark")}
    onclick={() => void withViewTransition(() => (app.theme = app.theme === "dark" ? "light" : "dark"))}
  >
    {#if app.theme === "dark"}
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
        <circle cx="12" cy="12" r="4" />
        <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
      </svg>
    {:else}
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" aria-hidden="true">
        <path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5Z" />
      </svg>
    {/if}
  </button>
  <button
    class="pref lang"
    title={tr("language")}
    aria-label={tr("language")}
    onclick={() => (app.locale = app.locale === "en" ? "pt-BR" : "en")}
  >
    {app.locale === "en" ? "PT" : "EN"}
  </button>
</div>

<style>
  .prefs { display: flex; align-items: center; gap: 4px; }
  .pref {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 32px;
    height: 32px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--r-ctl);
    background: transparent;
    color: var(--fg-2);
    font: 12px var(--font);
    cursor: pointer;
    transition: background-color 150ms var(--ease), color 150ms var(--ease);
  }
  .pref:hover { background: var(--hover); color: var(--fg); }
  .pref:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
  @media (prefers-reduced-motion: reduce) { .pref { transition: none; } }
</style>
