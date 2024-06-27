<script lang="ts">
  import Paginator from '$lib/Paginator.svelte'
  import QInput from '$lib/QInput.svelte'
    import SortBtn from '$lib/SortBtn.svelte'
  import { queryable, paginate, sortable } from '$lib/data.svelte'
  let {data}: { data: import('./$types').PageData } = $props()
  let q = queryable(() => data.indices, x => x[0])
  let sb = sortable(() => q.queried, ([,j]) => data.inherits[j].length)
  let pag = paginate(() => sb.sorted)
  let selected = $state(-1)
  let selObj = $derived.by(() => ({
    str: selected >= 0 ? data.indices.find(([,i]) => i == selected)![0] : "",
    list: selected >= 0 ? data.inherits[selected].map(k => data.indices.find(([,j]) => j == k)![0]) : []
  }))
  let dialog: HTMLDialogElement = $state()
  $effect(() => {
    if (selected >= 0) dialog?.showModal(); else dialog?.close()
  })
</script>
<div>
  <QInput {...q} />
  <SortBtn label="Sort by count" bind:sort={sb.sortID} />
</div>
<Paginator {pag} />
<ul class="text-xs">
  {#each pag as [s, i] (i)}
    <li><a href="#" onclick={() => selected = i}>{s} ({data.inherits[i].length} inherited classes)</a></li>
  {/each}
</ul>
<dialog bind:this={dialog} class="max-w-full">
  {#if selected >= 0}
  <h2>{selObj.str}</h2>
  <ul class="text-xs">
    {#each selObj.list as c}
      <li>{c}</li>
    {/each}
  </ul>
  <button onclick={() => selected = -1}>Close</button>
  {/if}
</dialog>