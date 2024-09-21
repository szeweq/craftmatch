<script lang="ts">
  let {onclose}: {onclose?: () => void} = $props()
</script>
<section>
  <div>Wrong modal dialog.</div>
  <nav><button onclick={onclose}>Close</button></nav>
</section>