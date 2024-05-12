<script lang="ts">
  import FilesList from "$lib/FilesList.svelte"
  import Welcome from "$lib/Welcome.svelte"
  import { ws } from "$lib/workspace.svelte"
  
  let fileQuery = $state("")
  function uuidv7time(id: UUID) {
    const d = new Date(parseInt(id.slice(0, 8) + id.slice(9, 13), 16))
    return d.toISOString()
  }
  const kbfmt = new Intl.NumberFormat('en-US', {style: 'unit', unit: 'kilobyte', unitDisplay: 'short'})
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
      <a role="button" href="/dbg/parse_times">Parse times</a>
    </nav>
  </div>
  <h2>Files</h2>
  <section class="sticky top-0 rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1">
    <input type="text" bind:value={fileQuery} placeholder="Search files...">
    <span>Mods found: {ws.files.length}</span>
  </section>
  <FilesList class="text-sm b-2 b-solid b-white/40 rounded-md list-none mx-0 my-2 text-truncate" q={fileQuery}>
    {#snippet item(id, f, n)}
      <li><a class="block c-inherit hover:c-inherit! p-1 hover:bg-white/20" href={`/jar/${id}`} title={uuidv7time(id)}>{f} ({kbfmt.formatToParts(n / 1024).map(x => x.value).join("")})</a></li>
    {/snippet}
  </FilesList>
{/if}