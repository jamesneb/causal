<script lang="ts">
  import { page } from '$app/stores';
  import { TraceViewer } from '../../../components/trace-viewer';
  import type { Trace, Span, TimelineEvent } from '../../../components/timeline/types';
  import { onMount } from 'svelte';
  
  // Get trace ID from route
  const traceId = $page.params.id;
  
  // Sample trace data (same as in the traces page)
  const sampleTrace: Trace = {
    id: 'trace-123456789',
    name: 'GET /api/products',
    startTime: Date.now() - 350,
    endTime: Date.now(),
    duration: 350,
    status: 'success',
    spans: [
      {
        id: 'span-1',
        name: 'HTTP GET /api/products',
        serviceName: 'api-gateway',
        startTime: Date.now() - 350,
        endTime: Date.now() - 320,
        duration: 30,
        status: 'success'
      },
      {
        id: 'span-2',
        name: 'Authenticate Request',
        serviceName: 'auth-service',
        startTime: Date.now() - 315,
        endTime: Date.now() - 290,
        duration: 25,
        status: 'success',
        parentId: 'span-1'
      },
      {
        id: 'span-3',
        name: 'Get Products',
        serviceName: 'product-service',
        startTime: Date.now() - 285,
        endTime: Date.now() - 190,
        duration: 95,
        status: 'success',
        parentId: 'span-1',
        attributes: {
          'http.method': 'GET',
          'http.url': '/internal/products',
          'db.statement': 'SELECT * FROM products LIMIT 20',
          'products.count': 15
        }
      },
      {
        id: 'span-4',
        name: 'DB Query: Get Products',
        serviceName: 'database',
        startTime: Date.now() - 260,
        endTime: Date.now() - 210,
        duration: 50,
        status: 'success',
        parentId: 'span-3'
      },
      {
        id: 'span-5',
        name: 'Get Product Inventory',
        serviceName: 'product-service',
        startTime: Date.now() - 185,
        endTime: Date.now() - 100,
        duration: 85,
        status: 'warning',
        parentId: 'span-1',
        attributes: {
          'http.method': 'GET',
          'http.url': '/internal/inventory',
          'warning.reason': 'High latency detected'
        }
      },
      {
        id: 'span-6',
        name: 'DB Query: Get Inventory',
        serviceName: 'database',
        startTime: Date.now() - 175,
        endTime: Date.now() - 110,
        duration: 65,
        status: 'warning',
        parentId: 'span-5'
      },
      {
        id: 'span-7',
        name: 'Get User Preferences',
        serviceName: 'user-service',
        startTime: Date.now() - 95,
        endTime: Date.now() - 70,
        duration: 25,
        status: 'success',
        parentId: 'span-1'
      },
      {
        id: 'span-8',
        name: 'Format Response',
        serviceName: 'api-gateway',
        startTime: Date.now() - 65,
        endTime: Date.now() - 5,
        duration: 60,
        status: 'success',
        parentId: 'span-1'
      }
    ]
  };
  
  const sampleEvents: TimelineEvent[] = [
    {
      id: 'event-1',
      timestamp: Date.now() - 260,
      name: 'Breakpoint: DB Query',
      type: 'breakpoint',
      serviceName: 'database',
      description: 'Conditional breakpoint on product query'
    },
    {
      id: 'event-2',
      timestamp: Date.now() - 175,
      name: 'High Latency Warning',
      type: 'warning',
      serviceName: 'database',
      description: 'Database query exceeding 50ms threshold'
    }
  ];
  
  // Causal inference data
  const causalInferences = [
    {
      id: 'ci-1',
      type: 'anomaly',
      description: 'The inventory database query has higher latency than normal',
      severity: 'warning',
      spanId: 'span-6',
      evidence: 'This query typically takes 25-35ms but took 65ms in this trace',
      recommendation: 'Check database indexing on inventory table'
    },
    {
      id: 'ci-2',
      type: 'causality',
      description: 'The high inventory query latency is the root cause of the overall trace slowdown',
      severity: 'info',
      spanId: 'span-6',
      parentEffectId: 'span-5',
      evidence: '76% of the "Get Product Inventory" span time is spent in database query',
      confidence: 0.92
    },
    {
      id: 'ci-3',
      type: 'pattern',
      description: 'Similar pattern observed in 15 other traces in the last hour',
      severity: 'warning',
      spanId: 'span-6',
      evidence: 'Database queries for inventory data consistently taking 60-70ms',
      similarTraces: ['trace-987654', 'trace-876543', 'trace-765432']
    }
  ];
  
  // State
  let activeTab: 'timeline' | 'causal' = 'timeline';
  let selectedInferenceId: string | null = null;
  
  // Computed properties
  $: selectedInference = causalInferences.find(ci => ci.id === selectedInferenceId);
  
  // Handle trace view
  let expandedSpanId: string | null = null;
  
  // Initialize SVG for causal graph
  let graphSvg;
  
  onMount(() => {
    if (activeTab === 'causal') {
      drawCausalGraph();
    }
  });
  
  // Draw causal graph whenever the active tab changes
  $: if (activeTab === 'causal' && graphSvg) {
    drawCausalGraph();
  }
  
  // Draw causal inference graph
  function drawCausalGraph() {
    if (!graphSvg) return;
    
    // Clear SVG
    while (graphSvg.firstChild) {
      graphSvg.removeChild(graphSvg.firstChild);
    }
    
    // Define graph layout data
    const nodes = [
      { id: 'span-1', label: 'API Gateway', x: 100, y: 50, type: 'root' },
      { id: 'span-3', label: 'Get Products', x: 250, y: 30, type: 'normal' },
      { id: 'span-4', label: 'DB: Products', x: 400, y: 30, type: 'normal' },
      { id: 'span-5', label: 'Get Inventory', x: 250, y: 80, type: 'normal' },
      { id: 'span-6', label: 'DB: Inventory', x: 400, y: 80, type: 'anomaly' },
      { id: 'span-7', label: 'Get User Prefs', x: 250, y: 130, type: 'normal' },
      { id: 'span-8', label: 'Format Response', x: 550, y: 50, type: 'normal' }
    ];
    
    const links = [
      { source: 'span-1', target: 'span-3', type: 'flow' },
      { source: 'span-3', target: 'span-4', type: 'flow' },
      { source: 'span-1', target: 'span-5', type: 'flow' },
      { source: 'span-5', target: 'span-6', type: 'causal' },
      { source: 'span-1', target: 'span-7', type: 'flow' },
      { source: 'span-1', target: 'span-8', type: 'flow' },
      { source: 'span-6', target: 'span-5', type: 'causal' }
    ];
    
    // Set up SVG namespace
    const svgNS = "http://www.w3.org/2000/svg";
    
    // Draw links first (so they're underneath nodes)
    links.forEach(link => {
      const sourceNode = nodes.find(n => n.id === link.source);
      const targetNode = nodes.find(n => n.id === link.target);
      
      if (!sourceNode || !targetNode) return;
      
      // Create path element
      const path = document.createElementNS(svgNS, "path");
      
      // Define path
      const sourceX = sourceNode.x + 60; // Width of node/2
      const sourceY = sourceNode.y + 20; // Height of node/2
      const targetX = targetNode.x + 60;
      const targetY = targetNode.y + 20;
      
      // Create curved path
      const controlPointX = (sourceX + targetX) / 2;
      const controlPointY = (sourceY + targetY) / 2;
      
      // Path data
      let d = `M ${sourceX} ${sourceY} Q ${controlPointX} ${controlPointY} ${targetX} ${targetY}`;
      
      // Set attributes
      path.setAttribute("d", d);
      path.setAttribute("fill", "none");
      
      if (link.type === 'flow') {
        path.setAttribute("stroke", "rgba(255, 255, 255, 0.3)");
        path.setAttribute("stroke-width", "2");
      } else if (link.type === 'causal') {
        path.setAttribute("stroke", "#FF3864"); // Plasma Pink
        path.setAttribute("stroke-width", "3");
        path.setAttribute("stroke-dasharray", "4 2");
        
        // Add arrowhead for causal links
        const marker = document.createElementNS(svgNS, "marker");
        marker.setAttribute("id", `arrow-${link.source}-${link.target}`);
        marker.setAttribute("viewBox", "0 0 10 10");
        marker.setAttribute("refX", "5");
        marker.setAttribute("refY", "5");
        marker.setAttribute("markerWidth", "6");
        marker.setAttribute("markerHeight", "6");
        marker.setAttribute("orient", "auto");
        
        const arrowPath = document.createElementNS(svgNS, "path");
        arrowPath.setAttribute("d", "M 0 0 L 10 5 L 0 10 z");
        arrowPath.setAttribute("fill", "#FF3864");
        
        marker.appendChild(arrowPath);
        graphSvg.appendChild(marker);
        
        path.setAttribute("marker-end", `url(#arrow-${link.source}-${link.target})`);
      }
      
      graphSvg.appendChild(path);
    });
    
    // Draw nodes
    nodes.forEach(node => {
      // Create node group
      const group = document.createElementNS(svgNS, "g");
      group.setAttribute("transform", `translate(${node.x}, ${node.y})`);
      group.setAttribute("data-id", node.id);
      group.addEventListener("click", () => {
        expandedSpanId = node.id;
        activeTab = 'timeline';
      });
      
      // Create node rectangle
      const rect = document.createElementNS(svgNS, "rect");
      rect.setAttribute("width", "120");
      rect.setAttribute("height", "40");
      rect.setAttribute("rx", "6");
      rect.setAttribute("ry", "6");
      
      // Style based on node type
      if (node.type === 'root') {
        rect.setAttribute("fill", "rgba(138, 43, 226, 0.3)"); // Quantum Purple
        rect.setAttribute("stroke", "#8A2BE2");
      } else if (node.type === 'anomaly') {
        rect.setAttribute("fill", "rgba(255, 56, 100, 0.3)"); // Plasma Pink
        rect.setAttribute("stroke", "#FF3864");
        rect.setAttribute("stroke-width", "2");
      } else {
        rect.setAttribute("fill", "rgba(26, 22, 48, 0.8)"); // BG tertiary
        rect.setAttribute("stroke", "rgba(255, 255, 255, 0.3)");
      }
      
      group.appendChild(rect);
      
      // Create node label
      const text = document.createElementNS(svgNS, "text");
      text.setAttribute("x", "60");
      text.setAttribute("y", "22");
      text.setAttribute("text-anchor", "middle");
      text.setAttribute("dominant-baseline", "middle");
      text.setAttribute("fill", "rgba(255, 255, 255, 0.9)");
      text.setAttribute("font-size", "12");
      text.textContent = node.label;
      
      group.appendChild(text);
      
      // Add highlight effect for anomaly
      if (node.type === 'anomaly') {
        const highlight = document.createElementNS(svgNS, "rect");
        highlight.setAttribute("width", "120");
        highlight.setAttribute("height", "40");
        highlight.setAttribute("rx", "6");
        highlight.setAttribute("ry", "6");
        highlight.setAttribute("fill", "none");
        highlight.setAttribute("stroke", "#FF3864");
        highlight.setAttribute("stroke-width", "2");
        highlight.setAttribute("stroke-dasharray", "4 2");
        highlight.setAttribute("class", "pulse-animation");
        
        const style = document.createElementNS(svgNS, "style");
        style.textContent = `
          @keyframes pulse {
            0% { stroke-opacity: 0.3; }
            50% { stroke-opacity: 1; }
            100% { stroke-opacity: 0.3; }
          }
          .pulse-animation {
            animation: pulse 2s infinite;
          }
        `;
        
        graphSvg.appendChild(style);
        group.appendChild(highlight);
      }
      
      graphSvg.appendChild(group);
    });
  }
