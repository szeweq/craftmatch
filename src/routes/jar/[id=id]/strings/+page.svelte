<script lang="ts">
  import EntryHeader from '$lib/EntryHeader.svelte';
import Paginator from '$lib/Paginator.svelte'
  import QInput from '$lib/QInput.svelte'
  import SortBtn from '$lib/SortBtn.svelte'
  import { queryable, paginate, sortable } from '$lib/data.svelte'
  let {data}: { data: import('./$types').PageData } = $props()
  const q = queryable(() => data?.strings ?? [], x => x[0])
  const sb = sortable(q, x => x[1].length)
  const pag = paginate(sb)
</script>
<EntryHeader {data} title="Strings" />
<section class="stick-top rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1">
  <QInput {...q} />
  <SortBtn label="Sort by count" bind:sort={sb.sortID} />
  <Paginator {pag} />
</section>
<ul class="text-xs px-1">{#each pag as [c, x] (c)}
  <li><details>
    <summary>{JSON.stringify(c)} ({x.length})</summary>
    <div class="ml-1 pl-3 b-0 b-l-2 b-solid b-white/40">{#each x as u}
      <div>{data.classes[u]}</div>
    {/each}</div>
  </details></li>
{/each}</ul>