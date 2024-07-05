<script lang="ts">
  import { settings } from "./settings.svelte"
  import { currentLang } from "./intl.svelte"
  let {onclose}: {onclose?: () => void} = $props()
</script>
<section>
  <h2>About</h2>
  <div>Craftmatch</div>
  <div>by Szeweq</div>
  <div>
    <label>
      <input id="loc" type="checkbox" bind:checked={settings.loc}>
      <span>Use local language for formatting</span>
    </label>
    <div>Current language: {currentLang()}</div>
  </div>
  <div></div>
  <nav><button onclick={onclose}>Close</button></nav>
</section>