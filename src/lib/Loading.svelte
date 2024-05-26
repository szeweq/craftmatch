<script>
  import { navigating } from "$app/stores"
  import { fade } from "svelte/transition"

  let load = $state(false)
  $effect.pre(() => navigating.subscribe(n => load = !!n))
</script>
{#if load}
<div class="fixed inset-0 left-[var(--s-aside)] z-99 darken" in:fade={{duration: 200}} out:fade={{duration: 200}}>
  <div class="f items-center justify-center text-2xl p-1 m-4">
    <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
      <circle cx="4" cy="12" r="3" fill="#fff">
        <animate id="b0" attributeName="cy" begin="0;b1.end+0.25s" calcMode="spline" dur="0.6s" keySplines=".33,.66,.66,1;.33,0,.66,.33" values="12;6;12" />
      </circle>
      <circle cx="12" cy="12" r="3" fill="#fff">
        <animate attributeName="cy" begin="b0.begin+0.1s" calcMode="spline" dur="0.6s" keySplines=".33,.66,.66,1;.33,0,.66,.33" values="12;6;12" />
      </circle>
      <circle cx="20" cy="12" r="3" fill="#fff">
        <animate id="b1" attributeName="cy" begin="b0.begin+0.2s" calcMode="spline" dur="0.6s" keySplines=".33,.66,.66,1;.33,0,.66,.33" values="12;6;12" />
      </circle>
    </svg>
  </div>
</div>
{/if}