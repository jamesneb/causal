<script lang="ts">
  import { onMount } from 'svelte';
  import type { Trace, Span } from '../timeline/types';
  
  // Props
  export let trace: Trace | null = null;
  export let width: number = 800;
  export let height: number = 400;
  export let onCauseIdentified = (spanId: string) => {};
  
  // Internal state
  let canvasElement: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let causesMap: Map<string, string[]> = new Map();
  let highlightedSpanId: string | null = null;
  let animationFrame: number;
  
  // Setup canvas
  onMount(() => {
    if (!canvasElement) return;
    
    ctx = canvasElement.getContext('2d')!;
    window.addEventListener('resize', handleResize);
    handleResize();
    
    // Process trace to build causality graph
    if (trace?.spans) {
      buildCausalityGraph(trace.spans);
    }
    
    // Start animation loop
    animationFrame = requestAnimationFrame(draw);
    
    return () => {
      window.removeEventListener('resize', handleResize);
      cancelAnimationFrame(animationFrame);
    };
  });
  
  // Handle resize
  function handleResize() {
    const { width: containerWidth, height: containerHeight } = canvasElement.parentElement!.getBoundingClientRect();
    canvasElement.width = containerWidth;
    canvasElement.height = containerHeight;
    width = containerWidth;
    height = containerHeight;
    draw();
  }
  
  // Build causality graph from spans
  function buildCausalityGraph(spans: Span[]) {
    causesMap.clear();
    
    // First pass - create parent-child relationships
    spans.forEach(span => {
      if (span.parentId) {
        if (!causesMap.has(span.id)) {
          causesMap.set(span.id, []);
        }
        
        if (!causesMap.has(span.parentId)) {
          causesMap.set(span.parentId, []);
        }
        
        // Parent causes child
        const causes = causesMap.get(span.parentId)!;
        if (!causes.includes(span.id)) {
          causes.push(span.id);
        }
      }
    });
    
    // Second pass - add more sophisticated causal relationships
    spans.forEach(span => {
      if (span.status === 'error') {
        // Find spans that overlap with this error span
        const overlappingSpans = spans.filter(s => 
          s.id !== span.id && 
          ((s.startTime <= span.startTime && s.endTime >= span.startTime) ||
           (s.startTime <= span.endTime && s.endTime >= span.endTime) ||
           (s.startTime >= span.startTime && s.endTime <= span.endTime))
        );
        
        // Potential causal relationships for errors
        overlappingSpans.forEach(otherSpan => {
          // Only consider spans that started before the error
          if (otherSpan.startTime < span.startTime) {
            if (!causesMap.has(otherSpan.id)) {
              causesMap.set(otherSpan.id, []);
            }
            
            // The other span may have contributed to this error
            const causes = causesMap.get(otherSpan.id)!;
            if (!causes.includes(span.id)) {
              causes.push(span.id);
            }
          }
        });
      }
      
      // Add temporal dependencies for spans within the same service
      const sameServiceSpans = spans.filter(s => 
        s.id !== span.id && 
        s.serviceName === span.serviceName &&
        s.startTime < span.startTime &&
        s.endTime <= span.startTime
      );
      
      // Add causal relationships for temporal dependencies
      sameServiceSpans.forEach(otherSpan => {
        if (!causesMap.has(otherSpan.id)) {
          causesMap.set(otherSpan.id, []);
        }
        
        const causes = causesMap.get(otherSpan.id)!;
        if (!causes.includes(span.id)) {
          causes.push(span.id);
        }
      });
    });
  }
  
  // Draw the causality graph
  function draw() {
    if (!ctx || !trace?.spans) return;
    
    // Clear canvas
    ctx.clearRect(0, 0, width, height);
    
    // Draw background
    ctx.fillStyle = '#121023';
    ctx.fillRect(0, 0, width, height);
    
    // Group spans by service
    const serviceGroups = groupSpansByService(trace.spans);
    const services = Object.keys(serviceGroups);
    
    // Calculate positions for nodes
    const nodePositions = calculateNodePositions(trace.spans, services);
    
    // Draw connections first (so they're behind nodes)
    drawConnections(nodePositions);
    
    // Draw nodes
    drawNodes(nodePositions);
    
    // Continue animation loop
    animationFrame = requestAnimationFrame(draw);
  }
  
  // Group spans by service
  function groupSpansByService(spans: Span[]): Record<string, Span[]> {
    const result: Record<string, Span[]> = {};
    
    spans.forEach(span => {
      if (!result[span.serviceName]) {
        result[span.serviceName] = [];
      }
      
      result[span.serviceName].push(span);
    });
    
    return result;
  }
  
  // Calculate positions for nodes
  function calculateNodePositions(spans: Span[], services: string[]): Map<string, {x: number, y: number, span: Span}> {
    const positions = new Map<string, {x: number, y: number, span: Span}>();
    const serviceSpacing = height / (services.length + 1);
    const servicePositions: Record<string, number> = {};
    
    // Assign y-positions to services
    services.forEach((service, index) => {
      servicePositions[service] = (index + 1) * serviceSpacing;
    });
    
    // Find min and max times
    const startTime = trace?.startTime || 0;
    const endTime = trace?.endTime || 0;
    const timeRange = endTime - startTime;
    
    // Position spans horizontally based on start time
    spans.forEach(span => {
      const timePosition = (span.startTime - startTime) / timeRange;
      const xPos = 50 + timePosition * (width - 100);
      const yPos = servicePositions[span.serviceName];
      
      positions.set(span.id, {
        x: xPos,
        y: yPos,
        span
      });
    });
    
    return positions;
  }
  
  // Draw connections between nodes
  function drawConnections(positions: Map<string, {x: number, y: number, span: Span}>) {
    // Draw parent-child connections
    positions.forEach((pos, spanId) => {
      const span = pos.span;
      
      if (span.parentId) {
        const parentPos = positions.get(span.parentId);
        if (parentPos) {
          // Draw direct parent-child connection
          ctx.beginPath();
          ctx.moveTo(parentPos.x, parentPos.y);
          ctx.lineTo(pos.x, pos.y);
          ctx.strokeStyle = 'rgba(255, 255, 255, 0.2)';
          ctx.lineWidth = 1;
          ctx.stroke();
        }
      }
    });
    
    // Draw causal inference connections
    causesMap.forEach((effects, causeId) => {
      const causePos = positions.get(causeId);
      
      if (causePos) {
        effects.forEach(effectId => {
          const effectPos = positions.get(effectId);
          
          if (effectPos) {
            // Compute control points for the curve
            const midX = (causePos.x + effectPos.x) / 2;
            const midY = (causePos.y + effectPos.y) / 2;
            const curveFactor = 0.5;
            
            // Draw curved causal connection
            ctx.beginPath();
            ctx.moveTo(causePos.x, causePos.y);
            ctx.quadraticCurveTo(
              midX + (Math.abs(causePos.y - effectPos.y) * curveFactor), 
              midY, 
              effectPos.x, 
              effectPos.y
            );
            
            // Highlight connections for highlighted span
            if (highlightedSpanId === causeId || highlightedSpanId === effectId) {
              ctx.strokeStyle = '#00F5D4'; // Electric Teal
              ctx.lineWidth = 2;
            } else {
              // Causal connections are more prominent than parent-child
              ctx.strokeStyle = 'rgba(138, 43, 226, 0.4)'; // Quantum Purple
              ctx.lineWidth = 1.5;
            }
            
            ctx.stroke();
            
            // Draw arrow at the end of the line
            const dx = effectPos.x - midX;
            const dy = effectPos.y - midY;
            const angle = Math.atan2(dy, dx);
            
            const arrowSize = 6;
            
            ctx.beginPath();
            ctx.moveTo(
              effectPos.x - arrowSize * Math.cos(angle - Math.PI / 6),
              effectPos.y - arrowSize * Math.sin(angle - Math.PI / 6)
            );
            ctx.lineTo(effectPos.x, effectPos.y);
            ctx.lineTo(
              effectPos.x - arrowSize * Math.cos(angle + Math.PI / 6),
              effectPos.y - arrowSize * Math.sin(angle + Math.PI / 6)
            );
            
            if (highlightedSpanId === causeId || highlightedSpanId === effectId) {
              ctx.fillStyle = '#00F5D4'; // Electric Teal
            } else {
              ctx.fillStyle = 'rgba(138, 43, 226, 0.4)'; // Quantum Purple
            }
            
            ctx.fill();
          }
        });
      }
    });
  }
  
  // Draw nodes representing spans
  function drawNodes(positions: Map<string, {x: number, y: number, span: Span}>) {
    positions.forEach((pos, spanId) => {
      const span = pos.span;
      
      // Node radius based on span duration
      const minRadius = 5;
      const maxRadius = 15;
      const maxDuration = 200; // ms
      const radius = minRadius + Math.min(span.duration / maxDuration, 1) * (maxRadius - minRadius);
      
      // Node color based on status
      let nodeColor;
      switch (span.status) {
        case 'error':
          nodeColor = '#FF3864'; // Plasma Pink
          break;
        case 'warning':
          nodeColor = '#FFD600'; // Signal Yellow
          break;
        default:
          nodeColor = '#00F5D4'; // Electric Teal
      }
      
      // Highlight effects
      const isHighlighted = spanId === highlightedSpanId;
      const isCause = highlightedSpanId && causesMap.get(highlightedSpanId)?.includes(spanId);
      const isEffect = highlightedSpanId && causesMap.get(spanId)?.includes(highlightedSpanId);
      
      // Draw glow effect for highlighted nodes
      if (isHighlighted || isCause || isEffect) {
        ctx.beginPath();
        ctx.arc(pos.x, pos.y, radius + 6, 0, Math.PI * 2);
        
        if (isHighlighted) {
          ctx.fillStyle = `rgba(${hexToRgb(nodeColor)}, 0.3)`;
        } else if (isCause) {
          ctx.fillStyle = 'rgba(255, 56, 100, 0.3)'; // Plasma Pink glow for causes
        } else if (isEffect) {
          ctx.fillStyle = 'rgba(0, 245, 212, 0.3)'; // Electric Teal glow for effects
        }
        
        ctx.fill();
      }
      
      // Draw node
      ctx.beginPath();
      ctx.arc(pos.x, pos.y, radius, 0, Math.PI * 2);
      ctx.fillStyle = nodeColor;
      ctx.fill();
      
      // Draw node border
      ctx.lineWidth = isHighlighted ? 3 : 1.5;
      ctx.strokeStyle = isHighlighted ? '#FFFFFF' : 'rgba(255, 255, 255, 0.5)';
      ctx.stroke();
      
      // Draw span name on hover
      if (isHighlighted || isCause || isEffect) {
        ctx.font = '12px Arial';
        ctx.fillStyle = '#FFFFFF';
        ctx.textAlign = 'center';
        ctx.fillText(span.name, pos.x, pos.y - radius - 10);
        
        // Add service name
        ctx.font = '10px Arial';
        ctx.fillStyle = 'rgba(255, 255, 255, 0.7)';
        ctx.fillText(span.serviceName, pos.x, pos.y - radius - 25);
        
        // Add duration
        ctx.fillText(`${span.duration.toFixed(2)}ms`, pos.x, pos.y + radius + 15);
      }
    });
  }
  
  // Handle mouse move
  function handleMouseMove(event: MouseEvent) {
    if (!trace?.spans) return;
    
    const rect = canvasElement.getBoundingClientRect();
    const mouseX = event.clientX - rect.left;
    const mouseY = event.clientY - rect.top;
    
    // Group spans by service
    const serviceGroups = groupSpansByService(trace.spans);
    const services = Object.keys(serviceGroups);
    
    // Calculate positions for nodes
    const nodePositions = calculateNodePositions(trace.spans, services);
    
    // Find node under mouse
    let foundNodeId: string | null = null;
    
    nodePositions.forEach((pos, spanId) => {
      const span = pos.span;
      
      // Node radius based on span duration
      const minRadius = 5;
      const maxRadius = 15;
      const maxDuration = 200; // ms
      const radius = minRadius + Math.min(span.duration / maxDuration, 1) * (maxRadius - minRadius);
      
      // Check if mouse is over node
      const dx = mouseX - pos.x;
      const dy = mouseY - pos.y;
      const distance = Math.sqrt(dx * dx + dy * dy);
      
      if (distance <= radius) {
        foundNodeId = spanId;
      }
    });
    
    // Update highlighted node
    highlightedSpanId = foundNodeId;
  }
  
  // Handle mouse click
  function handleClick() {
    if (highlightedSpanId) {
      onCauseIdentified(highlightedSpanId);
    }
  }
  
  // Convert hex color to RGB
  function hexToRgb(hex: string): string {
    // Remove # if present
    hex = hex.replace('#', '');
    
    // Parse
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);
    
    return `${r}, ${g}, ${b}`;
  }
