<script lang="ts">
import { page } from "$app/stores"
</script>
<button class="btn-icon before:i-ms-arrow-back" onclick={() => history.back()} disabled={$page.url.pathname === "/"} aria-label="Go back" title="Go back"></button>