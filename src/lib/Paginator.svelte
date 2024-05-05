<script lang="ts">
  let {page = $bindable(), count}: {page: number, count: number} = $props()
  let around = $derived.by(() => Array.from({length: count}, (_, i) => i).filter(i => i >= page - 4 && i <= page + 4))
  let navElem = $state<HTMLElement>()
  $effect(() => navElem?.addEventListener("wheel", e => {
    e.preventDefault()
    page = Math.min(count - 1, Math.max(0, page + Math.sign(e.deltaY) * (e.shiftKey ? 10 : 1)))
  }, {passive: false}))
</script>
<nav bind:this={navElem} class="pag f items-center justify-center gap-0.5">
  {#if page > 0}
    <button class="p-prev before:i-ms-arrow-back" onclick={() => page -= 1}></button>
  {/if}
  {#each around as p (p)}
    <button class:active={p === page} onclick={() => page = p}>{p + 1}</button>
  {/each}
  {#if page < count - 1}
    <button class="p-next before:i-ms-arrow-forward" onclick={() => page += 1}></button>
  {/if}
</nav>