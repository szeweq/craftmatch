<script lang="ts">
  import type { Snippet } from "svelte"

  let {open, children}: {open: boolean, children: Snippet} = $props()
  let dialog: HTMLDialogElement = $state()
  $effect.pre(() => {
    if (dialog) {
      if (open) dialog.showModal()
      else dialog.close()
    }
  })
</script>
<dialog bind:this={dialog} class="w-50vw">{@render children()}</dialog>