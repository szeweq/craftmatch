<script lang="ts">
  import Welcome from "$lib/Welcome.svelte"
  import WorkspaceOpen from "$lib/WorkspaceOpen.svelte"
  import { ws } from "$lib/workspace.svelte"
  let Wsc = ws.isOpen ? WorkspaceOpen : Welcome
</script>
<Wsc />