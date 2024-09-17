<script lang="ts">
  let {data, title}: { data: { id: string, name: string }, title: string } = $props()
</script>
<div class="f items-center gap-2"><a href="/jar/{data.id}">{data.name}</a><hr class="grow" /></div>
<h1>{title}</h1>