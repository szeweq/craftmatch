<script lang="ts">
  import Paginator from '$lib/Paginator.svelte'
  import QInput from '$lib/QInput.svelte'
    import SortBtn from '$lib/SortBtn.svelte';
  import { paginate } from '$lib/paginate.svelte'
  import { sortBy } from '$lib/query'
  import { queryable } from '$lib/queryable.svelte'
  let {data}: { data: import('./$types').PageData } = $props()
  let q = queryable(() => data?.strings ?? [], x => x[0])
  let sortCount = $state(0)
  let sorted = $derived(sortBy(q.queried, sortCount && (([,j]) => j.length), sortCount > 1))
  let pag = paginate(() => sorted)
</script>
<section class="sticky top-0 rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1">
  <QInput {...q} />
  <SortBtn label="Sort by count" bind:sort={sortCount} />
  <Paginator {pag} />
</section>
<ul class="text-xs list-none px-1">{#each pag.chunk as [c, x] (c)}
  <li><details>
    <summary>{JSON.stringify(c)} ({x.length})</summary>
    <div class="ml-1 pl-3 b-0 b-l-2 b-solid b-white/40">{#each x as u}
      <div>{data.classes[u]}</div>
    {/each}</div>
  </details></li>
{/each}</ul>