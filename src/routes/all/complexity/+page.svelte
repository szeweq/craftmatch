<script lang="ts">
  import Paginator from '$lib/Paginator.svelte'
  import QInput from '$lib/QInput.svelte'
    import SortBtn from '$lib/SortBtn.svelte'
  import { sortBy } from '$lib/query'
  import { queryable, paginate } from '$lib/data.svelte'
  let {data}: { data: import('./$types').PageData } = $props()
  let complx = $derived(Object.entries(data))
  const q = queryable(() => complx, x => x[0])
  let sortCount = $state(0)
  let sorted = $derived(sortBy(q.queried, sortCount && (x => x[1].total), sortCount > 1))
  let pag = paginate(() => sorted)
</script>
<section class="sticky top-0 rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1">
  <QInput {...q} />
  <SortBtn label="Sort by count" bind:sort={sortCount} />
  <Paginator {pag} />
</section>
<ul class="text-xs list-none px-1">
  {#each pag.chunk as [k, v] (k)}
    <li><details>
      <summary>{k} ({v.total})</summary>
      <div class="ml-1 pl-3 b-0 b-l-2 b-solid b-white/40">{#each v.code as [c, j] (c)}
        <div>{c}: {j}</div>
      {/each}</div>
    </details></li>
  {/each}
</ul>