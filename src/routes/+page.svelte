<script lang="ts">
  import Welcome from "$lib/Welcome.svelte"
  import WorkspaceOpen from "$lib/WorkspaceOpen.svelte"
  import { ws } from "$lib/workspace.svelte"
</script>
<svelte:component this={ws.isOpen ? WorkspaceOpen : Welcome} />