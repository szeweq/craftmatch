<script lang="ts">
  import { page } from '$app/stores'
  let {data}: { data: import('./$types').PageData } = $props()
  let is_404 = $state(false)
  $effect(() => page.subscribe(p => is_404 = p.status == 404))
</script>
{#if is_404}
  <h1>Not found</h1>
  <p>It seems that this mod page does not exist.</p>
{:else}
  <h1>Error in {data.name}</h1>
  <p>Check the errors page for this mod to see what happened.</p>
  <p>It seems that the author(s) if this mod developed a mod with broken manifest or dependencies.</p>
  <a role="button" href="/jar/{data.id}/errors">Errors</a>
{/if}