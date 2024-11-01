<script module>
  const dur = {duration: 200}
</script>
<script>
  import { navigating } from "$app/stores"
  import { fade } from "svelte/transition"
  import loadingSvg from "./loading.svg"

  let load = $state(false)
  $effect.pre(() => navigating.subscribe(n => load = !!n))
</script>
{#if load}
<div class="f justify-center fixed inset-0 left-[var(--s-aside)] z-99 darken" in:fade={dur} out:fade={dur}>
  <img class="m-4" src={loadingSvg} alt="Loading" width="32" height="32" />
</div>
{/if}