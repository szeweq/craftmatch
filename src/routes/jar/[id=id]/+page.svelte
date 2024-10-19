<script lang="ts">
  import { routes } from '$lib/jarRoutes'
  import { imgColors } from '$lib/gradient.svelte'
  import { invokeWS } from '$lib/ws'
  import srv from '$lib/srv'

  const versionRegexp = /^(\d+)(\.(\d+)(\.(\d+))?)?/
  let {data}: { data: import('./$types').PageData } = $props()
  let img = $state<HTMLImageElement | null>(null)
  const logoUrl = (path: string) => path ? srv.url(`/raw/${data.id}/${path}`) : ""
  function imgError(e: Event & { currentTarget: HTMLImageElement }) {
    e.currentTarget.setAttribute("is-broken", "")
  }
  let gradient = imgColors(() => img)
  $effect(gradient.compute)
</script>
<div class="attop" style={`--top-bg: ${gradient.bg}`}></div>
<h1>File: {data.name}</h1>
<nav class="actions py-1">
  <button onclick={() => invokeWS('ws_show', {id: data.id})}>Show</button>
  {#each routes as { n, p }}
    <a role="button" href="/jar/{data.id}{p}">{n}</a>
  {/each}
</nav>
{#each data.mods as m (m.slug)}
  <div class="px-2">
    <h2>{m.name}</h2>
    <div class="text-sm has-before before:me-2 before:inline-block before:i-ms-person-edit" title="Authors">{m.authors ?? "Unknown"}</div>
    <div class="text-sm has-before before:me-2 before:inline-block before:i-ms-license" title="License">{m.license ?? "Unknown"}</div>
    <p>{m.description}</p>
    {#if versionRegexp.test(m.version)}
      <div class="text-xs">Version: {m.version}</div>
    {:else}
      <div class="c-amber text-xs">The version provided in the mod manifest is not in the correct format: {m.version}</div>
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
      <ul class="text-sm b-2 b-solid b-w/40 rounded-md mx-0 my-2 text-truncate">
        {#each Object.entries(data.deps[data.depNames.indexOf(m.slug)]?.[1] ?? {}) ?? [] as [d, [v, r]] (d)}
          <li class="hov-effect justify-between p-1 items-center">{data.depNames[d]}: {v} ({r})</li>
        {:else}
          <li class="p-1 text-center">No dependencies (or broken manifest)</li>
        {/each}
      </ul>
    </div>
  </div>
{/each}
{#if !data.mods}
  <div>It seems that this file is not a mod. It is more likely a library or a mod provider.</div>
{/if}