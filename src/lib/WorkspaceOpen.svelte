<script module>
const actions = [
  { name: 'Tags', href: '/all/tags' },
  { name: 'Inheritance', href: '/all/inheritance' },
  { name: 'Complexity', href: '/all/complexity' },
  { name: 'File types', href: '/all/filetypes' },
  { name: 'Dependencies', href: '/all/deps' },
  { name: 'Parse times', href: '/dbg/parse_times' },
]
</script>
<script lang="ts">
  import QInput from "$lib/QInput.svelte"
  import SortBtn from "$lib/SortBtn.svelte"
  import { useUnitFmt } from "$lib/intl.svelte"
  import { queryable, sortable } from "$lib/data.svelte"
  import { ws } from "$lib/workspace.svelte"
  import { invokeWS } from "$lib/ws"
  import { routes as jarActions } from "./jarRoutes"
  import type { ToggleEventHandler } from "svelte/elements"

  let queryFiles = queryable(() => ws.files, x => x[1])
  let sortFiles = sortable(queryFiles, x => x[2])
  let kbfmt = useUnitFmt('kilobyte')
  let lastSelected = $state<FileID | null>(null)
  let menupos = $state<[number, number]>([0, 0])
  let activePopover = $state<HTMLElement | null>(null)
  const showMenu = (e: HTMLElement, id: FileID) => {
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

  $effect.pre(ws.loadFiles)
</script>
<h1>Workspace opened</h1>
<div>
  Check full reports on all mods in this directory:
  <nav class="actions py-1">
    {#each actions as { name, href }}
      <a role="button" {href}>{name}</a>
    {/each}
  </nav>
</div>
<h2>Files</h2>
<section class="stick-top rounded-md bgvar-c-bg1 p-1 z-1">
  <QInput {...queryFiles} id="ws-files-q" placeholder="Search files" />
  <SortBtn label="Sort by size" bind:sort={sortFiles.sortID} />
</section>
<ul class="text-sm b-2 b-solid b-white/40 rounded-md mx-0 my-2 text-truncate">
  {#each sortFiles as [id, f, n] (id)}
    <li class="f hov-effect justify-between gap-1 px-1 items-center">
      <a class=":uno: flex-1 block hover:c-inherit! p-1" href={`/jar/${id}`}>
        <div>{f}</div>
        <div class="text-xs c-white/60">{kbfmt(n / 1024)}</div>
      </a>
      <button class="btn-icon before:i-ms-open-in-new" aria-label="Show" onclick={() => invokeWS('ws_show', {id})}></button>
      <button class="btn-icon before:i-ms-more-vert" aria-label="Options" popovertarget="file-opts" onclick={e => showMenu(e.currentTarget, id)}></button>
    </li>
  {/each}
</ul>
<div bind:this={activePopover} id="file-opts" popover="auto" class="rounded-xl p-1 left-unset" style={`top: ${menupos[0]}px; right: ${menupos[1]}px`} ontoggle={popoverToggle}>
  {#if lastSelected}
    <nav class="f flex-col">
      {#each jarActions as { n, p }}
        <a role="button" href="/jar/{lastSelected}{p}">{n}</a>
      {/each}
    </nav>
  {:else}
    <span>Nothing selected</span>
  {/if}
</div>