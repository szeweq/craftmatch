<script lang="ts">
  import Paginator from '$lib/Paginator.svelte'
  import QInput from '$lib/QInput.svelte'
  import { useUnitFmt } from '$lib/intl.svelte'
  import { queryable, paginate, sortable } from '$lib/data.svelte'
  import SortBtn from '$lib/SortBtn.svelte'
  let {data}: { data: import('./$types').PageData } = $props()
  let oe = $derived(Object.entries(data))
  const q = queryable(() => oe, x => x[0])
  const sb = sortable(q, x => x[1])
  const pag = paginate(sb)
  let timeFmt = useUnitFmt('microsecond')
</script>
<h1>Debug – parsing times</h1>
<section class="sticky top-0 rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1">
  <QInput {...q} />
  <SortBtn label="Sort by time" bind:sort={sb.sortID} />
  <Paginator {pag} />
</section>
<ul class="text-xs">
  {#each pag as [k, v] (k)}
    <li>{k}: {timeFmt(1e6 * v)}</li>
  {/each}
</ul>