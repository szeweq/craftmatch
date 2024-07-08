<script lang="ts">
  import { dateTimeFmt } from '$lib/intl.svelte'
  let {data}: { data: import('./$types').PageData } = $props()
</script>
<h1>Errors</h1>
<p class="text-sm">From {data.name}</p>
<ul class="text-sm b-2 b-solid b-white/40 rounded-md list-none mx-0 my-2 text-truncate">
  {#each data.errors as [d, t, e]}
    <li class="f flex-col hov-effect justify-between gap-1 p-1">
      <div class="text-xs c-white/60">{dateTimeFmt(d)}: {t}</div>
      <div>{e}</div>
    </li>
  {:else}
    <li class="p-1 text-center">No errors</li>
  {/each}
</ul>