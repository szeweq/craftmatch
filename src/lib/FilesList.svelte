<script lang="ts">
import { filterBy, sortBy } from "./query"
import { ws } from "./workspace.svelte"
import type { Snippet } from "svelte"
let {class: c = "", q = "", qlen = $bindable(0), sortSize = false, item}: {
  class?: string,
  q?: string,
  qlen?: number,
  sortSize?: boolean,
  item: Snippet<[FileID, string, number]>
} = $props()
let queried = $derived(filterBy(ws.files, q, x => x[1]))
let sorted = $derived(sortBy(queried, sortSize, x => x[2]))
$effect.pre(ws.loadFiles)
$effect(() => {qlen = queried.length})
</script>
<ul class={c}>
  {#each sorted as [id, f, n] (id)}
    {@render item(id, f, n)}
  {/each}
</ul>