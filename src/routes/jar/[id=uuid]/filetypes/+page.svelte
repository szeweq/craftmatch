<script lang="ts">
  import { useUnitFmt, percentFmt } from '$lib/intl.svelte';
  let {data}: { data: import('./$types').PageData } = $props()
  let types = $derived(Object.keys(data.sizes))
  const sortFns: ((a: [number, number, number], b: [number, number, number]) => number)[] = [
    (a, b) => b[0] - a[0],
    (a, b) => b[1] - a[1],
    (a, b) => b[2] - a[2],
    (a, b) => b[2] / b[1] - a[2] / a[1]
  ]
  let sortBy = $state(0)
  let orderAsc = $state(true)
  let typesSorted = $derived.by(() => {
    let as = [...types]
    as.sort(sortBy ? (ka, kb) => sortFns[sortBy-1](data.sizes[ka], data.sizes[kb]) : undefined)
    if (!orderAsc) as.reverse()
    return as
  })
  let allSizes = $derived.by(() => {
    let as: [number, number, number] = [0, 0, 0]
    for (const [, j] of Object.entries(data.sizes)) j.forEach((s, i) => as[i] += s)
    return as
  })
  let inKB = useUnitFmt('kilobyte', 2)
  const switchSort = (i: number) => {
    if (i == sortBy) orderAsc = !orderAsc
    else {
      sortBy = i, orderAsc = true
    }
  }
  const headNames = ['Type', 'Count', 'Internal size', 'External size', 'Compression ratio']
</script>
<h1>File types</h1>
<div>{data.name}</div>
<table class="border-collapse w-full">
  <thead>
    <tr class="b-white/60 b-b-2 b-b-solid *:p-1 hover:*:bg-white/10 select-none">
      {#each headNames as h, i}
        <th class={sortBy == i ? orderAsc ? 'after:content-["▲"]' : 'after:content-["▼"]' : ''} onclick={() => switchSort(i)}>{h}</th>
      {/each}
    </tr>
  </thead>
  <tbody class="text-sm">
    {#each typesSorted as t}
      {@const d = data.sizes[t]}
      <tr class="*:p-1 [&>td:not(:first-child)]:text-end hover:bg-white/10 hover:*:bg-white/10">
        <td>{t == "" ? "<empty>" : t}</td>
        <td>{d[0]}</td>
        <td>{inKB(d[1] / 1024)}</td>
        <td>{inKB(d[2] / 1024)}</td>
        <td>{percentFmt(d[1] && d[2] / d[1])}</td>
      </tr>
    {/each}
  </tbody>
</table>