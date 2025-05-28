<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import type { ServiceMapData, ServiceMapSettings, ServiceMapState, Service, Connection } from './types-exports';
  import { mockServiceMapData, generateMockServiceMapData } from './mockData';
  
  // Import browser flag from SvelteKit - must be at the top level
  import { browser } from '$app/environment';
  
  // Props
  export let width: number = 1200;
  export let height: number = 800;
  export let data: ServiceMapData = mockServiceMapData;
  export let settings: Partial<ServiceMapSettings> = {};
  
  // Default settings
  const defaultSettings: ServiceMapSettings = {
    layout: 'force',
    showLabels: true,
    showMetrics: true,
    animateEdges: true,
    colorBy: 'status',
    sizeBy: 'fixed',
    focusMode: false,
    physics: {
      enabled: true,
      gravity: 0.05,
      linkStrength: 0.7,
      friction: 0.9,
      repulsion: 300
    },
    performance: {
      useWebGL: true,
      nodeLimit: 1000,
      edgeLimit: 2000,
      animationFPS: 60,
      detailLevel: 'high'
    },
    theme: {
      background: 'var(--bg-darkest)',
      text: 'var(--text-bright)',
      healthyColor: 'var(--success)',
      warningColor: 'var(--warning)',
      errorColor: 'var(--error)',
      unknownColor: 'var(--text-dim)',
      selectionGlow: 'var(--primary)',
      highlightGlow: 'var(--accent)'
    }
  };
  
  // Merge with user settings
  $: mergedSettings = { ...defaultSettings, ...settings };
  
  // Internal state
  let canvas: HTMLCanvasElement;
  let svgLabels: SVGSVGElement;
  let containerDiv: HTMLDivElement;
  let renderer: WebGLRenderer;
  let simulation: ForceSimulation;
  let animationFrameId: number;
  let isDragging = false;
  let isInitialized = false;
  let canInteract = false;
  let lastUpdateTime = 0;
  let canvasScale = 1;
  
  // Performance stats
  let stats = {
    fps: 0,
    renderTime: 0,
    physicsTime: 0,
    nodeCount: 0,
    edgeCount: 0
  };
  
  // Interaction state
  let interactionState: ServiceMapState = {
    hoveredNode: null,
    hoveredEdge: null,
    selectedNode: null,
    selectedEdge: null,
    transform: {
      x: 0,
      y: 0,
      k: 1
    }
  };
  
  // Set up resizing observer
  let resizeObserver: ResizeObserver;
  
  // Highlight sets
  let highlightedNodeIds = new Set<string>();
  let highlightedEdgeIds = new Set<string>();
  
  
  // Function to safely initialize WebGL after component is mounted in browser
  async function browserInitialize() {
    // This function should only run in the browser
    if (!browser) {
      console.warn('Browser environment not available, skipping WebGL initialization');
      return;
    }

    try {
      // Check if WebGL is supported
      const testCanvas = document.createElement('canvas');
      const testContext = testCanvas.getContext('webgl');
      
      if (!testContext) {
        // WebGL not supported - use fallback
        console.warn('WebGL not supported by this browser. Using fallback rendering.');
        isInitialized = true;
        canInteract = false;
        return;
      }
      
      // Initialize renderer - dynamically import to avoid SSR issues
      try {
        if (canvas) {
          // Import the renderer dynamically to avoid SSR issues
          const { WebGLRenderer: DynamicRenderer } = await import('./webgl/renderer');
          renderer = new DynamicRenderer(canvas);
        } else {
          console.error('Canvas element not available');
          isInitialized = true;
          canInteract = false;
          return;
        }
      } catch (error) {
        console.error('Failed to initialize WebGL renderer:', error);
        isInitialized = true;
        canInteract = false;
        return;
      }
      
      // Initialize simulation
      try {
        const { ForceSimulation: DynamicSimulation } = await import('./physics/simulation');
        simulation = new DynamicSimulation(width, height, () => {
          try {
            // Update node positions after physics simulation step
            const updatedServices = simulation.exportPositions(data.services);
            data = { ...data, services: updatedServices };
            updateRenderState();
          } catch (error) {
            console.error('Error updating positions:', error);
          }
        });
      } catch (error) {
        console.error('Failed to initialize simulation:', error);
        isInitialized = true;
        canInteract = false;
        return;
      }
      
      // Set up resize observer
      try {
        if (typeof ResizeObserver !== 'undefined' && containerDiv) {
          resizeObserver = new ResizeObserver(entries => {
            for (const entry of entries) {
              const { width: newWidth, height: newHeight } = entry.contentRect;
              handleResize(newWidth, newHeight);
            }
          });
          
          resizeObserver.observe(containerDiv);
        }
      } catch (error) {
        console.error('Error setting up ResizeObserver:', error);
      }
      
      // Apply initial layout
      try {
        applyLayout(mergedSettings.layout);
      } catch (error) {
        console.error('Error applying layout:', error);
      }
      
      // Start animation loop
      startAnimationLoop();
      
      isInitialized = true;
      canInteract = true;
    } catch (error) {
      console.error('Failed to initialize ServiceMap:', error);
      isInitialized = true;
      canInteract = false;
    }
  }
  
  // Initialize on mount - only if in browser environment
  onMount(() => {
    // Only initialize in browser environment
    if (browser) {
      // Initialize in the next event loop to ensure DOM is ready
      setTimeout(() => {
        browserInitialize();
      }, 0);
    }
    
    return () => {
      try {
        if (animationFrameId) {
          stopAnimationLoop();
        }
        if (resizeObserver) {
          resizeObserver.disconnect();
        }
        if (renderer) {
          renderer.dispose();
        }
      } catch (error) {
        console.error('Error cleaning up ServiceMap:', error);
      }
    };
  });
  
  onDestroy(() => {
    if (isInitialized) {
      stopAnimationLoop();
      if (resizeObserver) resizeObserver.disconnect();
      if (renderer) renderer.dispose();
    }
  });
  
  // Watch for data changes
  $: if (isInitialized && data) {
    simulation.initialize(data.services, data.connections);
    updateRenderState();
  }
  
  // Watch for settings changes
  $: if (isInitialized && mergedSettings) {
    simulation.updateSettings(mergedSettings.physics);
    if (mergedSettings.layout !== defaultSettings.layout) {
      applyLayout(mergedSettings.layout);
    }
  }
  
  // Apply layout
  function applyLayout(layout: ServiceMapSettings['layout']) {
    if (simulation) {
      simulation.applyLayout(layout);
    }
  }
  
  // Handle container resize
  function handleResize(newWidth: number, newHeight: number) {
    width = newWidth;
    height = newHeight;
    
    if (renderer) {
      renderer.resize(width, height);
    }
    
    if (simulation) {
      simulation.resize(width, height);
    }
  }
  
  // Start animation loop
  function startAnimationLoop() {
    lastUpdateTime = performance.now();
    animationFrameId = requestAnimationFrame(animationFrame);
  }
  
  // Stop animation loop
  function stopAnimationLoop() {
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
    }
  }
  
  // Animation frame
  function animationFrame(timestamp: number) {
    try {
      // Calculate delta time
      const deltaTime = (timestamp - lastUpdateTime) / 1000;
      lastUpdateTime = timestamp;
      
      // Limit FPS if needed
      const targetFPS = mergedSettings.performance.animationFPS;
      const frameTime = 1000 / targetFPS;
      
      // Update packets for animation
      if (mergedSettings.animateEdges) {
        try {
          updatePackets(deltaTime);
        } catch (error) {
          console.error('Error updating packets:', error);
        }
      }
      
      // Render frame
      if (renderer) {
        try {
          stats = renderer.render();
        } catch (error) {
          console.error('Error rendering frame:', error);
        }
      }
      
      // Request next frame
      animationFrameId = requestAnimationFrame(animationFrame);
    } catch (error) {
      console.error('Error in animation frame:', error);
      // Attempt to continue animation even if there was an error
      animationFrameId = requestAnimationFrame(animationFrame);
    }
  }
  
  // Update packet positions for animation
  function updatePackets(deltaTime: number) {
    // Update packet progress
    data.connections.forEach(connection => {
      if (!connection.dataPackets) return;
      
      connection.dataPackets.forEach(packet => {
        packet.progress += packet.speed * deltaTime;
        if (packet.progress > 1) {
          packet.progress = 0;
        }
      });
    });
    
    // Update render state
    updateRenderState();
  }
  
  // Update WebGL render state
  function updateRenderState() {
    if (renderer) {
      try {
        renderer.updateRenderState(
          data.services,
          data.connections,
          highlightedNodeIds,
          highlightedEdgeIds
        );
      } catch (error) {
        console.error('Error updating render state:', error);
      }
    }
  }
  
  // SVG label positioning
  $: svgTransform = `translate(${interactionState.transform.x}px, ${interactionState.transform.y}px) scale(${interactionState.transform.k})`;
  
  // Handle node hover
  function handleNodeHover(service: Service) {
    interactionState.hoveredNode = service;
    
    // Highlight connected edges and nodes
    highlightedNodeIds.clear();
    highlightedEdgeIds.clear();
    
    // Find connected edges
    const connectedEdges = data.connections.filter(
      conn => conn.source === service.id || conn.target === service.id
    );
    
    // Highlight connected nodes
    connectedEdges.forEach(edge => {
      if (edge.source !== service.id) {
        highlightedNodeIds.add(edge.source);
      }
      if (edge.target !== service.id) {
        highlightedNodeIds.add(edge.target);
      }
      highlightedEdgeIds.add(edge.id);
    });
    
    updateRenderState();
  }
  
  // Handle node leave
  function handleNodeLeave() {
    interactionState.hoveredNode = null;
    
    // Clear highlights if no node is selected
    if (!interactionState.selectedNode) {
      highlightedNodeIds.clear();
      highlightedEdgeIds.clear();
      updateRenderState();
    }
  }
  
  // Handle node click
  function handleNodeClick(service: Service, event: MouseEvent) {
    event.stopPropagation();
    
    // Toggle selection
    if (interactionState.selectedNode?.id === service.id) {
      interactionState.selectedNode = null;
      highlightedNodeIds.clear();
      highlightedEdgeIds.clear();
    } else {
      interactionState.selectedNode = service;
      
      // Highlight connected edges and nodes
      highlightedNodeIds.clear();
      highlightedEdgeIds.clear();
      
      // Find connected edges
      const connectedEdges = data.connections.filter(
        conn => conn.source === service.id || conn.target === service.id
      );
      
      // Highlight connected nodes
      connectedEdges.forEach(edge => {
        if (edge.source !== service.id) {
          highlightedNodeIds.add(edge.source);
        }
        if (edge.target !== service.id) {
          highlightedNodeIds.add(edge.target);
        }
        highlightedEdgeIds.add(edge.id);
      });
    }
    
    updateRenderState();
  }
  
  // Background click
  function handleBackgroundClick() {
    interactionState.selectedNode = null;
    interactionState.selectedEdge = null;
    highlightedNodeIds.clear();
    highlightedEdgeIds.clear();
    updateRenderState();
  }
  
  // Generate status color
  function getStatusColor(status: string): string {
    switch (status) {
      case 'healthy':
        return mergedSettings.theme.healthyColor;
      case 'warning':
        return mergedSettings.theme.warningColor;
      case 'error':
        return mergedSettings.theme.errorColor;
      default:
        return mergedSettings.theme.unknownColor;
    }
  }
  
  // Generate service icon based on type
  function getServiceIcon(type: string): string {
    switch (type) {
      case 'api':
        return 'api';
      case 'service':
        return 'code';
      case 'database':
        return 'database';
      case 'queue':
        return 'queue';
      case 'cache':
        return 'memory';
      case 'gateway':
        return 'router';
      case 'auth':
        return 'lock';
      case 'storage':
        return 'storage';
      case 'external':
        return 'cloud';
      default:
        return 'devices';
    }
  }
  
  // Debug: generate new random data
  function generateNewData() {
    data = generateMockServiceMapData();
    simulation.initialize(data.services, data.connections);
    updateRenderState();
  }
  
  // Debug: toggle performance overlay
  let showPerformance = false;
  function togglePerformance() {
    showPerformance = !showPerformance;
  }
