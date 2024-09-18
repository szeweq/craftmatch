<script lang="ts">
  import Welcome from "$lib/Welcome.svelte"
  import WorkspaceOpen from "$lib/WorkspaceOpen.svelte"
  import { ws } from "$lib/workspace.svelte"
</script>
{#if ws.isOpen} <WorkspaceOpen /> {:else} <Welcome /> {/if}