<script lang="ts">
  import { invoke } from "@tauri-apps/api/core"
  import { ws } from "./workspace.svelte"
  let moddirs = $state<string[]>([])
  let select = $state<string | null>(null)
  $effect.pre(() => {
    if (moddirs.length === 0) invoke<string[]>("mod_dirs", {kind: null}).then(x => {
      moddirs = x
    })
    else if (select) invoke("mod_dirs", {kind: select})
  })
</script>
<div class="f flex-col items-center justify-center select-none">
  <h1>Welcome!</h1>
  <p>Choose the options:</p>
  <nav>
    <button onclick={ws.open}>Open workspace</button>
  </nav>
</div>
{#if select}
  <h2>Opening selected directory</h2>
  <p>At: {select}</p>
{:else if moddirs.length > 0}
  <h2>Found Minecraft directories</h2>
  <ul class="text-sm b-2 b-solid b-white/40 rounded-md mx-0 my-2 text-truncate">
    {#each moddirs as d (d)}
      <li><a class="block c-inherit hover:c-inherit! p-1 hov-effect" href="#" onclick={e => {e.preventDefault(); select = d}}>{d}</a></li>
    {/each}
  </ul>
{/if}