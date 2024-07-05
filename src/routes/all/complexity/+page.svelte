<script lang="ts">
  import Paginator from '$lib/Paginator.svelte'
  import QInput from '$lib/QInput.svelte'
  import SortBtn from '$lib/SortBtn.svelte'
  import { queryable, paginate, sortable } from '$lib/data.svelte'
  let {data}: { data: import('./$types').PageData } = $props()
  let complx = $derived(Object.entries(data))
  const q = queryable(() => complx, x => x[0])
  const sb = sortable(q, x => x[1].total)
  const pag = paginate(sb)
</script>
<section class="stick-top rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1">
  <QInput {...q} />
  <SortBtn label="Sort by count" bind:sort={sb.sortID} />
  <Paginator {pag} />
</section>
<ul class="text-xs list-none px-1">
  {#each pag as [k, v] (k)}
    <li><details>
      <summary>{k} ({v.total})</summary>
      <div class="ml-1 pl-3 b-0 b-l-2 b-solid b-white/40">{#each v.code as [c, j] (c)}
        <div>{c}: {j}</div>
      {/each}</div>
    </details></li>
  {/each}
</ul>