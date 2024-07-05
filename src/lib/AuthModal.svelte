<script lang="ts">
  import { doAuth } from "./auth.svelte";
  let {onclose}: {onclose?: () => void} = $props()
  let step = $state(0)
  let code = $state("")
  let codeUrl = $state("")
  const login = () => {
    step = 1
    doAuth((c, u) => {
      code = c
      codeUrl = u
      step = 2
    }).then(e =>step = e ? 3 : 4)
  }
  $effect(() => {
    if (step === 3 && onclose) setTimeout(onclose, 1000)
  })
</script>
<section>
  <h2>Sign in</h2>
  {#if step == 0}
    <nav class="flex gap-1">
      <button onclick={login}>Log in with GitHub</button>
      <button onclick={onclose}>Close</button>
    </nav>
  {:else if step == 1}
    <div>Waiting for GitHub code...</div>
  {:else if step == 2}
    <div>Code received: {code}</div>
    <div>Paste the code in the browser window!</div>
    <div>If the browser window didn't show up, enter the following URL in the address bar: {codeUrl}</div>
  {:else if step == 3}
    <div>Logged in!</div>
  {:else if step == 4}
    <div>Failed to log in!</div>
    <nav><button onclick={onclose}>Close</button></nav>
  {/if}
</section>