<script lang="ts">
  import Paginator from '$lib/Paginator.svelte';
  import { filterBy, sortBy } from '$lib/query';
  const perPage = 40
  let {data}: { data: import('./$types').PageData } = $props()
  let q = $state("")
  let page = $state(0)
  let queried = $derived(filterBy(data.indices, q, ([s]) => s))
  let pages = $derived(Math.ceil(queried.length / perPage))
  let sortCount = $state(false)
  let sorted = $derived(sortBy(queried, sortCount, ([,j]) => data.inherits[j].length))
  let selected = $state(-1)
  let selectedString = $derived.by(() => selected >= 0 ? data.indices.find(([,i]) => i == selected)![0] : "")
  let selectedList = $derived.by(() => selected >= 0 ? data.inherits[selected].map(k => data.indices.find(([,j]) => j == k)![0]) : [])
  let dialog: HTMLDialogElement = $state()
  $effect(() => {
    if (selected >= 0) dialog?.showModal(); else dialog?.close()
  })
</script>
<div>
  <label class="input-group">
    <input type="text" bind:value={q} />
    <span>{queried.length}/{data.indices.length}</span>
  </label>
  <input id="sortCount" type="checkbox" bind:checked={sortCount} />
  <label for="sortCount">Sort by count</label>
</div>
<Paginator bind:page={page} count={pages} />
<ul class="text-xs">
  {#each sorted.slice(page * perPage, (page + 1) * perPage) as [s, i], j (i)}
    <li><a href="#" onclick={() => selected = i}>{s} ({data.inherits[i].length} inherited classes)</a></li>
  {/each}
</ul>
<dialog bind:this={dialog} class="max-w-full">
  {#if selected >= 0}
  <h2>{selectedString}</h2>
  <ul class="text-xs">
    {#each selectedList as c}
      <li>{c}</li>
    {/each}
  </ul>
  <button onclick={() => selected = -1}>Close</button>
  {/if}
</dialog>