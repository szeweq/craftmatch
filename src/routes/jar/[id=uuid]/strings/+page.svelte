<script lang="ts">
  import { filterBy, sortBy } from '$lib/query';

  let {data}: { data: import('./$types').PageData } = $props()
  let q = $state("")
  let queried = $derived(filterBy(data?.strings || [], q, ([s]) => s))
  let sortCount = $state(false)
  let sorted = $derived(sortBy(queried, sortCount, ([,j]) => j.length))
</script>
<section class="sticky top-0 rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1">
  <input type="text" bind:value={q} />
  <span class="text-xs">Item count: {queried.length}</span>
  <input id="sortCount" type="checkbox" bind:checked={sortCount} />
  <label for="sortCount">Sort by count</label>
</section>
<ul class="text-xs list-none px-1">{#each sorted as [c, x] (c)}
  <li><details>
    <summary>{JSON.stringify(c)} ({x.length})</summary>
    <div class="ml-1 pl-3 b-0 b-l-2 b-solid b-white/40">{#each x as u}
      <div>{data.classes[u]}</div>
    {/each}</div>
  </details></li>
{/each}</ul>