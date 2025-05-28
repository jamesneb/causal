<script lang="ts">
  import { onMount } from 'svelte';
  import type { Trace, Span } from '../timeline/types';
  
  // Props
  export let trace: Trace | null = null;
  export let selectedSpan: Span | null = null;
  export let onSpanSelect = (spanId: string) => {};
  
  // State
  let rootCauses: {spanId: string, confidence: number, reason: string}[] = [];
  let loading = false;
  let expanded = true;
  let selectedRootCause: string | null = null;
  
  // Analyze trace whenever it changes
  $: if (trace && selectedSpan) {
    analyzeRootCauses();
  }
  
  // Mock function to analyze root causes
  // In a real implementation, this would call a backend service
  async function analyzeRootCauses() {
    if (!trace || !selectedSpan) return;
    
    loading = true;
    
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Mock response - in real implementation, this would come from backend
    rootCauses = [
      {
        spanId: "span-123", 
        confidence: 0.92,
        reason: "High latency in database query execution"
      },
      {
        spanId: "span-456", 
        confidence: 0.85,
        reason: "Network timeout in external API call"
      },
      {
        spanId: "span-789", 
        confidence: 0.68,
        reason: "Resource contention in container"
      }
    ];
    
    loading = false;
  }
  
  function handleCauseClick(spanId: string) {
    selectedRootCause = selectedRootCause === spanId ? null : spanId;
  }
  
  function handleSpanSelect(spanId: string) {
    onSpanSelect(spanId);
  }
  
  function toggleExpanded() {
    expanded = !expanded;
  }
</script>