</script>

<div class="causal-inference">
  <div class="visualization-container">
    <canvas 
      bind:this={canvasElement}
      width={width}
      height={height}
      on:mousemove={handleMouseMove}
      on:click={handleClick}
    ></canvas>
  </div>
  
  <div class="controls">
    <div class="legend">
      <div class="legend-item">
        <div class="legend-color" style="background-color: #00F5D4;"></div>
        <span>Success</span>
      </div>
      
      <div class="legend-item">
        <div class="legend-color" style="background-color: #FFD600;"></div>
        <span>Warning</span>
      </div>
      
      <div class="legend-item">
        <div class="legend-color" style="background-color: #FF3864;"></div>
        <span>Error</span>
      </div>
      
      <div class="legend-item">
        <div class="legend-line"></div>
        <span>Parent-Child</span>
      </div>
      
      <div class="legend-item">
        <div class="legend-line causal"></div>
        <span>Causal Relationship</span>
      </div>
    </div>
    
    <div class="info-box">
      {#if highlightedSpanId}
        <p>
          Click on a node to inspect potential causal relationships
        </p>
      {:else}
        <p>
          Hover over nodes to explore the causal graph
        </p>
      {/if}
    </div>
  </div>
</div>

<style>
  .causal-inference {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-radius: 8px;
    overflow: hidden;
  }
  
  .visualization-container {
    flex: 1;
    overflow: hidden;
  }
  
  canvas {
    display: block;
    cursor: pointer;
  }
  
  .controls {
    padding: 0.75rem 1rem;
    border-top: 1px solid var(--border-light);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .legend {
    display: flex;
    gap: 1.5rem;
  }
  
  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.8);
  }
  
  .legend-color {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }
  
  .legend-line {
    width: 20px;
    height: 2px;
    background-color: rgba(255, 255, 255, 0.2);
  }
  
  .legend-line.causal {
    background-color: rgba(138, 43, 226, 0.4);
  }
  
  .info-box {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.7);
  }
  
  .info-box p {
    margin: 0;
  }
</style>