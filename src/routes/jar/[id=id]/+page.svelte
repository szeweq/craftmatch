<script lang="ts">
  import { imgColors } from '$lib/gradient.svelte'
  import { invokeWS } from '$lib/ws'
  import { convertFileSrc } from '@tauri-apps/api/core'

  const versionRegexp = /^(\d+)(\.(\d+)(\.(\d+))?)?/
  let {data}: { data: import('./$types').PageData } = $props()
  let img = $state<HTMLImageElement | null>(null)
  function imgError(e: Event & { currentTarget: HTMLImageElement }) {
    e.currentTarget.setAttribute("is-broken", "")
  }
  function logoUrl(path: string) {
    return path ? convertFileSrc(data.id, 'raw') + '/' + path : ""
  }
  let gradient = imgColors(() => img)
  $effect(() => gradient.compute())
</script>
<div class="attop" style={`--top-bg: ${gradient.bg}`}></div>
<h1>File: {data.name}</h1>
<nav class="actions py-1">
  <button onclick={() => invokeWS('ws_show', {id: data.id})}>Show</button>
  <a role="button" href="/jar/{data.id}/errors">Errors</a>
  <a role="button" href="/jar/{data.id}/strings">Strings</a>
  <a role="button" href="/jar/{data.id}/sizes">Sizes</a>
  <a role="button" href="/jar/{data.id}/filetypes">File types</a>
  <a role="button" href="/jar/{data.id}/recipes">Recipes</a>
  <a role="button" href="/jar/{data.id}/player">Player</a>
</nav>
{#each data.mods as m, i (m.slug)}
  <div class="px-2">
    <h2>{m.name}</h2>
    <p>{m.description}</p>
    <div class="text-sm">Authors: {m.authors ?? "Unknown"}</div>
    <div class="text-sm">License: {m.license ?? "Unknown"}</div>
    {#if versionRegexp.test(m.version)}
      <div class="text-sm">Version: {m.version}</div>
    {:else}
      <div class="c-amber text-sm">The version provided in the mod manifest is not in the correct format: {m.version}</div>
      {#if m.version.startsWith("$")}
        <div class="c-amber text-xs">YES! The version contains a dollar sign.</div>
      {/if}
    {/if}
    <div class="text-xs">This mod is developed for {data.type} (or alike).</div>
    {#if m.logo_path}<div>
      <img bind:this={img} src={logoUrl(m.logo_path)} crossorigin="" alt="logo" class="min-w-16 max-w-48" onerror={imgError} />
      <span class="img-error c-amber text-xs">Failed to load image: {m.logo_path}</span>
    </div>{/if}
    <div>
      <h3>Dependencies</h3>
      <ul class="text-sm b-2 b-solid b-white/40 rounded-md list-none mx-0 my-2 text-truncate">
        {#each Object.entries(data.deps[m.slug].deps) as [d, [v, r]] (d)}
          <li class="hover:bg-white/20 justify-between p-1 items-center">{d}: {v} ({r})</li>
        {/each}
      </ul>
    </div>
  </div>
{/each}
{#if !data.mods}
  <div>It seems that this file is not a mod. It is more likely a library or a mod provider.</div>
{/if}