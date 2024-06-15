<script lang="ts">
  import Paginator from '$lib/Paginator.svelte';
  import QInput from '$lib/QInput.svelte';
  import { sortBy } from '$lib/query';
  import { queryable } from '$lib/queryable.svelte';
  const perPage = 40
  let {data}: { data: import('./$types').PageData } = $props()
  let complx = $derived(Object.entries(data))
  const q = queryable(() => complx, x => x[0])
  let pages = $derived(Math.ceil(q.queried.length / perPage))
  let page = $state(0)
  let sortCount = $state(false)
  let sorted = $derived(sortBy(q.queried, sortCount && (x => x[1].total)))
</script>
<section class="sticky top-0 rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1">
  <QInput {q} />
  <input id="sortCount" type="checkbox" bind:checked={sortCount} />
  <label for="sortCount">Sort by count</label>
  <Paginator bind:page={page} count={pages} />
</section>
<ul class="text-xs list-none px-1">
  {#each sorted.slice(page * perPage, (page + 1) * perPage) as [k, v] (k)}
    <li><details>
      <summary>{k} ({v.total})</summary>
      <div class="ml-1 pl-3 b-0 b-l-2 b-solid b-white/40">{#each v.code as [c, j] (c)}
        <div>{c}: {j}</div>
      {/each}</div>
    </details></li>
  {/each}
</ul>