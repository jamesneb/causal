<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { Span, Trace, TimelineEvent } from './types';
  import DebugControls from './DebugControls.svelte';
  
  // Props
  export let trace: Trace | null = null;
  export let events: TimelineEvent[] = [];
  export let width: number = 1000;
  export let height: number = 200;
  export let startTime: number = 0;
  export let endTime: number = 0;
  export let onSpanClick = (spanId: string) => {};
  export let onEventClick = (eventId: string) => {};
  export let showDebugControls: boolean = true;
  
  // Internal state
  let container: HTMLElement;
  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let isDragging = false;
  let isPanning = false;
  let dragStart = { x: 0, y: 0 };
  let viewStartTime = startTime;
  let viewEndTime = endTime;
  let timeRange = endTime - startTime;
  let visibleTimeRange = timeRange;
  let mouseX = 0;
  let mouseY = 0;
  let hoverSpanId: string | null = null;
  let hoverEventId: string | null = null;
  let selectedSpanId: string | null = null;
  let selectedEventId: string | null = null;
  let animationFrame: number;
  
  // Constants
  const ROW_HEIGHT = 30;
  const EVENT_RADIUS = 6;
  const SPAN_HEIGHT = 20;
  const MIN_SPAN_WIDTH = 2;
  const VERTICAL_PADDING = 20;
  const TIME_AXIS_HEIGHT = 40;
  const LEFT_AXIS_WIDTH = 180;
  const SERVICE_COLOR_MAP: Record<string, string> = {
    'api-gateway': '#8A2BE2', // Quantum Purple
    'auth-service': '#00F5D4', // Electric Teal
    'user-service': '#FFD600', // Signal Yellow
    'product-service': '#FF3864', // Plasma Pink
    'order-service': '#42A5F5', // Info Blue
    'payment-service': '#00E676', // Success Green
    'database': '#AE7AFF', // Light Purple
    'cache': '#FFAB40', // Orange
    'queue': '#BA68C8', // Light Purple
    'external': '#78909C', // Gray
  };
  
  $: {
    // Auto-calculate time range if not provided
    if (!startTime && !endTime && trace?.spans) {
      const allStartTimes = trace.spans.map(span => span.startTime);
      const allEndTimes = trace.spans.map(span => span.endTime);
      startTime = Math.min(...allStartTimes);
      endTime = Math.max(...allEndTimes);
      timeRange = endTime - startTime;
      visibleTimeRange = timeRange;
      viewStartTime = startTime;
      viewEndTime = endTime;
    }
  }
  
  onMount(() => {
    ctx = canvas.getContext('2d')!;
    window.addEventListener('resize', handleResize);
    handleResize();
    
    // Start animation loop
    animationFrame = requestAnimationFrame(draw);
  });
  
  onDestroy(() => {
    window.removeEventListener('resize', handleResize);
    cancelAnimationFrame(animationFrame);
  });
  
  function handleResize() {
    const { width: containerWidth, height: containerHeight } = container.getBoundingClientRect();
    canvas.width = containerWidth;
    canvas.height = containerHeight;
    width = containerWidth;
    height = containerHeight;
    draw();
  }
  
  function draw() {
    if (!ctx) return;
    
    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    // Calculate dimensions
    const contentWidth = width - LEFT_AXIS_WIDTH;
    const contentHeight = height - TIME_AXIS_HEIGHT;
    
    // Draw background grid
    drawGrid(LEFT_AXIS_WIDTH, 0, contentWidth, contentHeight);
    
    // Draw time axis
    drawTimeAxis(LEFT_AXIS_WIDTH, contentHeight, contentWidth, TIME_AXIS_HEIGHT);
    
    // Draw service names on left
    drawServiceAxis(0, 0, LEFT_AXIS_WIDTH, contentHeight);
    
    // Draw spans if we have trace data
    if (trace?.spans) {
      drawSpans(LEFT_AXIS_WIDTH, 0, contentWidth, contentHeight);
    }
    
    // Draw events
    drawEvents(LEFT_AXIS_WIDTH, 0, contentWidth, contentHeight);
    
    // Continue animation loop
    animationFrame = requestAnimationFrame(draw);
  }
  
  function drawGrid(x: number, y: number, width: number, height: number) {
    // Background
    ctx.fillStyle = 'rgba(26, 22, 48, 0.5)';
    ctx.fillRect(x, y, width, height);
    
    // Draw horizontal grid lines (one per service)
    if (trace?.spans) {
      const services = [...new Set(trace.spans.map(span => span.serviceName))];
      
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
      ctx.lineWidth = 1;
      
      services.forEach((service, i) => {
        const yPos = y + VERTICAL_PADDING + i * ROW_HEIGHT + ROW_HEIGHT / 2;
        
        ctx.beginPath();
        ctx.moveTo(x, yPos);
        ctx.lineTo(x + width, yPos);
        ctx.stroke();
      });
    }
    
    // Draw time intervals
    const timeIntervals = getTimeIntervals(viewStartTime, viewEndTime);
    
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
    ctx.lineWidth = 1;
    
    timeIntervals.forEach(timeMs => {
      const xPos = timeToX(timeMs, x, width);
      
      ctx.beginPath();
      ctx.moveTo(xPos, y);
      ctx.lineTo(xPos, y + height);
      ctx.stroke();
    });
  }
  
  function drawTimeAxis(x: number, y: number, width: number, height: number) {
    // Background
    ctx.fillStyle = 'rgba(18, 16, 35, 1)';
    ctx.fillRect(x, y, width, height);
    
    // Draw time labels
    const timeIntervals = getTimeIntervals(viewStartTime, viewEndTime);
    
    ctx.fillStyle = 'rgba(255, 255, 255, 0.7)';
    ctx.font = '11px Arial';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    
    timeIntervals.forEach(timeMs => {
      const xPos = timeToX(timeMs, x, width);
      const timeLabel = formatTime(timeMs - startTime);
      
      ctx.fillText(timeLabel, xPos, y + 12);
    });
    
    // Draw "now" label if visible
    if (viewStartTime <= Date.now() && Date.now() <= viewEndTime) {
      const nowX = timeToX(Date.now(), x, width);
      
      ctx.strokeStyle = 'rgba(255, 56, 100, 0.8)';
      ctx.lineWidth = 2;
      
      ctx.beginPath();
      ctx.moveTo(nowX, y - height);
      ctx.lineTo(nowX, y + height);
      ctx.stroke();
      
      ctx.fillStyle = 'rgba(255, 56, 100, 1)';
      ctx.fillText('NOW', nowX, y + 12);
    }
  }
  
  function drawServiceAxis(x: number, y: number, width: number, height: number) {
    // Background
    ctx.fillStyle = 'rgba(18, 16, 35, 1)';
    ctx.fillRect(x, y, width, height);
    
    // Draw service names
    if (trace?.spans) {
      const services = [...new Set(trace.spans.map(span => span.serviceName))];
      
      ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
      ctx.font = '12px Arial';
      ctx.textAlign = 'left';
      ctx.textBaseline = 'middle';
      
      services.forEach((service, i) => {
        const yPos = y + VERTICAL_PADDING + i * ROW_HEIGHT + ROW_HEIGHT / 2;
        const serviceColor = SERVICE_COLOR_MAP[service] || '#FFFFFF';
        
        // Service color indicator
        ctx.fillStyle = serviceColor;
        ctx.beginPath();
        ctx.rect(x + 10, yPos - 8, 16, 16);
        ctx.fill();
        
        // Service name
        ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
        ctx.fillText(service, x + 36, yPos);
      });
    }
  }
  
  function drawSpans(x: number, y: number, width: number, height: number) {
    if (!trace?.spans) return;
    
    const services = [...new Set(trace.spans.map(span => span.serviceName))];
    const serviceIndex: Record<string, number> = {};
    
    services.forEach((service, index) => {
      serviceIndex[service] = index;
    });
    
    // Draw connection lines between parent-child spans
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.2)';
    ctx.lineWidth = 1;
    
    trace.spans.forEach(span => {
      if (span.parentId) {
        const parentSpan = trace.spans.find(s => s.id === span.parentId);
        if (parentSpan) {
          const parentServiceIndex = serviceIndex[parentSpan.serviceName];
          const childServiceIndex = serviceIndex[span.serviceName];
          
          if (parentServiceIndex !== childServiceIndex) {
            const parentY = y + VERTICAL_PADDING + parentServiceIndex * ROW_HEIGHT + ROW_HEIGHT / 2;
            const childY = y + VERTICAL_PADDING + childServiceIndex * ROW_HEIGHT + ROW_HEIGHT / 2;
            
            const parentEndX = timeToX(parentSpan.endTime, x, width);
            const childStartX = timeToX(span.startTime, x, width);
            
            ctx.beginPath();
            ctx.moveTo(parentEndX, parentY);
            ctx.lineTo(childStartX, childY);
            ctx.stroke();
          }
        }
      }
    });
    
    // Draw spans
    trace.spans.forEach(span => {
      const serviceIdx = serviceIndex[span.serviceName];
      const yPos = y + VERTICAL_PADDING + serviceIdx * ROW_HEIGHT + (ROW_HEIGHT - SPAN_HEIGHT) / 2;
      const spanStartX = timeToX(span.startTime, x, width);
      const spanEndX = timeToX(span.endTime, x, width);
      let spanWidth = Math.max(spanEndX - spanStartX, MIN_SPAN_WIDTH);
      
      // Determine span color based on service and status
      let spanColor = SERVICE_COLOR_MAP[span.serviceName] || '#FFFFFF';
      
      if (span.status === 'error') {
        spanColor = '#FF3864'; // Error color
      }
      
      // Highlight on hover or selection
      const isSelected = span.id === selectedSpanId;
      const isHovered = span.id === hoverSpanId;
      
      if (isSelected || isHovered) {
        // Draw highlight glow
        ctx.fillStyle = isSelected ? 
          `rgba(${hexToRgb(spanColor)}, 0.3)` : 
          `rgba(${hexToRgb(spanColor)}, 0.2)`;
          
        ctx.beginPath();
        ctx.roundRect(
          spanStartX - 2, 
          yPos - 2, 
          spanWidth + 4, 
          SPAN_HEIGHT + 4,
          4
        );
        ctx.fill();
      }
      
      // Draw span bar
      ctx.fillStyle = spanColor;
      ctx.beginPath();
      ctx.roundRect(spanStartX, yPos, spanWidth, SPAN_HEIGHT, 3);
      ctx.fill();
      
      if (spanWidth > 30) {
        // Draw span name if there's enough space
        ctx.fillStyle = getContrastTextColor(spanColor);
        ctx.font = '11px Arial';
        ctx.textAlign = 'left';
        ctx.textBaseline = 'middle';
        
        // Truncate text if needed
        const text = span.name;
        const maxTextWidth = spanWidth - 10;
        let displayText = text;
        
        let textMetrics = ctx.measureText(displayText);
        if (textMetrics.width > maxTextWidth) {
          // Truncate and add ellipsis
          while (textMetrics.width > maxTextWidth && displayText.length > 3) {
            displayText = displayText.substring(0, displayText.length - 1);
            textMetrics = ctx.measureText(displayText + '...');
          }
          displayText += '...';
        }
        
        ctx.fillText(
          displayText, 
          spanStartX + 5, 
          yPos + SPAN_HEIGHT / 2
        );
      }
      
      // Draw duration label if hovering/selected and enough width
      if ((isHovered || isSelected) && spanWidth > 40) {
        const duration = (span.endTime - span.startTime).toFixed(2);
        const durationText = `${duration}ms`;
        
        ctx.fillStyle = 'rgba(0, 0, 0, 0.7)';
        ctx.fillRect(
          spanStartX + spanWidth - 54, 
          yPos + SPAN_HEIGHT - 16, 
          54, 
          16
        );
        
        ctx.fillStyle = '#FFFFFF';
        ctx.font = '10px Arial';
        ctx.textAlign = 'center';
        ctx.fillText(
          durationText, 
          spanStartX + spanWidth - 27, 
          yPos + SPAN_HEIGHT - 8
        );
      }
    });
  }
  
  function drawEvents(x: number, y: number, width: number, height: number) {
    if (!events || events.length === 0) return;
    
    if (trace?.spans) {
      const services = [...new Set(trace.spans.map(span => span.serviceName))];
      const serviceIndex: Record<string, number> = {};
      
      services.forEach((service, index) => {
        serviceIndex[service] = index;
      });
      
      events.forEach(event => {
        if (!serviceIndex[event.serviceName]) return;
        
        const serviceIdx = serviceIndex[event.serviceName];
        const yPos = y + VERTICAL_PADDING + serviceIdx * ROW_HEIGHT + ROW_HEIGHT / 2;
        const eventX = timeToX(event.timestamp, x, width);
        
        // Get event color based on type
        let eventColor;
        switch (event.type) {
          case 'breakpoint':
            eventColor = '#FF3864'; // Plasma Pink
            break;
          case 'error':
            eventColor = '#FF3864'; // Error
            break;
          case 'warning':
            eventColor = '#FFD600'; // Warning
            break;
          case 'info':
            eventColor = '#42A5F5'; // Info
            break;
          default:
            eventColor = '#FFFFFF';
        }
        
        // Highlight on hover or selection
        if (event.id === selectedEventId || event.id === hoverEventId) {
          // Draw highlight glow
          ctx.fillStyle = `rgba(${hexToRgb(eventColor)}, 0.3)`;
          ctx.beginPath();
          ctx.arc(eventX, yPos, EVENT_RADIUS + 4, 0, Math.PI * 2);
          ctx.fill();
        }
        
        // Draw event marker
        ctx.fillStyle = eventColor;
        ctx.beginPath();
        
        if (event.type === 'breakpoint') {
          // Diamond shape for breakpoints
          ctx.moveTo(eventX, yPos - EVENT_RADIUS);
          ctx.lineTo(eventX + EVENT_RADIUS, yPos);
          ctx.lineTo(eventX, yPos + EVENT_RADIUS);
          ctx.lineTo(eventX - EVENT_RADIUS, yPos);
        } else {
          // Circle for other events
          ctx.arc(eventX, yPos, EVENT_RADIUS, 0, Math.PI * 2);
        }
        
        ctx.fill();
        
        // Draw icon or letter in the center
        ctx.fillStyle = getContrastTextColor(eventColor);
        ctx.font = 'bold 9px Arial';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        
        let eventSymbol = '';
        switch (event.type) {
          case 'breakpoint':
            eventSymbol = 'B';
            break;
          case 'error':
            eventSymbol = '!';
            break;
          case 'warning':
            eventSymbol = '!';
            break;
          case 'info':
            eventSymbol = 'i';
            break;
          default:
            eventSymbol = 'E';
        }
        
        ctx.fillText(eventSymbol, eventX, yPos);
        
        // Show event details on hover
        if (event.id === hoverEventId || event.id === selectedEventId) {
          const tooltipWidth = 150;
          const tooltipHeight = 70;
          let tooltipX = eventX + 10;
          const tooltipY = yPos - tooltipHeight - 10;
          
          // Adjust if tooltip would go off the right edge
          if (tooltipX + tooltipWidth > x + width) {
            tooltipX = eventX - tooltipWidth - 10;
          }
          
          // Background
          ctx.fillStyle = 'rgba(18, 16, 35, 0.9)';
          ctx.strokeStyle = eventColor;
          ctx.lineWidth = 2;
          
          ctx.beginPath();
          ctx.roundRect(tooltipX, tooltipY, tooltipWidth, tooltipHeight, 6);
          ctx.fill();
          ctx.stroke();
          
          // Title
          ctx.fillStyle = '#FFFFFF';
          ctx.font = 'bold 12px Arial';
          ctx.textAlign = 'left';
          ctx.textBaseline = 'top';
          ctx.fillText(event.name, tooltipX + 10, tooltipY + 10);
          
          // Time
          ctx.fillStyle = 'rgba(255, 255, 255, 0.7)';
          ctx.font = '10px Arial';
          ctx.fillText(
            formatTime(event.timestamp - startTime), 
            tooltipX + 10, 
            tooltipY + 30
          );
          
          // Description
          if (event.description) {
            ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
            ctx.font = '11px Arial';
            ctx.fillText(
              truncateText(event.description, 18), 
              tooltipX + 10, 
              tooltipY + 50
            );
          }
        }
      });
    }
  }
  
  function handleMouseDown(event: MouseEvent) {
    isDragging = true;
    dragStart = { x: event.offsetX, y: event.offsetY };
    
    // Check if clicking on a span or event
    if (hoverSpanId) {
      selectedSpanId = hoverSpanId;
      selectedEventId = null;
      isPanning = false;
      
      // Notify parent component
      onSpanClick(hoverSpanId);
    } else if (hoverEventId) {
      selectedEventId = hoverEventId;
      selectedSpanId = null;
      isPanning = false;
      
      // Notify parent component
      onEventClick(hoverEventId);
    } else {
      isPanning = true;
      selectedSpanId = null;
      selectedEventId = null;
    }
  }
  
  function handleMouseMove(event: MouseEvent) {
    mouseX = event.offsetX;
    mouseY = event.offsetY;
    
    if (isDragging && isPanning) {
      // Pan the timeline
      const deltaX = mouseX - dragStart.x;
      const contentWidth = width - LEFT_AXIS_WIDTH;
      const pixelsPerMs = contentWidth / visibleTimeRange;
      const timeShift = deltaX / pixelsPerMs;
      
      viewStartTime = Math.max(startTime, viewStartTime - timeShift);
      viewEndTime = Math.min(endTime, viewEndTime - timeShift);
      
      // If we hit the edges, adjust both to maintain the same visible time range
      if (viewStartTime === startTime) {
        viewEndTime = viewStartTime + visibleTimeRange;
      } else if (viewEndTime === endTime) {
        viewStartTime = viewEndTime - visibleTimeRange;
      }
      
      dragStart = { x: mouseX, y: mouseY };
    } else {
      // Check for hover over spans
      updateHoverState();
    }
  }
  
  function handleMouseUp() {
    isDragging = false;
    isPanning = false;
  }
  
  function handleWheel(event: WheelEvent) {
    event.preventDefault();
    
    // Calculate zoom factor
    const zoomDirection = event.deltaY < 0 ? -1 : 1;
    const zoomFactor = 1 + (zoomDirection * 0.1);
    
    // Calculate cursor position as a percentage of the content area
    const contentWidth = width - LEFT_AXIS_WIDTH;
    const cursorXPercentage = (mouseX - LEFT_AXIS_WIDTH) / contentWidth;
    
    // Calculate new visible time range (zoom in/out)
    const newVisibleTimeRange = Math.min(
      Math.max(visibleTimeRange * zoomFactor, 100), // Min 100ms visible
      timeRange // Max entire time range
    );
    
    // Calculate new start/end times while keeping cursor position fixed
    const timeAtCursor = viewStartTime + visibleTimeRange * cursorXPercentage;
    const newStartTime = Math.max(
      startTime,
      timeAtCursor - (newVisibleTimeRange * cursorXPercentage)
    );
    const newEndTime = Math.min(
      endTime,
      timeAtCursor + (newVisibleTimeRange * (1 - cursorXPercentage))
    );
    
    // Update state
    visibleTimeRange = newVisibleTimeRange;
    viewStartTime = newStartTime;
    viewEndTime = newEndTime;
    
    // Redraw
    draw();
  }
  
  function updateHoverState() {
    // If not over the content area, clear hover states
    if (mouseX < LEFT_AXIS_WIDTH || mouseY > height - TIME_AXIS_HEIGHT) {
      hoverSpanId = null;
      hoverEventId = null;
      return;
    }
    
    // Check for hover over events (events have priority over spans)
    if (events.length > 0 && trace?.spans) {
      const services = [...new Set(trace.spans.map(span => span.serviceName))];
      const serviceIndex: Record<string, number> = {};
      
      services.forEach((service, index) => {
        serviceIndex[service] = index;
      });
      
      for (const event of events) {
        if (!serviceIndex[event.serviceName]) continue;
        
        const serviceIdx = serviceIndex[event.serviceName];
        const eventY = VERTICAL_PADDING + serviceIdx * ROW_HEIGHT + ROW_HEIGHT / 2;
        const eventX = timeToX(event.timestamp, LEFT_AXIS_WIDTH, width - LEFT_AXIS_WIDTH);
        
        const dx = mouseX - eventX;
        const dy = mouseY - eventY;
        const distance = Math.sqrt(dx * dx + dy * dy);
        
        if (distance <= EVENT_RADIUS + 2) {
          hoverEventId = event.id;
          hoverSpanId = null;
          return;
        }
      }
      
      hoverEventId = null;
    }
    
    // Check for hover over spans
    if (trace?.spans) {
      const services = [...new Set(trace.spans.map(span => span.serviceName))];
      const serviceIndex: Record<string, number> = {};
      
      services.forEach((service, index) => {
        serviceIndex[service] = index;
      });
      
      for (const span of trace.spans) {
        const serviceIdx = serviceIndex[span.serviceName];
        const spanY = VERTICAL_PADDING + serviceIdx * ROW_HEIGHT + (ROW_HEIGHT - SPAN_HEIGHT) / 2;
        const spanStartX = timeToX(span.startTime, LEFT_AXIS_WIDTH, width - LEFT_AXIS_WIDTH);
        const spanEndX = timeToX(span.endTime, LEFT_AXIS_WIDTH, width - LEFT_AXIS_WIDTH);
        const spanWidth = Math.max(spanEndX - spanStartX, MIN_SPAN_WIDTH);
        
        if (
          mouseX >= spanStartX &&
          mouseX <= spanStartX + spanWidth &&
          mouseY >= spanY &&
          mouseY <= spanY + SPAN_HEIGHT
        ) {
          hoverSpanId = span.id;
          return;
        }
      }
      
      hoverSpanId = null;
    }
  }
  
  // Utility function to convert time to X coordinate
  function timeToX(time: number, startX: number, contentWidth: number): number {
    const timePercentage = (time - viewStartTime) / (viewEndTime - viewStartTime);
    return startX + timePercentage * contentWidth;
  }
  
  // Generate appropriate time intervals based on visible time range
  function getTimeIntervals(start: number, end: number): number[] {
    const range = end - start;
    
    let interval: number;
    if (range <= 100) interval = 10; // 10ms
    else if (range <= 500) interval = 50; // 50ms
    else if (range <= 1000) interval = 100; // 100ms
    else if (range <= 5000) interval = 500; // 500ms
    else if (range <= 10000) interval = 1000; // 1s
    else if (range <= 60000) interval = 5000; // 5s
    else if (range <= 300000) interval = 30000; // 30s
    else interval = 60000; // 1min
    
    const result: number[] = [];
    let current = Math.ceil(start / interval) * interval;
    
    while (current <= end) {
      result.push(current);
      current += interval;
    }
    
    return result;
  }
  
  // Format time in appropriate units
  function formatTime(ms: number): string {
    if (ms < 1000) return `${ms.toFixed(0)}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
    const minutes = Math.floor(ms / 60000);
    const seconds = ((ms % 60000) / 1000).toFixed(0);
    return `${minutes}m ${seconds}s`;
  }
  
  // Determine text color based on background for contrast
  function getContrastTextColor(bgColor: string): string {
    const rgb = hexToRgb(bgColor);
    if (!rgb) return '#000000';
    
    const [r, g, b] = rgb.split(',').map(Number);
    
    // Calculate luminance - simplified formula
    const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
    
    return luminance > 0.5 ? '#000000' : '#FFFFFF';
  }
  
  // Convert hex color to rgb string
  function hexToRgb(hex: string): string {
    // Default fallback color if conversion fails
    if (!hex || hex === 'undefined') return '255,255,255';
    
    // Remove # if present
    hex = hex.replace('#', '');
    
    // Convert shorthand to full form (e.g., #FFF to #FFFFFF)
    if (hex.length === 3) {
      hex = hex[0] + hex[0] + hex[1] + hex[1] + hex[2] + hex[2];
    }
    
    // Parse
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);
    
    return `${r},${g},${b}`;
  }
  
  // Truncate text with ellipsis
  function truncateText(text: string, maxLength: number): string {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + '...';
  }
</script>

<div 
  class="timeline-viewer"
  bind:this={container}
  on:mousedown={handleMouseDown}
  on:mousemove={handleMouseMove}
  on:mouseup={handleMouseUp}
  on:mouseleave={handleMouseUp}
  on:wheel={handleWheel}
>
  <canvas
    bind:this={canvas}
    width={width}
    height={height}
  ></canvas>
  
  <div class="timeline-controls">
    <button class="btn-sm" title="Zoom In" on:click={() => handleWheel({ preventDefault: () => {}, deltaY: -100 } as WheelEvent)}>
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
        <line x1="11" y1="8" x2="11" y2="14" />
        <line x1="8" y1="11" x2="14" y2="11" />
      </svg>
    </button>
    
    <button class="btn-sm" title="Zoom Out" on:click={() => handleWheel({ preventDefault: () => {}, deltaY: 100 } as WheelEvent)}>
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
        <line x1="8" y1="11" x2="14" y2="11" />
      </svg>
    </button>
    
    <button class="btn-sm" title="Reset View" on:click={() => {
      viewStartTime = startTime;
      viewEndTime = endTime;
      visibleTimeRange = timeRange;
    }}>
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 3H6a3 3 0 00-3 3v12a3 3 0 003 3h12a3 3 0 003-3V6a3 3 0 00-3-3z"></path>
        <path d="M15 9h-6v6h6V9z"></path>
      </svg>
    </button>
  </div>
  
  {#if showDebugControls && trace}
    <div class="debug-controls-wrapper">
      <DebugControls 
        currentTimestamp={Date.now()} 
        {startTime}
        {endTime}
        isPaused={true}
        breakpoints={[]}
        onResume={() => {}}
        onPause={() => {}}
        onStepOver={() => {}}
        onStepInto={() => {}}
        onStepOut={() => {}}
        onSeek={(time) => {}}
      />
    </div>
  {/if}
</div>

<style>
  .timeline-viewer {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
    border-radius: 8px;
    cursor: default;
  }
  
  canvas {
    display: block;
    user-select: none;
  }
  
  .timeline-controls {
    position: absolute;
    top: 10px;
    right: 10px;
    display: flex;
    gap: 4px;
  }
  
  .btn-sm {
    background-color: rgba(26, 22, 48, 0.8);
    border: 1px solid var(--border-light);
    border-radius: 4px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-light);
    cursor: pointer;
  }
  
  .btn-sm:hover {
    background-color: var(--bg-tertiary);
    border-color: var(--border-active);
  }
  
  .debug-controls-wrapper {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 10;
  }
</style>