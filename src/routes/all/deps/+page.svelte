<script lang="ts">
  let {data}: { data: import('./$types').PageData } = $props()
</script>
<h1>All dependencies</h1>
<ul>
  {#each Object.entries(data) as [k, {v, deps}] (k)}
    <li>{k}: {v}</li>
  {/each}
</ul>