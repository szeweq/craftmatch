<script>
  let {data} = $props()
  let types = $derived(Object.keys(data))
  let ctype = $state("")
  $effect(() => {
    if (ctype == "") ctype = types[0]
  })
</script>
<h1>All tags</h1>
<div class="f">
  <div class="f flex-col w-40 text-xs gap-1 bgvar-c-bg2 rounded-md p-1 sel:bgvar-c-bg1">
    {#each types as s (s)}<label><input type="radio" bind:group={ctype} value={s} hidden /><span class="block p-1 rounded-md hov-effect">{s}</span></label>{/each}
  </div>
  {#if ctype && data[ctype]}
    <ul class="text-xs list-none px-1">
      {#each Object.entries(data[ctype]) as [k, v] (k)}
        {@const oe = Object.entries(v)}
        <li><details>
          <summary class:c-amber={oe.length < 1}>{k} ({oe.length})</summary>
          <div class="ml-1 pl-3 b-0 b-l-2 b-solid b-white/40">
            {#each oe as [k2, v2] (k2)}
              <div class:c-green={v2 === 1} class:c-red={v2 > 1}>{k2}: {v2}</div>
            {/each}
          </div>
        </details></li>
      {/each}
    </ul>
  {/if}
</div>