<script lang="ts">
  import type { ModelInfo } from "$lib/models";

  let { model, ready = false }: { model: ModelInfo; ready?: boolean } = $props();
</script>

<img class="thumb" class:ready class:mono={model.pad.mono} src={model.pad.cover} alt="" />

<style>
  .thumb {
    display: block;
    width: 100%;
    height: auto;
    opacity: 0.55;
    filter: grayscale(1);
  }
  .ready {
    opacity: 1;
    filter: none;
  }
  .mono {
    filter: grayscale(1) contrast(0.8);
  }
</style>
