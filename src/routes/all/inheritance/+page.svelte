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
</script>
<div>
    <input type="text" bind:value={q} />
    <span>Item count: {queried.length}</span>
    <input id="sortCount" type="checkbox" bind:checked={sortCount} />
    <label for="sortCount">Sort by count</label>
  </div>
  <Paginator bind:page={page} count={pages} />
  <ul class="text-xs">
    {#each sorted.slice(page * perPage, (page + 1) * perPage) as [s, i] (i)}
      <li>{s} ({data.inherits[i].length} inherited classes)</li>
    {/each}
  </ul>