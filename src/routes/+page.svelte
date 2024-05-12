<script lang="ts">
  import FilesList from "$lib/FilesList.svelte"
  import Welcome from "$lib/Welcome.svelte"
  import { dateFmt, useUnitFmt } from "$lib/intl.svelte";
  import { ws } from "$lib/workspace.svelte"
  
  let fileQuery = $state("")
  const uuidv7time = (id: UUID) => new Date(parseInt(id.slice(0, 8) + id.slice(9, 13), 16))
  let kbfmt = useUnitFmt('kilobyte')
  let sortSize = $state(false)
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
    <label>
      <input type="checkbox" bind:checked={sortSize} />
      <span>Sort by size</span>
    </label>
  </section>
  <FilesList class="text-sm b-2 b-solid b-white/40 rounded-md list-none mx-0 my-2 text-truncate" q={fileQuery} sortSize={sortSize}>
    {#snippet item(id, f, n)}
      <li><a class="block c-inherit hover:c-inherit! p-1 hover:bg-white/20" href={`/jar/${id}`}>
        <div>{f}</div>
        <div class="text-xs">{kbfmt(n / 1024)} | {dateFmt(uuidv7time(id))}</div>
      </a></li>
    {/snippet}
  </FilesList>
{/if}