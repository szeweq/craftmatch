<script lang="ts" generics="T">
  import type { HTMLInputAttributes } from "svelte/elements"
  import type { Queryable } from "./data.svelte";
  let {q = $bindable(), queried, all, placeholder = "", ...props}: Queryable<T> & Omit<HTMLInputAttributes, 'type' | 'value'> = $props()
</script>
<label class="input-group">
  <input type="text" bind:value={q} {placeholder} {...props} /><span class="fw-400">{queried.length}/{all}</span>
</label>