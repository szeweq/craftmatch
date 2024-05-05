<script>
  import FilesList from "$lib/FilesList.svelte"
  import Welcome from "$lib/Welcome.svelte"
  import { ws } from "$lib/workspace.svelte"
  
  let fileQuery = $state("")
</script>

{#if !ws.open}
  <Welcome />
{:else}
  <h1>Workspace opened</h1>
  <div>
    Check full reports on all mods in this directory:
    <nav class="py-1">
      <a role="button" href="/all/tags">Tags</a>
      <a role="button" href="/all/inheritance">Inheritance</a>
      <a role="button" href="/all/complexity">Complexity</a>
    </nav>
  </div>
  <h2>Files</h2>
  <section class="sticky top-0 rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1">
    <input type="text" bind:value={fileQuery} placeholder="Search files...">
  </section>
  <FilesList class="text-sm b-2 b-solid b-white/40 rounded-md list-none mx-0 my-2 text-truncate" q={fileQuery}>
    {#snippet item(id, f)}
      <li><a class="block c-inherit hover:c-inherit! p-1 hover:bg-white/20" href={`/jar/${id}`} title={id}>{f}</a></li>
    {/snippet}
  </FilesList>
{/if}