<script>
import "../styles.css"
import "virtual:uno.css"
import { page } from "$app/stores"
import { ws } from "$lib/workspace.svelte"
import Loading from "$lib/Loading.svelte"
import SettingsModal from "$lib/SettingsModal.svelte"
import { invoke } from "@tauri-apps/api/core"
import { goto } from "$app/navigation"
let {children} = $props()
let backEnabled = $state(false)
//let path = $state("")
$effect.pre(() => {
  page.subscribe(p => {
    backEnabled = p.url.pathname !== "/"
    //path = p.url.pathname
    //console.log(p.url.pathname)
  })
})
let settingsOpen = $state(false)
</script>
<svelte:document onerror={e => console.log(e)} />
<aside>
  <button class="btn-icon" onclick={() => history.back()} disabled={!backEnabled} title="Go back">
    <span class="i-ms-arrow-back"></span>
  </button>
  {#if ws.open}<button class="btn-icon" onclick={() => invoke("close_workspace").then(() => goto("/"))} title="Close workspace"><span class="i-ms-folder-off"></span></button>{/if}
  <div class="grow"></div>
  <button class="btn-icon" onclick={() => settingsOpen = true} title="Settings"><span class="i-ms-settings"></span></button>
</aside>
<main>
  {@render children()}
  <Loading />
</main>
<SettingsModal open={settingsOpen} onclose={() => settingsOpen = false}/>