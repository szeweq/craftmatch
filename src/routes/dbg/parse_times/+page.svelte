<script lang="ts">
  import Paginator from '$lib/Paginator.svelte'
  import QInput from '$lib/QInput.svelte';
  import { useUnitFmt } from '$lib/intl.svelte';
  import { sortBy } from '$lib/query'
  import { queryable } from '$lib/queryable.svelte';
  const perPage = 40
  let {data}: { data: import('./$types').PageData } = $props()
  let oe = $derived(Object.entries(data))
  let q = queryable(() => oe, x => x[0])
  let page = $state(0)
  let pages = $derived(Math.ceil(q.queried.length / perPage))
  let sortCount = $state(false)
  let sorted = $derived(sortBy(q.queried, sortCount && (x => x[1])))
  let timeFmt = useUnitFmt('microsecond')
</script>
<h1>Debug – parsing times</h1>
<section class="sticky top-0 rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1">
  <QInput {q} />
  <input id="sortCount" type="checkbox" bind:checked={sortCount} />
  <label for="sortCount">Sort by time</label>
  <Paginator bind:page={page} count={pages} />
</section>
<ul class="text-xs">
  {#each sorted.slice(page * perPage, (page + 1) * perPage) as [k, v] (k)}
    <li>{k}: {timeFmt(1e6 * v)}</li>
  {/each}
</ul>