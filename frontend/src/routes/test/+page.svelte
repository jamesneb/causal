<script>
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';
  
  let message = "This is a test page";
  let mounted = false;
  
  onMount(() => {
    mounted = true;
    console.log("Test page mounted in browser:", browser);
  });
</script>

<div class="test-page">
  <h1>Test Page</h1>
  <p>{message}</p>
  
  {#if mounted}
    <div class="success">✅ Component successfully mounted in browser</div>
  {:else}
    <div class="pending">⏳ Component not mounted yet</div>
  {/if}
  
  {#if browser}
    <div class="browser-check success">✅ Running in browser environment</div>
  {:else}
    <div class="browser-check warning">⚠️ Not running in browser environment</div>
  {/if}
</div>

<style>
  .test-page {
    padding: 2rem;
    max-width: 600px;
    margin: 0 auto;
  }
  
  h1 {
    margin-bottom: 1rem;
  }
  
  .success {
    color: #00ff88;
    padding: 0.5rem;
    border: 1px solid #00ff88;
    border-radius: 4px;
    margin: 1rem 0;
  }
  
  .pending, .warning {
    color: #ffcc00;
    padding: 0.5rem;
    border: 1px solid #ffcc00;
    border-radius: 4px;
    margin: 1rem 0;
  }
</style>