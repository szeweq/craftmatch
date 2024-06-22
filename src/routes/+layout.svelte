<script>
import "../styles.css"
import "virtual:uno.css"
import { page } from "$app/stores"
import { ws } from "$lib/workspace.svelte"
import Loading from "$lib/Loading.svelte"
import SettingsModal from "$lib/SettingsModal.svelte"
import { invoke } from "@tauri-apps/api/core"
import { goto } from "$app/navigation"
    import AuthModal from "$lib/AuthModal.svelte";
    import { user } from "$lib/auth.svelte";
let {children} = $props()
let backEnabled = $state(false)
$effect.pre(() => page.subscribe(p => backEnabled = p.url.pathname !== "/"))
let settingsOpen = $state(false)
let authOpen = $state(false)
</script>
<svelte:document onerror={e => console.log(e)} />
<aside>
  <button class="btn-icon" onclick={() => history.back()} disabled={!backEnabled} title="Go back">
    <span class="i-ms-arrow-back"></span>
  </button>
  {#if ws.open}<button class="btn-icon" onclick={() => invoke("workspace", {open: false}).then(() => goto("/"))} title="Close workspace"><span class="i-ms-folder-off"></span></button>{/if}
  <div class="grow"></div>
  {#if user.name === ""}
    <button class="btn-icon" onclick={() => authOpen = true} title="Log in"><span class="i-ms-account-circle"></span></button>
  {:else}
    <button class="btn-icon" onclick={() => {}} title="Log out"><img src={user.avatar} alt={user.name} class="rounded-full" width="32" height="32"/></button>
  {/if}
  <button class="btn-icon" onclick={() => settingsOpen = true} title="Settings"><span class="i-ms-settings"></span></button>
</aside>
<main>
  {@render children()}
  <Loading />
</main>
<SettingsModal open={settingsOpen} onclose={() => settingsOpen = false}/>
<AuthModal open={authOpen} onclose={() => authOpen = false} />