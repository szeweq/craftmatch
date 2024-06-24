<script lang="ts">
import { filterBy, sortBy } from "./query"
import { ws } from "./workspace.svelte"
import type { Snippet } from "svelte"
let {class: c = "", q = "", qlen = $bindable(0), sortSize = 0, item}: {
  class?: string,
  q?: string,
  qlen?: number,
  sortSize?: number,
  item: Snippet<[FileID, string, number]>
} = $props()
let queried = $derived(filterBy(ws.files, q, x => x[1]))
let sorted = $derived(sortBy(queried, sortSize && (x => x[2]), sortSize > 1))
$effect.pre(ws.loadFiles)
$effect(() => {qlen = queried.length})
</script>
<ul class={c}>
  {#each sorted as [id, f, n] (id)}
    {@render item(id, f, n)}
  {/each}
</ul>