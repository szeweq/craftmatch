<script lang="ts">
import { filterBy, sortBy } from "./query"
import { ws } from "./workspace.svelte"
import type { Snippet } from "svelte"
let {class: c = "", q = "", sortSize = false, item}: {class?: string, q?: string, sortSize?: boolean, item: Snippet<[UUID, string, number]>} = $props()
let queried = $derived(filterBy(ws.files, q, x => x[1]))
let sorted = $derived(sortBy(queried, sortSize, x => x[2]))
$effect.pre(ws.loadFiles)
</script>
<ul class={c}>
  {#each sorted as [id, f, n] (id)}
    {@render item(id, f, n)}
  {/each}
</ul>