<script lang="ts">
  import type { Paginate } from "./data.svelte";

  let {pag}: {pag: Paginate<any>} = $props()
  let around = $derived.by(() => Array.from({length: pag.all}, (_, i) => i).filter(i => i >= pag.current - 4 && i <= pag.current + 4))
  let navElem = $state<HTMLElement>()
  $effect(() => navElem?.addEventListener("wheel", e => {
    e.preventDefault()
    pag.current = Math.min(pag.all - 1, Math.max(0, pag.current + Math.sign(e.deltaY) * (e.shiftKey ? 10 : 1)))
  }, {passive: false}))
</script>
<nav bind:this={navElem} class="pag f items-center justify-center gap-0.5">
  {#if pag.current > 0}
    <button class="p-prev before:i-ms-arrow-back" title="Previous page" aria-label="Previous" onclick={() => pag.current -= 1}></button>
  {/if}
  {#each around as p (p)}
    <button class:active={p === pag.current} onclick={() => pag.current = p}>{p + 1}</button>
  {/each}
  {#if pag.current < pag.all - 1}
    <button class="p-next before:i-ms-arrow-forward" title="Next page" aria-label="Next" onclick={() => pag.current += 1}></button>
  {/if}
</nav>