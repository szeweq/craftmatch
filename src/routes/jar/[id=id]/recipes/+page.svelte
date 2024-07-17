<script>
  let {data} = $props()
  let types = $derived(Object.keys(data.recipes))
  let ctype = $state("")
  $effect(() => {
    if (ctype == "") ctype = types[0]
  })
</script>
<h1>All recipes</h1>
<p>From: {data.name}</p>
{#if types.length > 0}
<div class="f">
  <div class="f flex-col w-40 text-xs gap-1 bgvar-c-bg2 rounded-md p-1 sel:bgvar-c-bg1">
    {#each types as s (s)}<label><input type="radio" bind:group={ctype} value={s} hidden /><span class="block p-1 rounded-md hov-effect text-ellipsis of-hidden">{s}</span></label>{/each}
  </div>
  {#if ctype && data.recipes[ctype]}
    <div class="px-1">
      <h2>{ctype}</h2>
      <ul class="text-xs">
        {#each data.recipes[ctype] as k (k)}
          <li>{k}</li>
        {/each}
      </ul>
    </div>
  {/if}
</div>
{:else}
<div>It seems that this mod does not have any recipes.</div>
{/if}