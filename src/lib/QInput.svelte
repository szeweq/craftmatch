<script lang="ts" generics="T">
  import type { Queryable } from "./data.svelte";
  let {q = $bindable(), queried, source, placeholder = ""}: Queryable<T> & {placeholder?: string} = $props()
</script>
<label class="input-group">
  <input type="text" bind:value={q} {placeholder} /><span class="fw-400">{queried.length}/{source.length}</span>
</label>