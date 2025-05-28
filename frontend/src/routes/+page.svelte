<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  
  // Redirect to services page after load
  onMount(() => {
    goto('/services');
  });
</script>

<div class="loading">
  <div class="spinner"></div>
  <div class="loading-text">Loading Causal Systems Studio...</div>
</div>

<style>
  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    width: 100%;
  }
  
  .spinner {
    width: 50px;
    height: 50px;
    border: 3px solid rgba(0, 255, 170, 0.1);
    border-radius: 50%;
    border-top-color: var(--primary);
    animation: spin 0.8s linear infinite;
    margin-bottom: 1rem;
    position: relative;
  }
  
  .spinner::after {
    content: '';
    position: absolute;
    top: -3px;
    left: -3px;
    right: -3px;
    bottom: -3px;
    border-radius: 50%;
    border: 1px solid rgba(0, 255, 170, 0.3);
    opacity: 0;
    animation: pulse 2s ease-out infinite;
  }
  
  .loading-text {
    font-size: 1rem;
    color: var(--text-bright);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 1px;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  @keyframes pulse {
    0% { transform: scale(1); opacity: 0.7; }
    70% { transform: scale(1.5); opacity: 0; }
    100% { transform: scale(1.5); opacity: 0; }
  }
</style>