<div class="root-cause-panel">
  <div class="panel-header">
    <h3>Root Cause Analysis</h3>
    <button class="toggle-button" on:click={toggleExpanded}>
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="{expanded ? '18 15 12 9 6 15' : '6 9 12 15 18 9'}"></polyline>
      </svg>
    </button>
  </div>
  
  {#if expanded}
    <div class="panel-content">
      {#if loading}
        <div class="loading-state">
          <div class="loading-spinner"></div>
          <p>Analyzing potential root causes...</p>
        </div>
      {:else if rootCauses.length === 0}
        <div class="empty-state">
          <p>Select an error or warning span to analyze root causes.</p>
        </div>
      {:else}
        <div class="cause-list">
          {#each rootCauses as cause, index}
            <div class="cause-item" class:selected={selectedRootCause === cause.spanId} on:click={() => handleCauseClick(cause.spanId)}>
              <div class="cause-header">
                <div class="cause-number">{index + 1}</div>
                <div class="cause-confidence">
                  <div class="confidence-bar">
                    <div class="confidence-level" style="width: {cause.confidence * 100}%"></div>
                  </div>
                  <div class="confidence-value">{Math.round(cause.confidence * 100)}%</div>
                </div>
              </div>
              
              <p class="cause-reason">{cause.reason}</p>
              
              {#if selectedRootCause === cause.spanId}
                <div class="cause-details">
                  <p>This issue appears to be related to the following metrics:</p>
                  <ul class="metric-list">
                    <li>
                      <span class="metric-name">CPU Utilization</span>
                      <span class="metric-value high">87%</span>
                    </li>
                    <li>
                      <span class="metric-name">Memory Usage</span>
                      <span class="metric-value medium">74%</span>
                    </li>
                    <li>
                      <span class="metric-name">Error Rate</span>
                      <span class="metric-value high">12.3%</span>
                    </li>
                  </ul>
                  
                  <button class="btn-primary" on:click={() => handleSpanSelect(cause.spanId)}>
                    View Related Span
                  </button>
                </div>
              {/if}
            </div>
          {/each}
        </div>
        
        <div class="actions">
          <button class="btn-secondary">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
              <polyline points="7 10 12 15 17 10" />
              <line x1="12" y1="15" x2="12" y2="3" />
            </svg>
            <span>Export Analysis</span>
          </button>
          
          <button class="btn-secondary">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="23 4 23 10 17 10" />
              <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
            </svg>
            <span>Refresh Analysis</span>
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .root-cause-panel {
    background-color: var(--bg-tertiary);
    border-radius: 8px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
    margin-bottom: 1.5rem;
    border: 1px solid var(--border-medium);
    overflow: hidden;
    position: relative;
  }
  
  .root-cause-panel::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: linear-gradient(90deg, var(--plasma-pink), var(--electric-teal));
    z-index: 1;
  }
  
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border-medium);
  }
  
  .panel-header h3 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0;
    color: var(--text-primary);
  }
  
  .toggle-button {
    background: none;
    border: none;
    color: var(--text-light);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .toggle-button:hover {
    background-color: var(--bg-hover);
  }
  
  .panel-content {
    padding: 1rem;
  }
  
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem 0;
    color: var(--text-light);
  }
  
  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(0, 245, 212, 0.2);
    border-top-color: var(--electric-teal);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .empty-state {
    text-align: center;
    padding: 2rem 0;
    color: var(--text-secondary);
  }
  
  .cause-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }
  
  .cause-item {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-light);
    border-radius: 4px;
    padding: 0.75rem;
    cursor: pointer;
    transition: all 0.2s ease;
    position: relative;
  }
  
  .cause-item:hover {
    border-color: var(--border-medium);
    background-color: var(--bg-hover);
  }
  
  .cause-item.selected {
    border-color: var(--electric-teal);
    border-left: 3px solid var(--electric-teal);
  }
  
  .cause-header {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    margin-bottom: 0.5rem;
  }
  
  .cause-number {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background-color: var(--bg-tertiary);
    border-radius: 50%;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-light);
  }
  
  .cause-confidence {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .confidence-bar {
    flex: 1;
    height: 8px;
    background-color: rgba(0, 0, 0, 0.25);
    border-radius: 4px;
    overflow: hidden;
    box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.2);
    position: relative;
  }
  
  .confidence-level {
    height: 100%;
    background: linear-gradient(90deg, var(--electric-teal), #4CFFE1);
    border-radius: 4px;
    position: relative;
    box-shadow: 0 0 8px rgba(0, 245, 212, 0.5);
    transition: width 0.3s ease-out;
  }
  
  .confidence-level::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
      to bottom,
      rgba(255, 255, 255, 0.15) 0%,
      rgba(255, 255, 255, 0) 100%
    );
    border-radius: 4px;
  }
  
  .confidence-value {
    font-size: 0.85rem;
    color: var(--electric-teal);
    font-weight: 600;
    min-width: 40px;
    text-align: right;
  }
  
  .cause-reason {
    margin: 0;
    font-size: 0.9rem;
    color: var(--text-primary);
    line-height: 1.4;
  }
  
  .cause-details {
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px dashed var(--border-light);
  }
  
  .cause-details p {
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }
  
  .metric-list {
    list-style: none;
    padding: 0;
    margin: 0 0 1rem;
  }
  
  .metric-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.4rem 0;
    font-size: 0.85rem;
    border-bottom: 1px dotted var(--border-light);
  }
  
  .metric-name {
    color: var(--text-secondary);
  }
  
  .metric-value {
    font-weight: 600;
    font-family: monospace;
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
    font-size: 0.8rem;
  }
  
  .metric-value.high {
    background-color: rgba(255, 56, 100, 0.15);
    color: var(--error);
  }
  
  .metric-value.medium {
    background-color: rgba(255, 214, 0, 0.15);
    color: var(--warning);
  }
  
  .metric-value.low {
    background-color: rgba(0, 245, 212, 0.15);
    color: var(--electric-teal);
  }
  
  .btn-primary {
    background: linear-gradient(to right, var(--electric-teal), #4CFFE1);
    color: var(--deep-space);
    border: none;
    border-radius: 6px;
    font-weight: 600;
    font-size: 0.85rem;
    padding: 0.6rem 0.75rem;
    cursor: pointer;
    transition: all 0.2s ease;
    width: 100%;
    position: relative;
    overflow: hidden;
    box-shadow: 0 4px 10px rgba(0, 245, 212, 0.3);
  }
  
  .btn-primary::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(
      90deg, 
      rgba(255, 255, 255, 0) 0%, 
      rgba(255, 255, 255, 0.2) 50%, 
      rgba(255, 255, 255, 0) 100%
    );
    transition: left 0.6s ease;
  }
  
  .btn-primary:hover {
    transform: translateY(-1px);
    box-shadow: 0 6px 14px rgba(0, 245, 212, 0.4);
  }
  
  .btn-primary:hover::before {
    left: 100%;
  }
  
  .btn-primary:active {
    transform: translateY(1px);
    box-shadow: 0 2px 6px rgba(0, 245, 212, 0.4);
  }
  
  .actions {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
  }
  
  .btn-secondary {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
    justify-content: center;
    padding: 0.6rem 0.75rem;
    background: rgba(0, 245, 212, 0.05);
    border: 1px solid rgba(0, 245, 212, 0.2);
    border-radius: 6px;
    color: var(--text-light);
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    position: relative;
    overflow: hidden;
  }
  
  .btn-secondary svg {
    transition: transform 0.2s ease;
  }
  
  .btn-secondary::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: linear-gradient(
      135deg,
      rgba(0, 245, 212, 0.1) 0%,
      rgba(0, 245, 212, 0) 50%
    );
    opacity: 0;
    transition: opacity 0.2s ease;
  }
  
  .btn-secondary:hover {
    background-color: rgba(0, 245, 212, 0.08);
    border-color: var(--electric-teal);
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  }
  
  .btn-secondary:hover::before {
    opacity: 1;
  }
  
  .btn-secondary:hover svg {
    transform: scale(1.1);
  }
  
  .btn-secondary:active {
    transform: translateY(1px);
  }
</style>