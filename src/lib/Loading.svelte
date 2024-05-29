<script>
  import { navigating } from "$app/stores"
  import { fade } from "svelte/transition"
  import loadingSvg from "./loading.svg"

  let load = $state(false)
  $effect.pre(() => navigating.subscribe(n => load = !!n))
</script>
{#if load}
<div class="fixed inset-0 left-[var(--s-aside)] z-99 darken" in:fade={{duration: 200}} out:fade={{duration: 200}}>
  <div class="f items-center justify-center text-2xl p-1 m-4">
    <img src={loadingSvg} alt="Loading" width="32" height="32" />
  </div>
</div>
{/if}