<script>
import "../styles.css"
import "virtual:uno.css"
import { page } from "$app/stores"
import { ws } from "$lib/workspace.svelte"
import Loading from "$lib/Loading.svelte"
import SettingsModal from "$lib/SettingsModal.svelte"
import { goto } from "$app/navigation"
import AuthModal from "$lib/AuthModal.svelte"
import { user } from "$lib/auth.svelte"
import Modal from "$lib/Modal.svelte"
let {children} = $props()
let backEnabled = $state(false)
$effect.pre(() => page.subscribe(p => backEnabled = p.url.pathname !== "/"))
let openDialog = $state(0)
const closeDialog = () => openDialog = 0
</script>
<svelte:document onerror={e => console.log(e)} />
<aside>
  <button class="btn-icon before:i-ms-arrow-back" onclick={() => history.back()} disabled={!backEnabled} title="Go back"></button>
  {#if ws.isOpen}<button class="btn-icon" onclick={() => ws.close().then(() => goto("/"))} title="Close workspace"><span class="i-ms-folder-off"></span></button>{/if}
  <div class="grow"></div>
  {#if user.name === ""}
    <button class="btn-icon before:i-ms-account-circle" onclick={() => openDialog = 2} title="Log in"></button>
  {:else}
    <button class="btn-icon" onclick={() => {}} title="Log out"><img src={user.avatar} alt={user.name} class="rounded-full" width="32" height="32"/></button>
  {/if}
  <button class="btn-icon before:i-ms-settings" onclick={() => openDialog = 1} title="Settings"></button>
</aside>
<main>
  {@render children()}
  <Loading />
</main>
<footer>...</footer>
<Modal open={openDialog === 1}><SettingsModal onclose={closeDialog}/></Modal>
<Modal open={openDialog === 2}><AuthModal onclose={closeDialog} /></Modal>

