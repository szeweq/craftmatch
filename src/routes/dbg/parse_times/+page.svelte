<script lang="ts">
  import Paginator from '$lib/Paginator.svelte'
  import { filterBy, sortBy } from '$lib/query'
  const perPage = 40
  let {data}: { data: import('./$types').PageData } = $props()
  let oe = $derived(Object.entries(data))
  let q = $state("")
  let page = $state(0)
  let queried = $derived(filterBy(oe, q, ([s]) => s))
  let pages = $derived(Math.ceil(queried.length / perPage))
  let sortCount = $state(false)
  let sorted = $derived(sortBy(queried, sortCount, ([,j]) => j))
  const timeFmt = new Intl.NumberFormat('en', {style: 'unit', unit: 'microsecond', unitDisplay: 'short'})
</script>
<h1>Debug – parsing times</h1>
<section class="sticky top-0 rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1">
  <input type="text" bind:value={q} />
  <input id="sortCount" type="checkbox" bind:checked={sortCount} />
  <label for="sortCount">Sort by time</label>
  <Paginator bind:page={page} count={pages} />
</section>
<ul class="text-xs">
  {#each sorted.slice(page * perPage, (page + 1) * perPage) as [k, v] (k)}
    <li>{k}: {timeFmt.format(1e6 * v)}</li>
  {/each}
</ul>