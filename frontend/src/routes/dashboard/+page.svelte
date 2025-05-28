<script lang="ts">
  import { onMount } from 'svelte';
  import ServiceMap from '../../components/service-map/ServiceMap.svelte';
  import { mockServiceMapData } from '../../components/service-map/mockData';
  import { browser } from '$app/environment';
  
  // Mock data for the dashboard
  const systemHealth = 87;
  const activeServices = 12;
  const activeMonitors = 8;
  const openAlerts = 3;
  
  // Service map container
  let serviceMapContainer: HTMLElement;
  let serviceMapWidth = 0;
  let serviceMapHeight = 0;
  
  // Handle resize for service map
  function handleServiceMapResize() {
    if (serviceMapContainer) {
      serviceMapWidth = serviceMapContainer.clientWidth;
      serviceMapHeight = serviceMapContainer.clientHeight || 400;
    }
  }
  
  // Mock data for service health
  const serviceHealth = [
    { name: 'payment-service', health: 92, latency: 42, errors: 0.05, requests: 2450 },
    { name: 'inventory-service', health: 78, latency: 68, errors: 1.2, requests: 1870 },
    { name: 'user-auth', health: 98, latency: 31, errors: 0.01, requests: 3200 },
    { name: 'order-processing', health: 85, latency: 75, errors: 0.8, requests: 1320 },
    { name: 'notification-service', health: 99, latency: 28, errors: 0, requests: 950 }
  ];
  
  // Mock data for recent system events
  const systemEvents = [
    { 
      id: 1, 
      title: 'Root Cause Detected', 
      message: 'Memory pressure identified as cause of checkout failures',
      time: '1h 15m ago',
      type: 'root-cause',
      confidence: 92
    },
    { 
      id: 2, 
      title: 'Connection Pool Optimization',
      message: 'Suggested pool size increase for inventory-service',
      time: '3h 40m ago',
      type: 'optimization',
      impact: 'Estimated 35% latency reduction'
    },
    { 
      id: 3, 
      title: 'State Capture', 
      message: 'State snapshot captured for payment-service debugging',
      time: '5h ago',
      type: 'debug'
    }
  ];
  
  // Mock data for optimization suggestions
  const optimizationSuggestions = [
    {
      id: 1,
      service: 'inventory-service',
      title: 'Database Connection Pool',
      description: 'Increase connection pool size from 10 to 25',
      impact: 'High',
      confidence: 95,
      latencyImprovement: '35%'
    },
    {
      id: 2,
      service: 'payment-service',
      title: 'Cache Implementation',
      description: 'Add Redis cache for frequently accessed payment methods',
      impact: 'Medium',
      confidence: 87,
      latencyImprovement: '28%'
    },
    {
      id: 3,
      service: 'order-processing',
      title: 'Queue Configuration',
      description: 'Adjust batch size from 50 to 200 for order processing queue',
      impact: 'Medium',
      confidence: 82,
      latencyImprovement: '15%'
    }
  ];
  
  // Mock causal relationships
  const causalRelationships = [
    {
      source: 'inventory-service',
      target: 'order-processing',
      type: 'latency',
      strength: 0.85,
      description: 'Inventory service latency directly affects order processing'
    },
    {
      source: 'user-auth',
      target: 'payment-service',
      type: 'dependency',
      strength: 0.92,
      description: 'Auth service errors cause payment service failures'
    },
    {
      source: 'database-primary',
      target: 'inventory-service',
      type: 'resource',
      strength: 0.78,
      description: 'Database resource contention affects inventory service'
    }
  ];
  
  let selectedTab = 'overview';
  
  function selectTab(tab) {
    selectedTab = tab;
    
    // Initialize service map when the services tab is selected
    if (tab === 'services') {
      setTimeout(() => {
        handleServiceMapResize();
      }, 0);
    }
  }
  
  onMount(() => {
    // Add resize listener
    window.addEventListener('resize', () => {
      if (selectedTab === 'services') {
        handleServiceMapResize();
      }
    });
    
    return () => {
      window.removeEventListener('resize', handleServiceMapResize);
    };
  });
  
  function getHealthClass(health) {
    if (health >= 95) return 'excellent';
    if (health >= 85) return 'good';
    if (health >= 70) return 'fair';
    return 'poor';
  }
