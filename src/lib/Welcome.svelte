<script lang="ts">
  import { invoke } from "@tauri-apps/api/core"
  import { ws } from "./workspace.svelte"
  let moddirs = $state<string[]>([])
  $effect.pre(() => {
    invoke<string[]>("mod_dirs", {kind: null}).then(x => {
      moddirs = x
    })
  })
</script>
<div class="f flex-col items-center justify-center select-none">
  <h1>Welcome!</h1>
  <p>Choose the options:</p>
  <nav>
    <button onclick={ws.open}>Open workspace</button>
  </nav>
</div>
{#if moddirs.length > 0}
  <h2>Found Minecraft directories</h2>
  <ul class="text-sm b-2 b-solid b-white/40 rounded-md list-none mx-0 my-2 text-truncate">
    {#each moddirs as d (d)}
      <li><a class="block c-inherit hover:c-inherit! p-1 hover:bg-white/20" href="#" onclick={e => {e.preventDefault(); invoke("mod_dirs", {kind: d})}}>{d}</a></li>
    {/each}
  </ul>
{/if}