</script>

<div 
  class="service-map-container" 
  bind:this={containerDiv}
  style="width: {width}px; height: {height}px; background-color: {mergedSettings.theme.background};"
  on:click={handleBackgroundClick}
>
  {#if typeof window !== 'undefined'}
    <!-- WebGL Canvas for rendering -->
    <canvas 
      bind:this={canvas} 
      class="webgl-canvas"
    ></canvas>
    
    <!-- SVG Layer for labels and interaction -->
    {#if mergedSettings.showLabels}
      <svg 
        bind:this={svgLabels} 
        class="label-layer"
        width={width}
        height={height}
        style="transform: {svgTransform};"
      >
        <!-- Service Labels -->
        {#each data.services as service}
          <g 
            class="service-label"
            transform="translate({service.x || 0}, {service.y || 0})"
            style="
              opacity: {
                interactionState.selectedNode && interactionState.selectedNode.id !== service.id && !highlightedNodeIds.has(service.id)
                  ? '0.3'
                  : '1'
              };
            "
            on:mouseenter={() => handleNodeHover(service)}
            on:mouseleave={handleNodeLeave}
            on:click={(e) => handleNodeClick(service, e)}
            role="button"
            tabindex="0"
            on:keydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                handleNodeClick(service, e);
              }
            }}
          >
            <!-- Service Name -->
            <text 
              class="service-name"
              y="25"
              text-anchor="middle"
              style="
                fill: {mergedSettings.theme.text}; 
                font-weight: {
                  interactionState.selectedNode?.id === service.id || 
                  interactionState.hoveredNode?.id === service.id
                    ? 'bold'
                    : 'normal'
                };
              "
            >
              {service.name}
            </text>
            
            <!-- Service Metrics (shown on hover/select) -->
            {#if (mergedSettings.showMetrics && (interactionState.hoveredNode?.id === service.id || interactionState.selectedNode?.id === service.id))}
              <g class="service-metrics">
                <rect
                  x="-70"
                  y="30"
                  width="140"
                  height="55"
                  rx="3"
                  fill="rgba(0,0,0,0.7)"
                  stroke={getStatusColor(service.status)}
                  stroke-width="1"
                ></rect>
                
                <text x="-65" y="45" class="metric-text">CPU: {service.metrics.cpuUsage.toFixed(1)}%</text>
                <text x="-65" y="60" class="metric-text">Mem: {service.metrics.memoryUsage.toFixed(1)}%</text>
                <text x="-65" y="75" class="metric-text">Lat: {service.metrics.responseTime.toFixed(0)}ms</text>
                
                <text x="10" y="45" class="metric-text">Req: {service.metrics.requestRate.toFixed(0)}/s</text>
                <text x="10" y="60" class="metric-text">Err: {service.metrics.errorRate.toFixed(1)}%</text>
                <text x="10" y="75" class="metric-text">{service.instances || 1} instance{service.instances !== 1 ? 's' : ''}</text>
              </g>
            {/if}
            
            <!-- Status Indicator (small dot below name) -->
            <circle
              cx="0"
              cy="30"
              r="3"
              fill={getStatusColor(service.status)}
            ></circle>
          </g>
        {/each}
      </svg>
    {/if}
  {:else}
    <!-- Server-side rendering fallback -->
    <div class="ssr-fallback">
      <p>Service Map</p>
    </div>
  {/if}
  
  <!-- Service Details Panel (when a service is selected) -->
  {#if typeof window !== 'undefined' && interactionState.selectedNode}
    <div class="service-details-panel">
      <div class="service-details-header" style="border-color: {getStatusColor(interactionState.selectedNode.status)};">
        <div class="service-details-title">
          <span class="service-type-badge" style="background-color: {getStatusColor(interactionState.selectedNode.status)};">
            {interactionState.selectedNode.type}
          </span>
          <h3>{interactionState.selectedNode.name}</h3>
        </div>
        <div 
          class="service-details-close" 
          on:click={handleBackgroundClick}
          role="button"
          tabindex="0"
          on:keydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              handleBackgroundClick();
            }
          }}
        >✕</div>
      </div>
      
      <div class="service-details-content">
        <div class="service-details-metrics">
          <div class="metric-group">
            <div class="metric-item">
              <div class="metric-name">Status</div>
              <div class="metric-value" style="color: {getStatusColor(interactionState.selectedNode.status)};">
                {interactionState.selectedNode.status}
              </div>
            </div>
            
            <div class="metric-item">
              <div class="metric-name">Version</div>
              <div class="metric-value">{interactionState.selectedNode.version || 'N/A'}</div>
            </div>
            
            <div class="metric-item">
              <div class="metric-name">Region</div>
              <div class="metric-value">{interactionState.selectedNode.region || 'N/A'}</div>
            </div>
            
            <div class="metric-item">
              <div class="metric-name">Instances</div>
              <div class="metric-value">{interactionState.selectedNode.instances || 1}</div>
            </div>
          </div>
          
          <div class="metric-group">
            <div class="metric-item">
              <div class="metric-name">Request Rate</div>
              <div class="metric-value">{interactionState.selectedNode.metrics.requestRate.toFixed(1)}/s</div>
            </div>
            
            <div class="metric-item">
              <div class="metric-name">Error Rate</div>
              <div class="metric-value" style="
                color: {interactionState.selectedNode.metrics.errorRate > 5 
                  ? mergedSettings.theme.errorColor 
                  : interactionState.selectedNode.metrics.errorRate > 1 
                    ? mergedSettings.theme.warningColor 
                    : mergedSettings.theme.text
                };"
              >
                {interactionState.selectedNode.metrics.errorRate.toFixed(1)}%
              </div>
            </div>
            
            <div class="metric-item">
              <div class="metric-name">Response Time</div>
              <div class="metric-value" style="
                color: {interactionState.selectedNode.metrics.responseTime > 300 
                  ? mergedSettings.theme.errorColor 
                  : interactionState.selectedNode.metrics.responseTime > 100 
                    ? mergedSettings.theme.warningColor 
                    : mergedSettings.theme.text
                };"
              >
                {interactionState.selectedNode.metrics.responseTime.toFixed(0)} ms
              </div>
            </div>
          </div>
          
          <div class="metric-group">
            <div class="metric-item">
              <div class="metric-name">CPU Usage</div>
              <div class="metric-value" style="
                color: {interactionState.selectedNode.metrics.cpuUsage > 80 
                  ? mergedSettings.theme.errorColor 
                  : interactionState.selectedNode.metrics.cpuUsage > 60 
                    ? mergedSettings.theme.warningColor 
                    : mergedSettings.theme.text
                };"
              >
                {interactionState.selectedNode.metrics.cpuUsage.toFixed(1)}%
              </div>
            </div>
            
            <div class="metric-item">
              <div class="metric-name">Memory Usage</div>
              <div class="metric-value" style="
                color: {interactionState.selectedNode.metrics.memoryUsage > 80 
                  ? mergedSettings.theme.errorColor 
                  : interactionState.selectedNode.metrics.memoryUsage > 60 
                    ? mergedSettings.theme.warningColor 
                    : mergedSettings.theme.text
                };"
              >
                {interactionState.selectedNode.metrics.memoryUsage.toFixed(1)}%
              </div>
            </div>
            
            <div class="metric-item">
              <div class="metric-name">Saturation</div>
              <div class="metric-value" style="
                color: {interactionState.selectedNode.metrics.saturation > 80 
                  ? mergedSettings.theme.errorColor 
                  : interactionState.selectedNode.metrics.saturation > 60 
                    ? mergedSettings.theme.warningColor 
                    : mergedSettings.theme.text
                };"
              >
                {interactionState.selectedNode.metrics.saturation.toFixed(1)}%
              </div>
            </div>
          </div>
        </div>
        
        <!-- Connected Services -->
        <div class="service-connections">
          <h4>Connected Services</h4>
          
          <div class="connections-list">
            {#each data.connections.filter(conn => conn.source === interactionState.selectedNode?.id || conn.target === interactionState.selectedNode?.id) as connection}
              {@const isSource = connection.source === interactionState.selectedNode?.id}
              {@const otherServiceId = isSource ? connection.target : connection.source}
              {@const otherService = data.services.find(s => s.id === otherServiceId)}
              
              {#if otherService}
                <div class="connection-item" style="border-left-color: {getStatusColor(connection.status)};">
                  <div class="connection-direction">
                    {isSource ? 'OUT →' : '← IN'}
                  </div>
                  <div class="connection-name">
                    {otherService.name}
                  </div>
                  <div class="connection-protocol">
                    {connection.protocol}
                  </div>
                  <div class="connection-metrics">
                    <span style="
                      color: {connection.metrics.latency > 300 
                        ? mergedSettings.theme.errorColor 
                        : connection.metrics.latency > 100 
                          ? mergedSettings.theme.warningColor 
                          : mergedSettings.theme.text
                      };"
                    >
                      {connection.metrics.latency.toFixed(0)}ms
                    </span>
                    <span>|</span>
                    <span style="
                      color: {connection.metrics.errorRate > 5 
                        ? mergedSettings.theme.errorColor 
                        : connection.metrics.errorRate > 1 
                          ? mergedSettings.theme.warningColor 
                          : mergedSettings.theme.text
                      };"
                    >
                      {connection.metrics.errorRate.toFixed(1)}% err
                    </span>
                  </div>
                </div>
              {/if}
            {/each}
          </div>
        </div>
      </div>
    </div>
  {/if}
  
  <!-- Performance Stats Overlay -->
  {#if showPerformance && isInitialized && renderer}
    <div class="performance-overlay">
      <div>FPS: {stats.fps}</div>
      <div>Render: {stats.renderTime.toFixed(2)}ms</div>
      <div>Physics: {stats.physicsTime.toFixed(2)}ms</div>
      <div>Nodes: {stats.nodeCount}</div>
      <div>Edges: {stats.edgeCount}</div>
    </div>
  {/if}
  
  <!-- Debug Controls -->
  {#if isInitialized && renderer}
    <div class="debug-controls">
      <button on:click={generateNewData}>Generate New Data</button>
      <button on:click={togglePerformance}>Toggle Performance</button>
      <button on:click={() => applyLayout('force')}>Force Layout</button>
      <button on:click={() => applyLayout('circular')}>Circular Layout</button>
      <button on:click={() => applyLayout('hierarchical')}>Hierarchical Layout</button>
    </div>
  {:else}
    <div class="fallback-message">
      <p>WebGL rendering is not available on your browser or device.</p>
      <p>Please try a modern browser with WebGL support.</p>
    </div>
  {/if}
</div>

<style>
  .service-map-container {
    position: relative;
    overflow: hidden;
    font-family: 'Inter', 'Roboto Mono', monospace;
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-md);
  }
  
  .ssr-fallback {
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: var(--bg-dark);
    color: var(--text-bright);
    font-weight: bold;
    font-size: 1.5rem;
  }
  
  .webgl-canvas {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }
  
  .label-layer {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
  
  .service-label {
    pointer-events: all;
    cursor: pointer;
    user-select: none;
  }
  
  .service-name {
    font-size: 12px;
    fill: var(--text-bright);
    text-shadow: 0 0 2px rgba(0, 0, 0, 0.8);
  }
  
  .metric-text {
    font-size: 10px;
    fill: var(--text-bright);
    text-shadow: 0 0 2px rgba(0, 0, 0, 0.5);
  }
  
  .service-details-panel {
    position: absolute;
    top: 16px;
    right: 16px;
    width: 320px;
    background-color: var(--bg-darker);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-lg);
    z-index: 10;
    overflow: hidden;
    border: 1px solid var(--border-medium);
  }
  
  .service-details-header {
    padding: 12px 16px;
    background-color: var(--bg-dark);
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 2px solid var(--primary);
  }
  
  .service-details-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .service-details-title h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-brightest);
  }
  
  .service-type-badge {
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 10px;
    font-weight: 600;
    color: var(--bg-darkest);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .service-details-close {
    cursor: pointer;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background-color: var(--bg-medium);
    color: var(--text-medium);
    font-size: 12px;
  }
  
  .service-details-close:hover {
    background-color: var(--bg-dark);
    color: var(--text-bright);
  }
  
  .service-details-content {
    padding: 16px;
  }
  
  .service-details-metrics {
    display: flex;
    flex-direction: column;
    gap: 16px;
    margin-bottom: 16px;
  }
  
  .metric-group {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  
  .metric-item {
    background-color: var(--bg-dark);
    padding: 8px 10px;
    border-radius: var(--radius-sm);
  }
  
  .metric-name {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-medium);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }
  
  .metric-value {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-bright);
  }
  
  .service-connections h4 {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-bright);
    margin: 8px 0;
  }
  
  .connections-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 200px;
    overflow-y: auto;
  }
  
  .connection-item {
    display: grid;
    grid-template-columns: auto 1fr auto auto;
    gap: 8px;
    padding: 8px;
    background-color: var(--bg-dark);
    border-radius: var(--radius-sm);
    font-size: 12px;
    align-items: center;
    border-left: 3px solid transparent;
  }
  
  .connection-direction {
    font-weight: 600;
    color: var(--text-medium);
    font-size: 10px;
  }
  
  .connection-name {
    color: var(--text-bright);
  }
  
  .connection-protocol {
    background-color: var(--bg-medium);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 10px;
    color: var(--text-medium);
  }
  
  .connection-metrics {
    font-size: 10px;
    color: var(--text-medium);
    display: flex;
    gap: 4px;
  }
  
  .performance-overlay {
    position: absolute;
    top: 10px;
    left: 10px;
    background-color: rgba(0, 0, 0, 0.7);
    padding: 8px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    color: var(--text-bright);
    font-family: monospace;
    z-index: 100;
  }
  
  .debug-controls {
    position: absolute;
    bottom: 10px;
    left: 10px;
    display: flex;
    gap: 8px;
    z-index: 100;
  }
  
  .debug-controls button {
    background-color: var(--bg-medium);
    border: 1px solid var(--border-medium);
    color: var(--text-bright);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    cursor: pointer;
  }
  
  .debug-controls button:hover {
    background-color: var(--bg-dark);
    border-color: var(--primary);
  }
  
  .fallback-message {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    text-align: center;
    background-color: var(--bg-medium);
    padding: 2rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-medium);
    box-shadow: var(--shadow-lg);
    max-width: 400px;
  }
  
  .fallback-message p {
    margin: 0.5rem 0;
    color: var(--text-bright);
  }
  
  .fallback-message p:first-child {
    font-weight: bold;
    font-size: 1.1rem;
    color: var(--error-coral);
  }
</style>