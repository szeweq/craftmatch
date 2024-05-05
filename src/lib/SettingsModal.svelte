<script lang="ts">
  let {open = $bindable(), onclose}: {open: boolean, onclose?: () => void} = $props()
  let dialog: HTMLDialogElement = $state()
  $effect.pre(() => {
    if (dialog) {
      if (open) dialog.showModal()
      else dialog.close()
    }
  })
</script>
<dialog bind:this={dialog}>
  <section>
    <h2>About</h2>
    <div>Craftmatch</div>
    <div>by Szeweq</div>
    <div></div>
    <nav><button onclick={onclose}>Close</button></nav>
  </section>
</dialog>