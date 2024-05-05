<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core'

  const versionRegexp = /^(\d+)(\.(\d+)(\.(\d+))?)?/
  let {data}: { data: import('./$types').PageData } = $props()
  function imgError(e: Event & { currentTarget: HTMLImageElement }) {
    e.currentTarget.setAttribute("is-broken", "")
  }
</script>
<h1>File: {data.name}</h1>
<nav class="py-1">
  <a role="button" href="/jar/{data.id}/strings">Strings</a>
  <a role="button" href="/jar/{data.id}/sizes">Sizes</a>
</nav>
{#each data.mods as m (m.slug)}
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
    <div class="text-sm">This mod is developed for {data.type} (or alike).</div>
    {#if m.logo_path}<div>
      <img src={convertFileSrc(data.id, 'raw') + '/' + m.logo_path} alt="logo" class="min-w-16 max-w-1/2" onerror={imgError} />
      <span class="img-error c-amber text-xs">Failed to load image: {m.logo_path}</span>
    </div>{/if}
  </div>
{/each}
{#if !data.mods}
  <div>It seems that this file is not a mod. It is more likely a library or a mod provider.</div>
{/if}