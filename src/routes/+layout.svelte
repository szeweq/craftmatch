<script module>
import SettingsModal from "$lib/SettingsModal.svelte"
import AuthModal from "$lib/AuthModal.svelte"
import WrongModal from "$lib/WrongModal.svelte"
const dialogs = [
  SettingsModal,
  AuthModal,
  WrongModal
]
</script>
<script>
import "../styles.css"
import "virtual:uno.css"
import { page } from "$app/stores"
import { ws } from "$lib/workspace.svelte"
import Loading from "$lib/Loading.svelte"
import { goto } from "$app/navigation"
import { user } from "$lib/auth.svelte"
import Modal from "$lib/Modal.svelte"
let {children} = $props()
let backEnabled = $state(false)
$effect.pre(() => page.subscribe(p => backEnabled = p.url.pathname !== "/"))
let openDialog = $state(0)
const closeDialog = () => openDialog = 0
let ModalInner = $derived(dialogs[Math.min(openDialog, dialogs.length - 1) - 1] ?? null)
</script>
<svelte:document onerror={e => console.log(e)} />
<aside>
  <button class="btn-icon before:i-ms-arrow-back" onclick={() => history.back()} disabled={!backEnabled} aria-label="Go back" title="Go back"></button>
  {#if ws.isOpen}<button class="btn-icon before:i-ms-folder-off" onclick={() => ws.close().then(() => goto("/"))} aria-label="Close workspace" title="Close workspace"></button>{/if}
  <div class="grow"></div>
  {#if user.name === ""}
    <button class="btn-icon before:i-ms-account-circle" onclick={() => openDialog = 2} aria-label="Log in" title="Log in"></button>
  {:else}
    <button class="btn-icon" onclick={() => {}} aria-label="Log out" title="Log out"><img src={user.avatar} alt={user.name} class="rounded-full" width="32" height="32"/></button>
  {/if}
  <button class="btn-icon before:i-ms-settings" onclick={() => openDialog = 1} aria-label="Settings" title="Settings"></button>
</aside>
<main>
  {@render children()}
  <Loading />
</main>
<footer>...</footer>
<Modal open={openDialog > 0}>
  <ModalInner onclose={closeDialog} />
</Modal>

