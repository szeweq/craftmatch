<script lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core';

let {data}: { data: import('./$types').PageData } = $props()
let selected = $state(-1)
</script>
<h1>Player</h1>
<p>Files from: {data.name}</p>
<div class="f flex-col text-xs">
  {#if data.files.length > 0}{#each data.files as f, i}
    <label><input type="radio" bind:group={selected} value={i} hidden /><span>{f}</span></label>
  {/each}{:else}
    <div>It seems that this mod does not have any audio files.</div>
  {/if}
</div>
<div class="stick-bottom left-[calc(var(--s-aside)+1rem)] right-4">
  {#if selected >= 0 && selected < data.files.length}
    {@const m = data.files[selected]}
    <audio class="w-full" controls autoplay src={convertFileSrc(data.id, 'raw') + '/' + m} preload="auto"></audio>
  {/if}
</div>