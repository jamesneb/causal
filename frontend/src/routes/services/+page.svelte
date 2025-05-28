<script lang="ts">
  import { onMount } from 'svelte';
  import ServiceMap from '../../components/service-map/ServiceMap.svelte';
  import { mockServiceMapData } from '../../components/service-map/mockData';
  import { browser } from '$app/environment';
  
  let containerWidth = 0;
  let containerHeight = 0;
  let container: HTMLElement;
  
  // Handle resize
  function handleResize() {
    if (container) {
      containerWidth = container.clientWidth;
      containerHeight = container.clientHeight;
    }
  }
  
  onMount(() => {
    // Set initial size
    handleResize();
    
    // Add resize listener
    window.addEventListener('resize', handleResize);
    
    return () => {
      // Clean up
      window.removeEventListener('resize', handleResize);
    };
  });
</script>

<div class="service-map-page">
  <div class="service-map-header">
    <h1>Service Map</h1>
    <div class="header-controls">
      <div class="search-box">
        <input type="text" placeholder="Search services...">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
      </div>
      <div class="filter-button">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
        </svg>
        Filters
      </div>
      <div class="layout-buttons">
        <button class="layout-btn active">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <circle cx="12" cy="12" r="4" />
            <line x1="4.93" y1="4.93" x2="9.17" y2="9.17" />
            <line x1="14.83" y1="14.83" x2="19.07" y2="19.07" />
            <line x1="14.83" y1="9.17" x2="19.07" y2="4.93" />
            <line x1="4.93" y1="19.07" x2="9.17" y2="14.83" />
          </svg>
          Force
        </button>
        <button class="layout-btn">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <circle cx="12" cy="5" r="1" />
            <circle cx="19" cy="12" r="1" />
            <circle cx="5" cy="12" r="1" />
            <circle cx="12" cy="19" r="1" />
            <circle cx="16.5" cy="7.5" r="1" />
            <circle cx="7.5" cy="7.5" r="1" />
            <circle cx="7.5" cy="16.5" r="1" />
            <circle cx="16.5" cy="16.5" r="1" />
          </svg>
          Circular
        </button>
        <button class="layout-btn">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" />
            <line x1="3" y1="9" x2="21" y2="9" />
            <line x1="3" y1="15" x2="21" y2="15" />
            <line x1="9" y1="3" x2="9" y2="21" />
            <line x1="15" y1="3" x2="15" y2="21" />
          </svg>
          Grid
        </button>
        <button class="layout-btn">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="5" y="3" width="14" height="18" rx="2" />
            <line x1="8" y1="7" x2="16" y2="7" />
            <line x1="8" y1="11" x2="16" y2="11" />
            <line x1="8" y1="15" x2="16" y2="15" />
            <line x1="8" y1="19" x2="16" y2="19" />
          </svg>
          Hierarchy
        </button>
      </div>
    </div>
  </div>
  
  <div class="service-map-container" bind:this={container}>
    {#if containerWidth > 0 && containerHeight > 0}
      <ServiceMap 
        width={containerWidth} 
        height={containerHeight}
        data={mockServiceMapData}
        settings={{
          performance: {
            useWebGL: true,
            animationFPS: 60,
            detailLevel: 'high'
          }
        }}
      />
    {/if}
  </div>
</div>

<style>
  .service-map-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 0;
    background-color: var(--bg-darkest);
  }
  
  .service-map-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 24px;
    background-color: var(--bg-darker);
    border-bottom: 1px solid var(--border-light);
  }
  
  .service-map-header h1 {
    font-size: 18px;
    font-weight: 700;
    color: var(--text-brightest);
    margin: 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    position: relative;
    padding-left: 12px;
  }
  
  .service-map-header h1::before {
    content: '';
    position: absolute;
    left: 0;
    top: 4px;
    bottom: 4px;
    width: 3px;
    background: var(--primary);
  }
  
  .header-controls {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  
  .search-box {
    position: relative;
    width: 240px;
  }
  
  .search-box input {
    width: 100%;
    padding: 8px 12px 8px 36px;
    background-color: var(--bg-dark);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-sm);
    color: var(--text-bright);
    font-size: 14px;
  }
  
  .search-box svg {
    position: absolute;
    left: 10px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-medium);
  }
  
  .filter-button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background-color: var(--bg-dark);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-sm);
    color: var(--text-bright);
    font-size: 14px;
    cursor: pointer;
    font-weight: 500;
  }
  
  .filter-button:hover {
    background-color: var(--bg-medium);
    border-color: var(--primary);
  }
  
  .layout-buttons {
    display: flex;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  
  .layout-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background-color: var(--bg-dark);
    border: none;
    border-right: 1px solid var(--border-medium);
    color: var(--text-medium);
    font-size: 14px;
    cursor: pointer;
  }
  
  .layout-btn:last-child {
    border-right: none;
  }
  
  .layout-btn:hover {
    background-color: var(--bg-medium);
    color: var(--text-bright);
  }
  
  .layout-btn.active {
    background-color: rgba(0, 255, 170, 0.1);
    color: var(--primary);
  }
  
  .service-map-container {
    flex: 1;
    width: 100%;
    min-height: 0;
    position: relative;
  }
  
  .service-map-placeholder {
    background: linear-gradient(135deg, rgba(45, 127, 249, 0.05) 0%, rgba(131, 103, 199, 0.05) 100%);
    border: 1px dashed rgba(45, 127, 249, 0.3);
    border-radius: 8px;
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
  }
  
  .placeholder-text {
    margin-top: 1rem;
    font-size: 1.2rem;
    font-weight: 600;
    color: var(--text-light);
  }
  
  .placeholder-subtext {
    margin-top: 0.5rem;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }
  
  @media (max-width: 992px) {
    .service-map-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 16px;
    }
    
    .header-controls {
      width: 100%;
      flex-wrap: wrap;
      gap: 12px;
    }
    
    .search-box {
      width: 100%;
    }
    
    .layout-buttons {
      flex-grow: 1;
      justify-content: space-between;
    }
    
    .layout-btn {
      flex: 1;
      justify-content: center;
      padding: 6px;
    }
    
    .layout-btn svg {
      margin-right: 0;
    }
    
    .layout-btn span {
      display: none;
    }
  }
</style>