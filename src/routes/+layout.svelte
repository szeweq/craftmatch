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
import { ws } from "$lib/workspace.svelte"
import Loading from "$lib/Loading.svelte"
import { goto } from "$app/navigation"
import Modal from "$lib/Modal.svelte"
import AuthBtn from "$lib/AuthBtn.svelte"
import BackBtn from "$lib/BackBtn.svelte"
let {children} = $props()
let openDialog = $state(0)
const closeDialog = () => openDialog = 0
let ModalInner = $derived(dialogs[Math.min(openDialog, dialogs.length - 1) - 1] ?? null)
</script>
<svelte:document onerror={e => console.log(e)} />
<aside>
  <BackBtn />
  {#if ws.isOpen}<button class="btn-icon before:i-ms-folder-off" onclick={() => ws.close().then(() => goto("/"))} aria-label="Close workspace" title="Close workspace"></button>{/if}
  <div class="grow"></div>
  <AuthBtn onlogin={() => openDialog = 2} />
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

