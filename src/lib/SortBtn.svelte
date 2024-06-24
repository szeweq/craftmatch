<script>
  /** @type {{label: string, sort: number}} */
  let {label = "", sort = $bindable()} = $props()
</script>
<button class={['', 'sort-desc', 'sort-asc'][sort]} onclick={() => sort = (sort + 1) % 3}>{label}</button>