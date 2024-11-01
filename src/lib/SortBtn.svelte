<script module>
  const sorts = ['sort-none', 'sort-desc', 'sort-asc']
</script>
<script>
  /** @type {{label: string, sort: number}} */
  let {label = "", sort = $bindable()} = $props()
</script>
<button class={sorts[sort]} onclick={() => sort = (sort + 1) % 3}>{label}</button>