<script lang="ts">
  import { useUnitFmt, percentFmt } from '$lib/intl.svelte';
  import type { ContentTypes } from '$lib/ws';

  let {data}: { data: import('./$types').PageData } = $props()
  let allSizes = $derived.by(() => {
    let as: [number, number, number] = [0, 0, 0]
    for (const [, j] of Object.entries(data.sizes)) j.forEach((s, i) => as[i] += s)
    return as
  })
  const types: [ContentTypes, string, string][] = [
    ['meta', 'bg-blue-500', 'Metadata'],
    ['classes', 'bg-green-500', 'Class files'],
    ['assets', 'bg-yellow-500', 'Assets'],
    ['data', 'bg-purple-500', 'Data'],
    ['other', 'bg-gray-500', 'Other']
  ]
  let by = $state(0)
  const byType = ['By count', 'By size (uncompressed)', 'By size (compressed)', 'By compression ratio']
  function typeData(n: [number, number, number], i: number) {
    if (i < 3) return n[i]
    return n[1] && n[2] / n[1]
  }
  function typeFmt(n: [number, number, number], i: number) {
    switch (i) {
      case 0: return n[0]
      case 1: return inKB(n[1] / 1024)
      case 2: return inKB(n[2] / 1024)
      default: return percentFmt(n[1] && n[2] / n[1])
    }
  }
  let inKB = useUnitFmt('kilobyte', 2)
</script>
<h1>File info</h1>
<div>{data.name}</div>
<div class="text-sm">Files: {typeFmt(allSizes, 0)}</div>
<div class="text-sm">Internal size: {typeFmt(allSizes, 2)} / {typeFmt(allSizes, 1)} ({typeFmt(allSizes, 3)})</div>
{#if data.sizes != null}
  <div class="f justify-center items-center bgvar-c-bg2 rounded-md mx-auto my-2 p-1 gap-1 text-sm sel:bgvar-c-bg1">
    {#each byType as bt, i}
      <label><input type="radio" bind:group={by} value={i} hidden><span class="inline-block p-1 rounded-md hov-effect">{bt}</span></label>
    {/each}
  </div>
  <div class="f gap-1 my-4 items-stretch justify-between">
    <div class="f flex-1 justify-center">
      <div class="f flex-col justify-end h-full w-16 [&>div]:h-[calc(100%*var(--v))]">
        {#each types as [c, s] (c)}
          {@const all = typeData(allSizes, by)}
          <div class={s} style:--v={all > 0 ? typeData(data.sizes[c], by)/all : 0}></div>
        {/each}
      </div>
    </div>
    <div class="flex-1">
      {#each types as [c, s, n] (c)}
        {@const d = data.sizes[c]}
        <div class="items-center g grid-cols-[1.5rem_1fr] grid-rows-2">
          <i class={s + " inline-block size-4 mx-1 rounded row-span-2"}></i>
          <span class="text-sm">{n}</span>
          <span class="text-xs">{typeFmt(d, by)}</span>
        </div>
      {/each}
    </div>
  </div>
{/if}