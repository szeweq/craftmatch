<script lang="ts" generics="T">
  import type { Queryable } from "./data.svelte";
  let {q = $bindable(), queried, all, placeholder = ""}: Queryable<T> & {placeholder?: string} = $props()
</script>
<label class="input-group">
  <input type="text" bind:value={q} {placeholder} /><span class="fw-400">{queried.length}/{all}</span>
</label>