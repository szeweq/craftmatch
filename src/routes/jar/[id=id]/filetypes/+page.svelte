<script lang="ts">
  import EntryHeader from '$lib/EntryHeader.svelte'
  import { useUnitFmt, percentFmt } from '$lib/intl.svelte'
  let {data}: { data: import('./$types').PageData } = $props()
  let types = $derived(Object.keys(data.sizes))
  const sortFns: ((a: [number, number, number], b: [number, number, number]) => number)[] = [
    (a, b) => a[0] - b[0],
    (a, b) => a[1] - b[1],
    (a, b) => a[2] - b[2],
    (a, b) => a[2] / a[1] - b[2] / b[1],
    (a, b) => (a[2] - a[1]) - (b[2] - b[1]),
  ]
  let sortBy = $state(0)
  let orderAsc = $state(true)
  let typesSorted = $derived.by(() => {
    let as = [...types]
    as.sort(sortBy ? (ka, kb) => sortFns[sortBy-1](data.sizes[ka], data.sizes[kb]) : undefined)
    if (!orderAsc) as.reverse()
    return as
  })
  let all = $derived.by(() => {
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
  const headNames = ['Type', 'Count', 'Size', 'Compressed', 'Ratio', 'Saving']
</script>
{#snippet item(name, [count, size, compressed])}
  <td>{name}</td>
  <td>{count}</td>
  <td>{inKB(size / 1024)}</td>
  <td>{inKB(compressed / 1024)}</td>
  <td>{percentFmt(size && compressed / size)}</td>
  <td>{inKB((size - compressed) / 1024)}</td>
{/snippet}
<EntryHeader {data} title="File types" />
<table class="border-collapse w-full">
  <thead class="b-white/60 b-b-2 b-b-solid">
    <tr class="*:p-1 hover:*:bg-white/10 select-none">
      {#each headNames as h, i}
        <th class={sortBy == i ? orderAsc ? 'sort-asc' : 'sort-desc' : ''} onclick={() => switchSort(i)}>{h}</th>
      {/each}
    </tr>
  </thead>
  <tbody class="text-xs hover:*:bg-white/10">
    {#each typesSorted as t (t)}
      {@const d = data.sizes[t]}
      <tr class="*:p-1 [&>td:not(:first-child)]:text-end hover:*:bg-white/10">
        {@render item(t == "" ? "<empty>" : t, d)}
      </tr>
    {/each}
  </tbody>
  <tfoot class="b-t-2 b-t-solid b-white/40">
    <tr class="*:p-1 [&>td:not(:first-child)]:text-end hover:bg-white/10 hover:*:bg-white/10">
      {@render item("Total", all)}
    </tr>
  </tfoot>
</table>