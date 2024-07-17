<script lang="ts">
    import { queryable } from '$lib/data.svelte';
    import QInput from '$lib/QInput.svelte';

  let {data}: { data: import('./$types').PageData } = $props()
  function findUsage(n: number) {
    return data.info.map((x, i) => ({i, d: x && x[1][n]})).filter(x => x.d)
  }
  let namesWithIndex = $derived<[string, number][]>(data.names.map((x, i) => [x, i]))
  const q = queryable(() => namesWithIndex, x => x[0])
</script>
<h1>All dependencies</h1>
<section class="stick-top rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1 z-1">
  <QInput {...q} placeholder="Search names" />
</section>
<ul class="text-sm b-2 b-solid b-white/40 rounded-md mx-0 my-2">
  {#each q as [k, i] (k)}
    {@const info = data.info[i]}
    {@const usage = findUsage(i)}
    <li class="hov-effect p-1 f flex-col justify-between gap-1">
      <div>{k}: {info ? info[0] ?? "Unknown" : "Not available"}</div>
      {#if info}
        {@const n = Object.entries(info[1])}
        <div class="text-xs">Deps ({n.length}): {n.map(([i, d]) => `${data.names[+i]} (${d[1]})`).join(", ")}</div>
      {/if}
      {#if usage.length}
        <div class="text-xs">Usages ({usage.length}): {usage.map(({i, d}) => `${data.names[i]} (${d[1]})`).join(", ")}</div>
      {/if}
    </li>
  {/each}
</ul>