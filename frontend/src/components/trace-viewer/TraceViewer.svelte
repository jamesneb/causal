<script lang="ts">
  import { TimelineViewer } from '../timeline';
  import type { Trace, Span, TimelineEvent } from '../timeline/types';
  import CausalInference from './CausalInference.svelte';
  import RootCauseAnalysis from './RootCauseAnalysis.svelte';
  import { SystemSynthesis } from '../system-synthesis';
  
  // Props
  export let trace: Trace | null = null;
  export let events: TimelineEvent[] = [];
  export let showDetails: boolean = true;
  export let expandedSpanId: string | null = null;
  
  // Internal state
  let selectedSpanId: string | null = null;
  let selectedEventId: string | null = null;
  let timelineHeight = 200;
  let activeTab: 'spans' | 'events' | 'causal' | 'rootcause' = 'spans';
  let showSystemSynthesis: boolean = true;
  
  // Computed properties
  $: selectedSpan = trace?.spans?.find(span => span.id === selectedSpanId);
  $: selectedEvent = events.find(event => event.id === selectedEventId);
  $: traceServiceNames = trace?.spans ? [...new Set(trace.spans.map(span => span.serviceName))] : [];
  $: spansByService = trace?.spans ? groupSpansByService(trace.spans) : {};
  // Detect errors in the trace to display system synthesis panel
  $: hasError = trace?.spans?.some(span => span.status === 'error') || false;
  
  // Functions
  function handleSpanClick(spanId: string) {
    selectedSpanId = spanId;
    selectedEventId = null;
  }
  
  function handleEventClick(eventId: string) {
    selectedEventId = eventId;
    selectedSpanId = null;
  }
  
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
  
  function toggleSpanExpand(spanId: string) {
    if (expandedSpanId === spanId) {
      expandedSpanId = null;
    } else {
      expandedSpanId = spanId;
    }
  }
  
  function toggleSystemSynthesis() {
    showSystemSynthesis = !showSystemSynthesis;
  }
</script>

