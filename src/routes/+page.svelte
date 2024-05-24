<script lang="ts">
  import FilesList from "$lib/FilesList.svelte"
  import Welcome from "$lib/Welcome.svelte"
  import { dateFmt, useUnitFmt } from "$lib/intl.svelte"
  import { ws } from "$lib/workspace.svelte"
  import { wsShow } from "$lib/ws"
  import type { ToggleEventHandler } from "svelte/elements"
  
  let fileQuery = $state("")
  const uuidv7time = (id: UUID) => new Date(parseInt(id.slice(0, 8) + id.slice(9, 13), 16))
  let kbfmt = useUnitFmt('kilobyte')
  let sortSize = $state(false)
  let lastSelected = $state<UUID | null>(null)
  let menupos = $state<[number, number]>([0, 0])
  let activePopover = $state<HTMLElement | null>(null)
  const showMenu = (e: HTMLElement, id: UUID) => {
    if (lastSelected == id) return
    lastSelected = id
    const rect = e.getBoundingClientRect()
    menupos = [rect.bottom, rect.right - rect.left]
    requestAnimationFrame(() => activePopover?.showPopover())
  }
  const closePopover = () => activePopover?.hidePopover()
  const popoverToggle: ToggleEventHandler<HTMLElement> = e => {
    const elemMain = document.querySelector('main')
    if (e.oldState === "open") {
      elemMain.removeEventListener('scroll', closePopover)
    } else {
      elemMain.addEventListener('scroll', closePopover, true)
    }
  }
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
  <section class="sticky top-0 rounded-md b-solid b-white/40 b-2 bgvar-c-bg1 p-1 z-1">
    <input type="text" bind:value={fileQuery} placeholder="Search files...">
    <span>Mods found: {ws.files.length}</span>
    <label>
      <input type="checkbox" bind:checked={sortSize} />
      <span>Sort by size</span>
    </label>
  </section>
  <FilesList class="text-sm b-2 b-solid b-white/40 rounded-md list-none mx-0 my-2 text-truncate" q={fileQuery} sortSize={sortSize}>
    {#snippet item(id, f, n)}
      <li class="f hover:bg-white/20 justify-between gap-1 px-1 items-center">
        <a class="flex-1 block c-inherit hover:c-inherit! p-1" href={`/jar/${id}`}>
          <div>{f}</div>
          <div class="text-xs">{kbfmt(n / 1024)} | {dateFmt(uuidv7time(id))}</div>
        </a>
        <button class="btn-icon" onclick={() => wsShow(id)}><span class="i-ms-open-in-new"></span></button>
        <button class="btn-icon" popovertarget="file-opts" onclick={e => showMenu(e.currentTarget, id)}><span class="i-ms-more-vert"></span></button>
      </li>
    {/snippet}
  </FilesList>
  <div bind:this={activePopover} id="file-opts" popover="auto" class="rounded-md p-2 left-unset" style={`top: ${menupos[0]}px; right: ${menupos[1]}px`} ontoggle={popoverToggle}>
    {#if lastSelected}
      <nav class="f flex-col">
        <a role="button" href="/jar/{lastSelected}/strings">Strings</a>
        <a role="button" href="/jar/{lastSelected}/sizes">Sizes</a>
        <a role="button" href="/jar/{lastSelected}/filetypes">File types</a>
        <a role="button" href="/jar/{lastSelected}/recipes">Recipes</a>
      </nav>
    {:else}
      <span>Nothing selected</span>
    {/if}
  </div>
{/if}