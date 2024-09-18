<script lang="ts">
    import EntryHeader from '$lib/EntryHeader.svelte';
import srv from '$lib/srv'

let {data}: { data: import('./$types').PageData } = $props()
let selected = $state(-1)
</script>
<EntryHeader {data} title="Player" />
<div class="f flex-col text-xs">
  {#each data.files as f, i}
    <label><input type="radio" bind:group={selected} value={i} hidden /><span>{f}</span></label>
  {:else}
    <div>It seems that this mod does not have any audio files.</div>
  {/each}
</div>
<div class="stick-bottom left-[calc(var(--s-aside)+1rem)] right-4">
  {#if selected >= 0 && selected < data.files.length}
    {@const m = data.files[selected]}
    <audio class="w-full" controls autoplay src={srv.url(`/raw/${data.id}/${m}`)} preload="auto"></audio>
  {/if}
</div>