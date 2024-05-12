<script lang="ts">
import { filterBy } from "./query"
import { ws } from "./workspace.svelte"
import type { Snippet } from "svelte"
let {class: c = "", q = "", item}: {class?: string, q?: string, item: Snippet<[UUID, string, number]>} = $props()
let queried = $derived(filterBy(ws.files, q, x => x[1]))
$effect.pre(ws.loadFiles)
</script>
<ul class={c}>
  {#each queried as [id, f, n] (id)}
    {@render item(id, f, n)}
  {/each}
</ul>