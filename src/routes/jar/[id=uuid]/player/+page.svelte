<script lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core';

let {data}: { data: import('./$types').PageData } = $props()
let selected = $state(-1)
</script>
<div class="f flex-col text-xs">
  {#each data.files as f, i}
    <label><input type="radio" bind:group={selected} value={i} hidden /><span>{f}</span></label>
  {/each}
</div>
<div class="sticky bottom-0 left-[calc(var(--s-aside)+1rem)] right-4">
  {#if selected >= 0 && selected < data.files.length}
    {@const m = data.files[selected]}
    <audio class="w-full" controls autoplay src={convertFileSrc(data.id, 'raw') + '/' + m}></audio>
  {/if}
</div>