</script>

<div class="dashboard">
  <div class="metrics-bar">
    <div class="metric">
      <div class="metric-value">{systemHealth}%</div>
      <div class="metric-label">System Health</div>
      <div class="metric-indicator {getHealthClass(systemHealth)}"></div>
    </div>
    
    <div class="metric">
      <div class="metric-value">{activeServices}</div>
      <div class="metric-label">Active Services</div>
    </div>
    
    <div class="metric">
      <div class="metric-value">{activeMonitors}</div>
      <div class="metric-label">Active Monitors</div>
    </div>
    
    <div class="metric">
      <div class="metric-value">{openAlerts}</div>
      <div class="metric-label">Open Alerts</div>
    </div>
  </div>
  
  <div class="dashboard-tabs">
    <button 
      class="tab-button {selectedTab === 'overview' ? 'active' : ''}" 
      on:click={() => selectTab('overview')}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="7" height="9" rx="1" />
        <rect x="14" y="3" width="7" height="5" rx="1" />
        <rect x="14" y="12" width="7" height="9" rx="1" />
        <rect x="3" y="16" width="7" height="5" rx="1" />
      </svg>
      Overview
    </button>
    
    <button 
      class="tab-button {selectedTab === 'services' ? 'active' : ''}" 
      on:click={() => selectTab('services')}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3" />
        <circle cx="19" cy="5" r="2" />
        <circle cx="5" cy="5" r="2" />
        <circle cx="5" cy="19" r="2" />
        <circle cx="19" cy="19" r="2" />
        <line x1="12" y1="9" x2="12" y2="6" />
        <line x1="15" y1="12" x2="18" y2="12" />
        <line x1="12" y1="15" x2="12" y2="18" />
        <line x1="9" y1="12" x2="6" y2="12" />
      </svg>
      Services
    </button>
    
    <button 
      class="tab-button {selectedTab === 'causality' ? 'active' : ''}" 
      on:click={() => selectTab('causality')}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="8" cy="8" r="3" />
        <circle cx="16" cy="16" r="3" />
        <path d="M10.5 10.5L13.5 13.5" stroke-dasharray="2" />
        <path d="M11 3V5" />
        <path d="M13 19V21" />
      </svg>
      Causality
    </button>
    
    <button 
      class="tab-button {selectedTab === 'optimizations' ? 'active' : ''}" 
      on:click={() => selectTab('optimizations')}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2v2m0 16v2M4 12H2m20 0h-2m-9 7l-3-3m3-11l-3 3m12 0l-3-3m0 14l3-3" />
        <circle cx="12" cy="12" r="4" />
      </svg>
      Optimizations
    </button>
  </div>
  
  <div class="dashboard-content">
    {#if selectedTab === 'overview'}
      <div class="overview-layout">
        <div class="service-list card">
          <h3 class="card-header">Service Health</h3>
          <div class="service-list-content">
            {#each serviceHealth as service}
              <div class="service-item">
                <div class="service-name">{service.name}</div>
                <div class="service-stats">
                  <div class="service-stat">
                    <span class="stat-label">Health</span>
                    <span class="stat-value health-{getHealthClass(service.health)}">{service.health}%</span>
                  </div>
                  <div class="service-stat">
                    <span class="stat-label">Latency</span>
                    <span class="stat-value">{service.latency}ms</span>
                  </div>
                  <div class="service-stat">
                    <span class="stat-label">Errors</span>
                    <span class="stat-value">{service.errors}%</span>
                  </div>
                  <div class="service-stat">
                    <span class="stat-label">Requests</span>
                    <span class="stat-value">{service.requests}/min</span>
                  </div>
                </div>
                <div class="service-health-bar">
                  <div class="health-indicator {getHealthClass(service.health)}" style="width: {service.health}%"></div>
                </div>
              </div>
            {/each}
          </div>
        </div>
        
        <div class="system-events card">
          <h3 class="card-header">Recent Activity</h3>
          <div class="events-list">
            {#each systemEvents as event}
              <div class="event-item event-{event.type}">
                <div class="event-icon">
                  {#if event.type === 'root-cause'}
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <circle cx="12" cy="12" r="10" />
                      <line x1="12" y1="8" x2="12" y2="12" />
                      <line x1="12" y1="16" x2="12" y2="16" />
                    </svg>
                  {:else if event.type === 'optimization'}
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M12 2v2m0 16v2M4 12H2m20 0h-2m-9 7l-3-3m3-11l-3 3m12 0l-3-3m0 14l3-3" />
                      <circle cx="12" cy="12" r="4" />
                    </svg>
                  {:else}
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <circle cx="12" cy="12" r="10" />
                      <path d="M12 16V12l-3-2" />
                    </svg>
                  {/if}
                </div>
                <div class="event-content">
                  <div class="event-header">
                    <span class="event-title">{event.title}</span>
                    <span class="event-time">{event.time}</span>
                  </div>
                  <div class="event-message">{event.message}</div>
                  {#if event.confidence}
                    <div class="event-confidence">
                      <span class="confidence-label">Confidence:</span>
                      <span class="confidence-value">{event.confidence}%</span>
                    </div>
                  {/if}
                  {#if event.impact}
                    <div class="event-impact">
                      <span class="impact-label">Impact:</span>
                      <span class="impact-value">{event.impact}</span>
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {:else if selectedTab === 'services'}
      <div class="services-content" bind:this={serviceMapContainer}>
        {#if serviceMapWidth > 0 && serviceMapHeight > 0}
          <ServiceMap 
            width={serviceMapWidth} 
            height={serviceMapHeight}
            data={mockServiceMapData}
            settings={{
              performance: {
                useWebGL: true,
                animationFPS: 60,
                detailLevel: 'high'
              }
            }}
          />
        {:else}
          <div class="service-map-placeholder">
            <svg xmlns="http://www.w3.org/2000/svg" width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
              <circle cx="12" cy="12" r="3" />
              <circle cx="19" cy="5" r="2" />
              <circle cx="5" cy="5" r="2" />
              <circle cx="5" cy="19" r="2" />
              <circle cx="19" cy="19" r="2" />
              <line x1="12" y1="9" x2="12" y2="7" />
              <line x1="15" y1="12" x2="17" y2="12" />
              <line x1="12" y1="15" x2="12" y2="17" />
              <line x1="9" y1="12" x2="7" y2="12" />
              <path d="M7 7L10 10" />
              <path d="M17 7L14 10" />
              <path d="M7 17L10 14" />
              <path d="M17 17L14 14" />
            </svg>
            <div class="placeholder-text">Service Map</div>
            <div class="placeholder-subtext">A visual representation of your service dependencies</div>
            <a href="/services" class="view-service-map-btn">View Full Service Map</a>
          </div>
        {/if}
      </div>
    {:else if selectedTab === 'causality'}
      <div class="causality-content">
        <div class="causal-relationships card">
          <h3 class="card-header">Detected Causal Relationships</h3>
          <div class="relationship-list">
            {#each causalRelationships as relationship}
              <div class="relationship-item">
                <div class="relationship-diagram">
                  <div class="source-node">{relationship.source}</div>
                  <div class="relationship-arrow relationship-{relationship.type}">
                    <div class="arrow-line"></div>
                    <div class="arrow-head"></div>
                    <div class="relationship-strength">{relationship.strength * 100}%</div>
                  </div>
                  <div class="target-node">{relationship.target}</div>
                </div>
                <div class="relationship-description">{relationship.description}</div>
              </div>
            {/each}
          </div>
        </div>
        
        <div class="causal-explorer-placeholder">
          <svg xmlns="http://www.w3.org/2000/svg" width="72" height="72" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
            <circle cx="8" cy="8" r="3" />
            <circle cx="16" cy="16" r="3" />
            <path d="M10.5 10.5L13.5 13.5" stroke-dasharray="2" />
            <path d="M18 8H21" />
            <path d="M3 16H5" />
            <path d="M11 3V5" />
            <path d="M13 19V21" />
          </svg>
          <div class="placeholder-text">Interactive Causal Explorer</div>
          <div class="placeholder-subtext">Discover and analyze cause-effect relationships</div>
        </div>
      </div>
    {:else if selectedTab === 'optimizations'}
      <div class="optimizations-content">
        <div class="optimization-suggestions card">
          <h3 class="card-header">System Optimization Recommendations</h3>
          <div class="suggestions-list">
            {#each optimizationSuggestions as suggestion}
              <div class="suggestion-item">
                <div class="suggestion-header">
                  <div class="suggestion-service">{suggestion.service}</div>
                  <div class="suggestion-impact impact-{suggestion.impact.toLowerCase()}">{suggestion.impact} Impact</div>
                </div>
                <div class="suggestion-title">{suggestion.title}</div>
                <div class="suggestion-description">{suggestion.description}</div>
                <div class="suggestion-metrics">
                  <div class="suggestion-metric">
                    <span class="metric-label">Confidence</span>
                    <div class="confidence-bar">
                      <div class="confidence-value" style="width: {suggestion.confidence}%"></div>
                    </div>
                    <span class="metric-value">{suggestion.confidence}%</span>
                  </div>
                  <div class="suggestion-metric">
                    <span class="metric-label">Est. Improvement</span>
                    <span class="metric-value improvement-value">{suggestion.latencyImprovement}</span>
                  </div>
                </div>
                <div class="suggestion-actions">
                  <button class="apply-button">Apply Change</button>
                  <button class="simulate-button">Simulate</button>
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .metrics-bar {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1rem;
  }
  
  .metric {
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
    border-radius: 8px;
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    position: relative;
    border: 1px solid var(--border-light);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    overflow: hidden;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }
  
  .metric:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.15);
  }
  
  .metric::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: linear-gradient(90deg, var(--juniper-500), var(--juniper-300));
    opacity: 0.7;
  }
  
  .metric-value {
    font-size: 2rem;
    font-weight: 700;
    margin-bottom: 0.5rem;
    color: var(--text-light);
  }
  
  .metric-label {
    font-size: 0.85rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .metric-indicator {
    position: absolute;
    top: 1rem;
    right: 1rem;
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }
  
  .metric-indicator.excellent {
    background-color: var(--success);
    box-shadow: 0 0 8px var(--success);
  }
  
  .metric-indicator.good {
    background-color: var(--juniper-400);
    box-shadow: 0 0 8px var(--insight-teal);
  }
  
  .metric-indicator.fair {
    background-color: var(--birch);
    box-shadow: 0 0 8px var(--birch);
  }
  
  .metric-indicator.poor {
    background-color: var(--cinnabar);
    box-shadow: 0 0 8px var(--cinnabar);
  }
  
  .dashboard-tabs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  
  .tab-button {
    background: none;
    border: none;
    padding: 0.75rem 1.25rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--text-secondary);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
    font-weight: 500;
  }
  
  .tab-button:hover {
    background-color: rgba(45, 127, 249, 0.1);
    color: var(--text-light);
  }
  
  .tab-button.active {
    background-color: rgba(45, 127, 249, 0.15);
    color: var(--causal-blue);
  }
  
  .card {
    background: linear-gradient(135deg, var(--bg-secondary) 0%, rgba(19, 47, 76, 0.8) 100%);
    border-radius: 8px;
    border: 1px solid var(--border-light);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    overflow: hidden;
  }
  
  .card-header {
    padding: 1rem 1.5rem;
    font-size: 1.1rem;
    font-weight: 600;
    border-bottom: 1px solid var(--border-light);
    background: linear-gradient(to right, rgba(45, 127, 249, 0.1), transparent);
    margin: 0;
  }
  
  .overview-layout {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
  }
  
  .service-list-content {
    padding: 0.5rem;
    max-height: 400px;
    overflow-y: auto;
  }
  
  .service-item {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border-light);
  }
  
  .service-item:last-child {
    border-bottom: none;
  }
  
  .service-name {
    font-weight: 600;
    margin-bottom: 0.5rem;
  }
  
  .service-stats {
    display: flex;
    gap: 1.5rem;
    margin-bottom: 0.75rem;
  }
  
  .stat-label {
    display: block;
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin-bottom: 0.25rem;
  }
  
  .stat-value {
    font-weight: 600;
    font-size: 0.9rem;
  }
  
  .health-excellent {
    color: var(--success);
  }
  
  .health-good {
    color: var(--insight-teal);
  }
  
  .health-fair {
    color: var(--debug-amber);
  }
  
  .health-poor {
    color: var(--error-coral);
  }
  
  .service-health-bar {
    height: 4px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
  }
  
  .health-indicator {
    height: 100%;
    border-radius: 2px;
  }
  
  .health-indicator.excellent {
    background-color: var(--success);
  }
  
  .health-indicator.good {
    background-color: var(--juniper-400);
  }
  
  .health-indicator.fair {
    background-color: var(--birch);
  }
  
  .health-indicator.poor {
    background-color: var(--cinnabar);
  }
  
  .events-list {
    padding: 0.5rem;
    max-height: 400px;
    overflow-y: auto;
  }
  
  .event-item {
    display: flex;
    padding: 1rem;
    border-bottom: 1px solid var(--border-light);
    position: relative;
  }
  
  .event-item:last-child {
    border-bottom: none;
  }
  
  .event-item::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
  }
  
  .event-root-cause::before {
    background-color: var(--cinnabar);
  }
  
  .event-optimization::before {
    background-color: var(--juniper-400);
  }
  
  .event-debug::before {
    background-color: var(--synthesis-purple);
  }
  
  .event-icon {
    margin-right: 1rem;
    display: flex;
    align-items: flex-start;
  }
  
  .event-root-cause .event-icon {
    color: var(--error-coral);
  }
  
  .event-optimization .event-icon {
    color: var(--insight-teal);
  }
  
  .event-debug .event-icon {
    color: var(--synthesis-purple);
  }
  
  .event-content {
    flex: 1;
  }
  
  .event-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }
  
  .event-title {
    font-weight: 600;
  }
  
  .event-time {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  
  .event-message {
    margin-bottom: 0.5rem;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }
  
  .event-confidence, .event-impact {
    font-size: 0.85rem;
    margin-top: 0.5rem;
  }
  
  .confidence-label, .impact-label {
    color: var(--text-secondary);
    margin-right: 0.5rem;
  }
  
  .confidence-value {
    color: var(--causal-blue);
    font-weight: 600;
  }
  
  .impact-value {
    color: var(--insight-teal);
    font-weight: 600;
  }
  
  .services-content, .causality-content, .optimizations-content {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    height: 500px;
    width: 100%;
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
  
  .view-service-map-btn {
    margin-top: 1.5rem;
    padding: 0.6rem 1.2rem;
    background: linear-gradient(135deg, var(--juniper-500), var(--copper-500));
    color: white;
    text-decoration: none;
    border-radius: 6px;
    font-weight: 500;
    font-size: 0.9rem;
    transition: all 0.2s ease;
  }
  
  .view-service-map-btn:hover {
    box-shadow: 0 0 15px rgba(45, 127, 249, 0.5);
    transform: translateY(-1px);
  }
  
  .causal-explorer-placeholder {
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
  
  .view-service-map-btn {
    margin-top: 1.5rem;
    padding: 0.6rem 1.2rem;
    background: linear-gradient(135deg, var(--juniper-500), var(--copper-500));
    color: white;
    text-decoration: none;
    border-radius: 6px;
    font-weight: 500;
    font-size: 0.9rem;
    transition: all 0.2s ease;
  }
  
  .view-service-map-btn:hover {
    box-shadow: 0 0 15px rgba(45, 127, 249, 0.5);
    transform: translateY(-1px);
  }
  
  .relationship-list {
    padding: 1rem;
  }
  
  .relationship-item {
    padding: 1rem;
    border-bottom: 1px solid var(--border-light);
  }
  
  .relationship-item:last-child {
    border-bottom: none;
  }
  
  .relationship-diagram {
    display: flex;
    align-items: center;
    margin-bottom: 1rem;
  }
  
  .source-node, .target-node {
    background-color: rgba(45, 127, 249, 0.1);
    border: 1px solid rgba(45, 127, 249, 0.3);
    padding: 0.5rem 1rem;
    border-radius: 4px;
    font-weight: 600;
    font-size: 0.9rem;
  }
  
  .relationship-arrow {
    flex: 1;
    position: relative;
    height: 2px;
    margin: 0 1rem;
  }
  
  .arrow-line {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
  }
  
  .relationship-latency .arrow-line {
    background-color: var(--cinnabar);
  }
  
  .relationship-dependency .arrow-line {
    background-color: var(--causal-blue);
  }
  
  .relationship-resource .arrow-line {
    background-color: var(--birch);
  }
  
  .arrow-head {
    position: absolute;
    right: -6px;
    top: -4px;
    width: 0;
    height: 0;
    border-style: solid;
    border-width: 5px 0 5px 8px;
  }
  
  .relationship-latency .arrow-head {
    border-color: transparent transparent transparent var(--error-coral);
  }
  
  .relationship-dependency .arrow-head {
    border-color: transparent transparent transparent var(--causal-blue);
  }
  
  .relationship-resource .arrow-head {
    border-color: transparent transparent transparent var(--debug-amber);
  }
  
  .relationship-strength {
    position: absolute;
    top: -20px;
    left: 50%;
    transform: translateX(-50%);
    font-size: 0.8rem;
    background-color: var(--bg-tertiary);
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
  }
  
  .relationship-description {
    color: var(--text-secondary);
    font-size: 0.9rem;
  }
  
  .suggestions-list {
    padding: 1rem;
  }
  
  .suggestion-item {
    background-color: rgba(45, 127, 249, 0.05);
    border-radius: 8px;
    padding: 1.25rem;
    margin-bottom: 1rem;
    border: 1px solid rgba(45, 127, 249, 0.1);
  }
  
  .suggestion-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }
  
  .suggestion-service {
    font-size: 0.85rem;
    background-color: rgba(45, 127, 249, 0.1);
    color: var(--causal-blue);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-weight: 500;
  }
  
  .suggestion-impact {
    font-size: 0.85rem;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-weight: 500;
  }
  
  .impact-high {
    background-color: rgba(6, 214, 160, 0.1);
    color: var(--success);
  }
  
  .impact-medium {
    background-color: rgba(255, 200, 87, 0.1);
    color: var(--debug-amber);
  }
  
  .impact-low {
    background-color: rgba(255, 90, 95, 0.1);
    color: var(--text-secondary);
  }
  
  .suggestion-title {
    font-weight: 600;
    font-size: 1.1rem;
    margin-bottom: 0.5rem;
  }
  
  .suggestion-description {
    color: var(--text-secondary);
    margin-bottom: 1rem;
    font-size: 0.9rem;
  }
  
  .suggestion-metrics {
    display: flex;
    gap: 2rem;
    margin-bottom: 1.25rem;
  }
  
  .suggestion-metric {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .confidence-bar {
    width: 100px;
    height: 6px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    overflow: hidden;
  }
  
  .confidence-value {
    height: 100%;
    background: linear-gradient(90deg, var(--juniper-500), var(--juniper-300));
    border-radius: 3px;
  }
  
  .improvement-value {
    color: var(--success);
  }
  
  .suggestion-actions {
    display: flex;
    gap: 1rem;
  }
  
  .apply-button, .simulate-button {
    padding: 0.6rem 1rem;
    border-radius: 6px;
    font-weight: 500;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .apply-button {
    background: linear-gradient(135deg, var(--juniper-500), var(--copper-500));
    color: white;
    border: none;
    flex: 1;
  }
  
  .apply-button:hover {
    box-shadow: 0 0 15px rgba(45, 127, 249, 0.5);
    transform: translateY(-1px);
  }
  
  .simulate-button {
    background: none;
    border: 1px solid var(--border-light);
    color: var(--text-secondary);
  }
  
  .simulate-button:hover {
    background-color: rgba(255, 255, 255, 0.05);
    color: var(--text-light);
  }
</style>