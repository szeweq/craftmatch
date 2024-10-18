<script lang="ts">
  import Paginator from '$lib/Paginator.svelte'
  import QInput from '$lib/QInput.svelte'
  import SortBtn from '$lib/SortBtn.svelte'
  import { queryable, paginate, sortable } from '$lib/data.svelte'
  let {data}: { data: import('./$types').PageData } = $props()
  const hashSelect = () => {
    const hash = location.hash.slice(1)
    selected = !hash || hash == "#" ? -1 : parseInt(hash)
  }
  const q = queryable(() => data.indices, x => x[0])
  const sb = sortable(q, ([,j]) => data.inherits[j].length)
  const pag = paginate(sb)
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
<svelte:window on:hashchange={hashSelect} />
<div>
  <QInput {...q} />
  <SortBtn label="Sort by count" bind:sort={sb.sortID} />
</div>
<Paginator {pag} />
<ul class="text-xs">
  {#each pag as [s, i] (i)}
    <li><a href={'#' + i}>{s} ({data.inherits[i].length} inherited classes)</a></li>
  {/each}
</ul>
<dialog bind:this={dialog} class="max-w-full">
  {#if selected >= 0}
  <h3>{selObj.str}</h3>
  <ul class="text-xs">
    {#each selObj.list as c}
      <li>{c}</li>
    {/each}
  </ul>
  <a role="button" href="##">Close</a>
  {/if}
</dialog>