</script>

<div class="trace-detail-page">
  <div class="trace-header">
    <div class="trace-title">
      <h1>Trace Details</h1>
      <div class="trace-id">ID: {traceId}</div>
    </div>
    
    <div class="tab-buttons">
      <button 
        class="tab-button" 
        class:active={activeTab === 'timeline'} 
        on:click={() => activeTab = 'timeline'}
      >
        Timeline View
      </button>
      
      <button 
        class="tab-button" 
        class:active={activeTab === 'causal'} 
        on:click={() => activeTab = 'causal'}
      >
        Causal Inference
      </button>
    </div>
  </div>
  
  <div class="trace-content">
    {#if activeTab === 'timeline'}
      <div class="timeline-view">
        <TraceViewer 
          trace={sampleTrace} 
          events={sampleEvents}
          expandedSpanId={expandedSpanId}
        />
      </div>
    {:else if activeTab === 'causal'}
      <div class="causal-view">
        <div class="causal-content">
          <div class="causal-graph-container">
            <div class="card">
              <div class="card-header">
                <h2>Causal Graph</h2>
                <div class="card-description">
                  This graph shows the causal relationships between spans
                </div>
              </div>
              
              <div class="graph-container">
                <svg 
                  bind:this={graphSvg} 
                  width="700" 
                  height="200" 
                  viewBox="0 0 700 200"
                ></svg>
                
                <div class="graph-legend">
                  <div class="legend-item">
                    <div class="legend-color normal"></div>
                    <div class="legend-label">Normal Span</div>
                  </div>
                  
                  <div class="legend-item">
                    <div class="legend-color anomaly"></div>
                    <div class="legend-label">Anomaly</div>
                  </div>
                  
                  <div class="legend-item">
                    <div class="legend-line flow"></div>
                    <div class="legend-label">Request Flow</div>
                  </div>
                  
                  <div class="legend-item">
                    <div class="legend-line causal"></div>
                    <div class="legend-label">Causal Relationship</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
          
          <div class="inference-container">
            <div class="inferences-list-container card">
              <div class="card-header">
                <h2>Causal Inferences</h2>
              </div>
              
              <div class="inferences-list">
                {#each causalInferences as inference}
                  <div 
                    class="inference-item" 
                    class:selected={selectedInferenceId === inference.id}
                    class:warning={inference.severity === 'warning'}
                    class:error={inference.severity === 'error'}
                    on:click={() => selectedInferenceId = inference.id}
                  >
                    <div class="inference-type-label" class:warning={inference.severity === 'warning'} class:info={inference.severity === 'info'} class:error={inference.severity === 'error'}>
                      {inference.type}
                    </div>
                    
                    <div class="inference-description">
                      {inference.description}
                    </div>
                  </div>
                {/each}
              </div>
            </div>
            
            <div class="inference-detail-container card">
              <div class="card-header">
                <h2>Inference Details</h2>
              </div>
              
              {#if selectedInference}
                <div class="inference-detail-content">
                  <div class="inference-header">
                    <div class="inference-title">
                      <div class="inference-badge" class:warning={selectedInference.severity === 'warning'} class:info={selectedInference.severity === 'info'} class:error={selectedInference.severity === 'error'}>
                        {selectedInference.type}
                      </div>
                      <h3>{selectedInference.description}</h3>
                    </div>
                  </div>
                  
                  <div class="inference-sections">
                    <div class="inference-section">
                      <h4>Evidence</h4>
                      <p>{selectedInference.evidence}</p>
                    </div>
                    
                    {#if selectedInference.recommendation}
                      <div class="inference-section">
                        <h4>Recommendation</h4>
                        <p>{selectedInference.recommendation}</p>
                      </div>
                    {/if}
                    
                    {#if selectedInference.confidence}
                      <div class="inference-section">
                        <h4>Confidence</h4>
                        <div class="confidence-meter">
                          <div class="confidence-bar" style="width: {selectedInference.confidence * 100}%"></div>
                          <div class="confidence-value">{(selectedInference.confidence * 100).toFixed(0)}%</div>
                        </div>
                      </div>
                    {/if}
                    
                    {#if selectedInference.similarTraces}
                      <div class="inference-section">
                        <h4>Similar Traces</h4>
                        <div class="similar-traces">
                          {#each selectedInference.similarTraces as traceId}
                            <a href={`/traces/${traceId}`} class="trace-link">{traceId}</a>
                          {/each}
                        </div>
                      </div>
                    {/if}
                    
                    <div class="inference-section">
                      <h4>Related Span</h4>
                      <button 
                        class="btn-primary" 
                        on:click={() => {
                          expandedSpanId = selectedInference.spanId;
                          activeTab = 'timeline';
                        }}
                      >
                        View in Timeline
                      </button>
                    </div>
                  </div>
                </div>
              {:else}
                <div class="inference-detail-placeholder">
                  <p>Select an inference to view details</p>
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .trace-detail-page {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .trace-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
  }
  
  .trace-title h1 {
    font-size: 1.8rem;
    font-weight: 700;
    margin: 0 0 0.5rem 0;
  }
  
  .trace-id {
    font-family: monospace;
    font-size: 0.9rem;
    color: rgba(255, 255, 255, 0.7);
  }
  
  .tab-buttons {
    display: flex;
    gap: 0.5rem;
  }
  
  .tab-button {
    padding: 0.6rem 1.25rem;
    background: none;
    border: 1px solid var(--border-light);
    border-radius: 6px;
    color: var(--text-light);
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .tab-button:hover {
    background-color: var(--bg-tertiary);
  }
  
  .tab-button.active {
    background-color: var(--bg-tertiary);
    border-color: var(--electric-teal);
    color: var(--electric-teal);
  }
  
  .trace-content {
    flex: 1;
    min-height: 0;
  }
  
  .timeline-view {
    height: 100%;
  }
  
  .causal-view {
    height: 100%;
  }
  
  .causal-content {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    height: 100%;
  }
  
  .causal-graph-container {
    min-height: 250px;
  }
  
  .card {
    background-color: var(--bg-secondary);
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    border: 1px solid var(--border-light);
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  
  .card-header {
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-light);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  
  .card-header h2 {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0;
  }
  
  .card-description {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.6);
  }
  
  .graph-container {
    flex: 1;
    position: relative;
    padding: 1rem;
  }
  
  .graph-legend {
    position: absolute;
    bottom: 1rem;
    right: 1rem;
    background-color: rgba(18, 16, 35, 0.8);
    border: 1px solid var(--border-light);
    border-radius: 6px;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.8);
  }
  
  .legend-color {
    width: 16px;
    height: 16px;
    border-radius: 4px;
  }
  
  .legend-color.normal {
    background-color: rgba(26, 22, 48, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.3);
  }
  
  .legend-color.anomaly {
    background-color: rgba(255, 56, 100, 0.3);
    border: 1px solid #FF3864;
  }
  
  .legend-line {
    width: 24px;
    height: 2px;
  }
  
  .legend-line.flow {
    background-color: rgba(255, 255, 255, 0.3);
  }
  
  .legend-line.causal {
    height: 3px;
    background-color: #FF3864;
    position: relative;
  }
  
  .legend-line.causal::after {
    content: "";
    position: absolute;
    right: -2px;
    top: -3px;
    width: 0;
    height: 0;
    border-style: solid;
    border-width: 4px 0 4px 6px;
    border-color: transparent transparent transparent #FF3864;
  }
  
  .inference-container {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
    min-height: 0;
  }
  
  .inferences-list-container, .inference-detail-container {
    min-height: 0;
  }
  
  .inferences-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.75rem;
  }
  
  .inference-item {
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 0.5rem;
    cursor: pointer;
    border-left: 3px solid var(--electric-teal);
    background-color: rgba(18, 16, 35, 0.8);
    transition: all 0.2s ease;
  }
  
  .inference-item:hover {
    background-color: var(--bg-tertiary);
  }
  
  .inference-item.selected {
    background-color: var(--bg-tertiary);
  }
  
  .inference-item.warning {
    border-left-color: var(--warning);
  }
  
  .inference-item.error {
    border-left-color: var(--error);
  }
  
  .inference-type-label {
    display: inline-block;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    text-transform: uppercase;
    font-weight: 500;
    margin-bottom: 0.5rem;
    background-color: rgba(0, 245, 212, 0.2);
    color: var(--electric-teal);
  }
  
  .inference-type-label.warning {
    background-color: rgba(255, 214, 0, 0.2);
    color: var(--warning);
  }
  
  .inference-type-label.error {
    background-color: rgba(255, 56, 100, 0.2);
    color: var(--error);
  }
  
  .inference-type-label.info {
    background-color: rgba(66, 165, 245, 0.2);
    color: var(--info);
  }
  
  .inference-description {
    font-size: 0.9rem;
  }
  
  .inference-detail-content {
    padding: 1.25rem;
    overflow-y: auto;
  }
  
  .inference-header {
    margin-bottom: 1.5rem;
  }
  
  .inference-title {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .inference-badge {
    align-self: flex-start;
    padding: 0.25rem 0.75rem;
    border-radius: 50px;
    font-size: 0.8rem;
    text-transform: uppercase;
    font-weight: 500;
    background-color: rgba(0, 245, 212, 0.2);
    color: var(--electric-teal);
  }
  
  .inference-badge.warning {
    background-color: rgba(255, 214, 0, 0.2);
    color: var(--warning);
  }
  
  .inference-badge.error {
    background-color: rgba(255, 56, 100, 0.2);
    color: var(--error);
  }
  
  .inference-badge.info {
    background-color: rgba(66, 165, 245, 0.2);
    color: var(--info);
  }
  
  .inference-title h3 {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0;
  }
  
  .inference-sections {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }
  
  .inference-section {
    background-color: var(--bg-tertiary);
    border-radius: 8px;
    padding: 1rem;
  }
  
  .inference-section h4 {
    font-size: 0.95rem;
    font-weight: 600;
    margin: 0 0 0.75rem 0;
    color: rgba(255, 255, 255, 0.9);
  }
  
  .inference-section p {
    margin: 0;
    font-size: 0.9rem;
    line-height: 1.5;
  }
  
  .confidence-meter {
    height: 8px;
    background-color: rgba(0, 0, 0, 0.2);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 0.5rem;
    position: relative;
  }
  
  .confidence-bar {
    height: 100%;
    background: linear-gradient(90deg, var(--electric-teal), var(--quantum-purple));
    border-radius: 4px;
  }
  
  .confidence-value {
    font-size: 0.9rem;
    font-weight: 500;
  }
  
  .similar-traces {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .trace-link {
    font-family: monospace;
    color: var(--electric-teal);
    text-decoration: none;
    font-size: 0.9rem;
  }
  
  .trace-link:hover {
    text-decoration: underline;
  }
  
  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    background-color: var(--electric-teal);
    color: var(--deep-space);
    border: none;
    border-radius: 4px;
    padding: 0.5rem 0.75rem;
    font-weight: 600;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .btn-primary:hover {
    filter: brightness(1.1);
  }
  
  .inference-detail-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: rgba(255, 255, 255, 0.5);
    font-size: 0.95rem;
  }
</style>