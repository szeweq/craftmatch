<script lang="ts">
  import { settings } from "./settings.svelte"
  import { currentLang } from "./intl.svelte"
  let {open = $bindable(), onclose}: {open: boolean, onclose?: () => void} = $props()
  let dialog: HTMLDialogElement = $state()
  $effect.pre(() => {
    if (dialog) {
      if (open) dialog.showModal()
      else dialog.close()
    }
  })
</script>
<dialog bind:this={dialog} class="w-1/2">
  <section>
    <h2>About</h2>
    <div>Craftmatch</div>
    <div>by Szeweq</div>
    <div>
      <label>
        <input type="checkbox" bind:checked={settings.loc}>
        <span>Use local language for formatting</span>
      </label>
      <div>Current language: {currentLang()}</div>
    </div>
    <div></div>
    <nav><button onclick={onclose}>Close</button></nav>
  </section>
</dialog>