<script lang="ts">
import EntryHeader from '$lib/EntryHeader.svelte'
import { dateTimeFmt } from '$lib/intl.svelte'
let {data}: { data: import('./$types').PageData } = $props()
</script>
<EntryHeader {data} title="Errors" />
<ul class="text-sm b-2 b-solid b-w/40 rounded-md mx-0 my-2 text-truncate">
  {#each data.errors as [d, t, e]}
    <li class="f flex-col hov-effect justify-between gap-1 p-1">
      <div class="text-xs c-w/60">{dateTimeFmt(d)}: {t}</div>
      <div>{e}</div>
    </li>
  {:else}
    <li class="p-1 text-center">No errors</li>
  {/each}
</ul>