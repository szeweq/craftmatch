<script lang="ts">
import { user } from "$lib/auth.svelte"
let {onlogin = () => {}, onlogout = () => {}}: {onlogin?: () => void, onlogout?: () => void} = $props()
</script>
{#if user.name == ""}
  <button class="btn-icon before:i-ms-account-circle" onclick={onlogin} aria-label="Log in" title="Log in"></button>
{:else}
  <button class="btn-icon" onclick={onlogout} aria-label="Log out" title="Log out"><img src={user.avatar} alt={user.name} class="rounded-full" width="32" height="32"/></button>
{/if}