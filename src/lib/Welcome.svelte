<script lang="ts">
  import { invoke } from "@tauri-apps/api/core"
</script>
<div class="f flex-col items-center justify-center select-none">
  <h1>Welcome!</h1>
  <p>Choose the options:</p>
  <nav>
    <button onclick={() => invoke<boolean>("open_workspace")}>Open workspace</button>
  </nav>
</div>
