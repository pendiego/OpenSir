<script lang="ts">
  import { app } from "$lib/app.svelte";
  import ButtonTest from "$lib/components/views/ButtonTest.svelte";
  import { tr } from "$lib/i18n";
</script>

<!-- Standalone entry from the device list; with a device open the shell shows the same view. -->
<div class="page">
  <header class="bar" data-tauri-drag-region="deep">
    <a class="back" href="/">
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6" /></svg>
      {app.openId ? tr("overview") : tr("back")}
    </a>
    <h1>{tr("buttonTest")}</h1>
  </header>
  <main class="body">
    <ButtonTest />
  </main>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .bar {
    flex: none;
    display: flex;
    align-items: center;
    gap: 16px;
    height: 52px;
    /* Room for the window buttons drawn by the layout. */
    padding: 0 132px 0 24px;
  }
  .back {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 32px;
    padding: 0 8px 0 4px;
    border-radius: var(--r-ctl);
    color: var(--fg-2);
    font-size: 13px;
    text-decoration: none;
    transition: background-color 140ms var(--ease), color 140ms var(--ease);
  }
  .back:hover { background: var(--raised); color: var(--fg); }
  .back svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  h1 {
    margin: 0;
    font-size: 16px;
    font-weight: 500;
  }
  .body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 24px;
    max-width: 1200px;
    width: 100%;
    margin: 0 auto;
  }
</style>
