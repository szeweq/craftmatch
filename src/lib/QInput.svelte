<script lang="ts" generics="T">
  import type { Queryable } from "./queryable.svelte";
  let {q}: {q: Queryable<T>} = $props()
</script>
<label class="input-group">
  <input type="text" bind:value={q.q} />
  <span class="fw-400">{q.queried.length}/{q.source.length}</span>
</label>