<div class="trace-viewer">
  {#if trace}
    <div class="trace-header">
      <div class="trace-title">
        <h2>{trace.name}</h2>
        <div class="trace-subtitle">
          <span class="trace-id">ID: {trace.id}</span>
          <span class="trace-duration">Duration: {trace.duration.toFixed(2)}ms</span>
          <span class="trace-status" class:error={trace.status === 'error'} class:warning={trace.status === 'warning'}>
            Status: {trace.status}
          </span>
        </div>
      </div>
      
      <div class="trace-actions">
        <button class="btn-secondary" title="Add Breakpoint">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="16" />
            <line x1="8" y1="12" x2="16" y2="12" />
          </svg>
          <span>Add Breakpoint</span>
        </button>
        
        <button class="btn-secondary" title="Export">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
          <span>Export</span>
        </button>
        
        {#if hasError}
          <button 
            class="btn-secondary {showSystemSynthesis ? 'active' : ''}" 
            title="Toggle System Synthesis"
            on:click={toggleSystemSynthesis}
          >
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2v2m0 16v2M4 12H2m20 0h-2m-9 7l-3-3m3-11l-3 3m12 0l-3-3m0 14l3-3" />
              <circle cx="12" cy="12" r="4" />
            </svg>
            <span>System Synthesis</span>
          </button>
        {/if}
      </div>
    </div>
    
    {#if hasError && showSystemSynthesis}
      <SystemSynthesis rootCauseData={null} showActions={true} />
    {/if}
    
    <div class="trace-timeline-container">
      <TimelineViewer 
        {trace}
        {events}
        height={timelineHeight}
        startTime={trace.startTime}
        endTime={trace.endTime}
        onSpanClick={handleSpanClick}
        onEventClick={handleEventClick}
        showDebugControls={true}
      />
    </div>
    
    {#if showDetails}
      <div class="trace-details">
        <div class="detail-tabs">
          <button 
            class={`tab-button ${activeTab === 'spans' ? 'active' : ''}`}
            on:click={() => activeTab = 'spans'}
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="5" y1="12" x2="19" y2="12"></line>
              <line x1="5" y1="5" x2="19" y2="5"></line>
              <line x1="5" y1="19" x2="19" y2="19"></line>
            </svg>
            <span>Spans</span>
          </button>
          
          <button 
            class={`tab-button ${activeTab === 'events' ? 'active' : ''}`}
            on:click={() => activeTab = 'events'}
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12" y2="16"></line>
            </svg>
            <span>Events</span>
          </button>
          
          <button 
            class={`tab-button ${activeTab === 'causal' ? 'active' : ''}`}
            on:click={() => activeTab = 'causal'}
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="6" cy="7" r="3"></circle>
              <circle cx="18" cy="7" r="3"></circle>
              <circle cx="12" cy="17" r="3"></circle>
              <line x1="6" y1="10" x2="12" y2="14"></line>
              <line x1="18" y1="10" x2="12" y2="14"></line>
            </svg>
            <span>Causal Graph</span>
          </button>
          
          <button 
            class={`tab-button ${activeTab === 'rootcause' ? 'active' : ''}`}
            on:click={() => activeTab = 'rootcause'}
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
              <polyline points="22 4 12 14.01 9 11.01"></polyline>
            </svg>
            <span>Root Cause Analysis</span>
          </button>
        </div>
        
        <div class="detail-content">
          {#if activeTab === 'causal'}
            <div class="causal-inference-container">
              <CausalInference 
                trace={trace}
                onCauseIdentified={(spanId) => {
                  selectedSpanId = spanId;
                  activeTab = 'spans';
                }}
              />
            </div>
          {:else if activeTab === 'rootcause'}
            <div class="root-cause-container">
              <RootCauseAnalysis 
                {trace}
                selectedSpan={trace?.spans?.find(span => span.status === 'error')}
                onSpanSelect={handleSpanClick}
              />
            </div>
          {:else if activeTab === 'spans' && selectedSpanId && selectedSpan}
            <div class="span-details">
              <div class="detail-header">
                <h3>Span: {selectedSpan.name}</h3>
                <div class="detail-service">{selectedSpan.serviceName}</div>
              </div>
              
              <div class="detail-section">
                <div class="detail-row">
                  <div class="detail-key">ID</div>
                  <div class="detail-value monospace">{selectedSpan.id}</div>
                </div>
                
                {#if selectedSpan.parentId}
                  <div class="detail-row">
                    <div class="detail-key">Parent ID</div>
                    <div class="detail-value monospace">
                      <a href="javascript:void(0)" on:click={() => handleSpanClick(selectedSpan.parentId || '')}>
                        {selectedSpan.parentId}
                      </a>
                    </div>
                  </div>
                {/if}
                
                <div class="detail-row">
                  <div class="detail-key">Duration</div>
                  <div class="detail-value">{selectedSpan.duration.toFixed(2)}ms</div>
                </div>
                
                <div class="detail-row">
                  <div class="detail-key">Start Time</div>
                  <div class="detail-value">
                    {new Date(selectedSpan.startTime).toISOString()}
                  </div>
                </div>
                
                <div class="detail-row">
                  <div class="detail-key">End Time</div>
                  <div class="detail-value">
                    {new Date(selectedSpan.endTime).toISOString()}
                  </div>
                </div>
                
                <div class="detail-row">
                  <div class="detail-key">Status</div>
                  <div class="detail-value status-badge" class:error={selectedSpan.status === 'error'} class:warning={selectedSpan.status === 'warning'}>
                    {selectedSpan.status}
                  </div>
                </div>
              </div>
              
              {#if selectedSpan.attributes && Object.keys(selectedSpan.attributes).length > 0}
                <div class="detail-section">
                  <h4>Attributes</h4>
                  
                  <div class="attribute-list">
                    {#each Object.entries(selectedSpan.attributes) as [key, value]}
                      <div class="detail-row">
                        <div class="detail-key">{key}</div>
                        <div class="detail-value">
                          {typeof value === 'object' ? JSON.stringify(value) : value}
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
              
              <div class="detail-actions">
                <button class="btn-primary btn-sm">Add Breakpoint</button>
                <button class="btn-secondary btn-sm">Query Related Spans</button>
                <button class="btn-secondary btn-sm">View Logs</button>
              </div>
            </div>
            
          {:else if activeTab === 'events' && selectedEventId && selectedEvent}
            <div class="event-details">
              <div class="detail-header">
                <h3>{selectedEvent.name}</h3>
                <div class="detail-service">{selectedEvent.serviceName}</div>
              </div>
              
              <div class="detail-section">
                <div class="detail-row">
                  <div class="detail-key">ID</div>
                  <div class="detail-value monospace">{selectedEvent.id}</div>
                </div>
                
                <div class="detail-row">
                  <div class="detail-key">Timestamp</div>
                  <div class="detail-value">
                    {new Date(selectedEvent.timestamp).toISOString()}
                  </div>
                </div>
                
                <div class="detail-row">
                  <div class="detail-key">Type</div>
                  <div class="detail-value status-badge" 
                       class:error={selectedEvent.type === 'error'} 
                       class:warning={selectedEvent.type === 'warning'}
                       class:info={selectedEvent.type === 'info'}
                       class:breakpoint={selectedEvent.type === 'breakpoint'}>
                    {selectedEvent.type}
                  </div>
                </div>
                
                {#if selectedEvent.description}
                  <div class="detail-row">
                    <div class="detail-key">Description</div>
                    <div class="detail-value">{selectedEvent.description}</div>
                  </div>
                {/if}
              </div>
              
              {#if selectedEvent.attributes && Object.keys(selectedEvent.attributes).length > 0}
                <div class="detail-section">
                  <h4>Attributes</h4>
                  
                  <div class="attribute-list">
                    {#each Object.entries(selectedEvent.attributes) as [key, value]}
                      <div class="detail-row">
                        <div class="detail-key">{key}</div>
                        <div class="detail-value">
                          {typeof value === 'object' ? JSON.stringify(value) : value}
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
              
              <div class="detail-actions">
                <button class="btn-primary btn-sm">Edit Breakpoint</button>
                <button class="btn-secondary btn-sm">Delete</button>
              </div>
            </div>
            
          {:else if expandedSpanId}
            <!-- Expanded span view -->
            <div class="expanded-span-details">
              <h3>Expanded View</h3>
              <!-- Expanded view content -->
            </div>
            
          {:else}
            <div class="span-list">
              <div class="service-list">
                {#each traceServiceNames as serviceName}
                  <div class="service-group">
                    <div class="service-header">
                      <h3>{serviceName}</h3>
                      <div class="service-metrics">
                        <span>{spansByService[serviceName]?.length || 0} spans</span>
                      </div>
                    </div>
                    
                    {#if spansByService[serviceName]}
                      <div class="spans">
                        {#each spansByService[serviceName] as span}
                          <div 
                            class="span-item" 
                            class:selected={span.id === selectedSpanId}
                            class:warning={span.status === 'warning'}
                            class:error={span.status === 'error'}
                            on:click={() => handleSpanClick(span.id)}
                          >
                            <div class="span-info">
                              <div class="span-name">{span.name}</div>
                              <div class="span-duration">{span.duration.toFixed(2)}ms</div>
                            </div>
                            
                            <button 
                              class="btn-icon expand-button"
                              title="Expand"
                              on:click|stopPropagation={() => toggleSpanExpand(span.id)}
                            >
                              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                                <polyline points="6 9 12 15 18 9"></polyline>
                              </svg>
                            </button>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  {:else}
    <div class="trace-empty-state">
      <p>No trace selected. Please select a trace to view details.</p>
    </div>
  {/if}
</div>

<style>
  .trace-viewer {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-radius: 8px;
    overflow: hidden;
  }
  
  .trace-header {
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-light);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .trace-title h2 {
    font-size: 1.2rem;
    font-weight: 600;
    margin: 0 0 0.5rem 0;
  }
  
  .trace-subtitle {
    display: flex;
    gap: 1rem;
    font-size: 0.85rem;
  }
  
  .trace-subtitle span {
    color: rgba(255, 255, 255, 0.7);
  }
  
  .trace-id {
    font-family: monospace;
  }
  
  .trace-status.error {
    color: var(--error);
  }
  
  .trace-status.warning {
    color: var(--warning);
  }
  
  .trace-actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .btn-secondary {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: none;
    border: 1px solid var(--border-medium);
    border-radius: 4px;
    color: var(--text-light);
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .btn-secondary:hover {
    background-color: var(--bg-tertiary);
    border-color: var(--primary);
  }
  
  .btn-secondary.active {
    background-color: var(--bg-hover);
    border-color: var(--primary);
    color: var(--primary);
  }
  
  .trace-timeline-container {
    height: 200px;
    border-bottom: 1px solid var(--border-light);
  }
  
  .trace-details {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  
  .detail-tabs {
    display: flex;
    padding: 0 1rem;
    border-bottom: 1px solid var(--border-light);
  }
  
  .tab-button {
    padding: 0.75rem 1rem;
    background: none;
    border: none;
    color: var(--text-light);
    font-size: 0.9rem;
    cursor: pointer;
    position: relative;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    border-radius: 0;
    overflow: hidden;
  }
  
  .tab-button span {
    position: relative;
    z-index: 2;
  }
  
  .tab-button svg {
    position: relative;
    z-index: 2;
    transition: transform 0.2s ease;
  }
  
  .tab-button:hover svg {
    transform: scale(1.1);
  }
  
  .tab-button::before {
    content: '';
    position: absolute;
    bottom: 0;
    left: 0;
    width: 100%;
    height: 0;
    background-color: rgba(0, 245, 212, 0.1);
    transition: height 0.2s ease;
  }
  
  .tab-button:hover::before {
    height: 100%;
  }
  
  .tab-button.active {
    color: var(--primary);
    font-weight: 500;
  }
  
  .tab-button.active svg {
    filter: drop-shadow(0 0 3px rgba(86, 97, 242, 0.5));
  }
  
  .tab-button.active::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 0;
    width: 100%;
    height: 3px;
    background: linear-gradient(90deg, transparent, var(--primary), transparent);
    z-index: 1;
  }
  
  .detail-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }
  
  .span-details, .event-details, .expanded-span-details {
    max-width: 800px;
  }
  
  .detail-header {
    margin-bottom: 1.25rem;
    display: flex;
    align-items: baseline;
    gap: 1rem;
  }
  
  .detail-header h3 {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0;
  }
  
  .detail-service {
    font-size: 0.8rem;
    text-transform: uppercase;
    padding: 0.2rem 0.5rem;
    background-color: var(--bg-tertiary);
    border-radius: 4px;
    color: rgba(255, 255, 255, 0.8);
  }
  
  .detail-section {
    margin-bottom: 1.5rem;
    padding: 1rem;
    background-color: var(--bg-tertiary);
    border-radius: 6px;
  }
  
  .detail-section h4 {
    font-size: 0.95rem;
    font-weight: 600;
    margin: 0 0 0.75rem 0;
  }
  
  .detail-row {
    display: flex;
    margin-bottom: 0.5rem;
    font-size: 0.9rem;
    line-height: 1.5;
  }
  
  .detail-key {
    flex: 0 0 120px;
    color: rgba(255, 255, 255, 0.7);
  }
  
  .detail-value {
    flex: 1;
  }
  
  .detail-value a {
    color: var(--electric-teal);
    text-decoration: none;
  }
  
  .detail-value a:hover {
    text-decoration: underline;
  }
  
  .monospace {
    font-family: monospace;
    font-size: 0.85rem;
  }
  
  .status-badge {
    display: inline-block;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    text-transform: uppercase;
    font-size: 0.8rem;
    font-weight: 500;
  }
  
  .status-badge.error {
    background-color: rgba(255, 56, 100, 0.2);
    color: var(--error);
  }
  
  .status-badge.warning {
    background-color: rgba(255, 214, 0, 0.2);
    color: var(--warning);
  }
  
  .status-badge.info {
    background-color: rgba(66, 165, 245, 0.2);
    color: var(--info);
  }
  
  .status-badge.breakpoint {
    background-color: rgba(242, 128, 86, 0.2);
    color: var(--accent);
  }
  
  .attribute-list {
    max-height: 200px;
    overflow-y: auto;
  }
  
  .detail-actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .btn-primary {
    background-color: var(--primary);
    color: white;
    border: none;
    border-radius: 4px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .btn-primary:hover {
    background-color: var(--primary-light);
  }
  
  .btn-sm {
    padding: 0.4rem 0.7rem;
    font-size: 0.85rem;
  }
  
  .service-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .service-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--border-light);
  }
  
  .service-header h3 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0;
  }
  
  .service-metrics {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.6);
  }
  
  .span-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.6rem 0.75rem;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.2s ease;
    margin-bottom: 0.25rem;
  }
  
  .span-item:hover {
    background-color: var(--bg-tertiary);
  }
  
  .span-item.selected {
    background-color: var(--bg-tertiary);
    border-left: 3px solid var(--electric-teal);
  }
  
  .span-item.warning {
    border-left: 3px solid var(--warning);
  }
  
  .span-item.error {
    border-left: 3px solid var(--error);
  }
  
  .span-info {
    display: flex;
    gap: 1rem;
    align-items: center;
  }
  
  .span-name {
    font-size: 0.9rem;
  }
  
  .span-duration {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.6);
  }
  
  .btn-icon {
    background: none;
    border: none;
    width: 24px;
    height: 24px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-light);
    cursor: pointer;
    transition: background-color 0.2s ease;
    padding: 0;
  }
  
  .btn-icon:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }
  
  .trace-empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(255, 255, 255, 0.5);
    font-size: 0.95rem;
  }
  
  .causal-inference-container {
    flex: 1;
    height: 100%;
    min-height: 400px;
    display: flex;
    flex-direction: column;
  }
  
  .root-cause-container {
    flex: 1;
    height: 100%;
    min-height: 400px;
    padding: 1rem 0;
  }
  
  .causal-tabs {
    display: flex;
    border-bottom: 1px solid var(--border-light);
    margin-bottom: 1rem;
  }
  
  .causal-tab {
    padding: 0.5rem 1rem;
    background: none;
    border: none;
    color: var(--text-light);
    font-size: 0.9rem;
    cursor: pointer;
    position: relative;
    transition: all 0.2s ease;
  }
  
  .causal-tab.active {
    color: var(--electric-teal);
  }
  
  .causal-tab.active::after {
    content: '';
    position: absolute;
    bottom: -1px;
    left: 0;
    width: 100%;
    height: 2px;
    background-color: var(--electric-teal);
  }
  
  .causal-content {
    flex: 1;
    position: relative;
    height: 100%;
  }
  
  .causal-panel {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: none;
    overflow-y: auto;
  }
</style>