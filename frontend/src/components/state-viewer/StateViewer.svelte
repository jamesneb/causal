<script lang="ts">
  import { onMount } from 'svelte';
  import type { ServiceState, StateSnapshot } from './types';
  
  // Props
  export let serviceState: ServiceState | null = null;
  export let snapshots: StateSnapshot[] = [];
  
  // Internal state
  let activeTab: 'state' | 'memory' | 'environment' | 'connections' | 'history' = 'state';
  let expandedPaths: Set<string> = new Set();
  let searchTerm = '';
  let timelinePosition = 100; // 0-100, represents percentage, 100 is current state
  let filterEmpty = false;
  let currentSnapshot: StateSnapshot | null = null;
  
  // Set current snapshot based on timeline position
  $: {
    if (snapshots.length > 0 && timelinePosition < 100) {
      const index = Math.floor((snapshots.length - 1) * (timelinePosition / 100));
      currentSnapshot = snapshots[index];
    } else {
      currentSnapshot = null;
    }
  }
  
  // Get state to display (either current or historical)
  $: displayState = currentSnapshot ? currentSnapshot.state : serviceState;
  
  // Toggle path expansion
  function togglePath(path: string) {
    if (expandedPaths.has(path)) {
      expandedPaths.delete(path);
    } else {
      expandedPaths.add(path);
    }
    expandedPaths = expandedPaths; // Trigger reactivity
  }
  
  // Expand all paths
  function expandAll() {
    if (!displayState) return;
    
    function traverseAndCollectPaths(obj: any, currentPath = '') {
      if (typeof obj !== 'object' || obj === null) return;
      
      Object.keys(obj).forEach(key => {
        const newPath = currentPath ? `${currentPath}.${key}` : key;
        expandedPaths.add(newPath);
        
        if (typeof obj[key] === 'object' && obj[key] !== null) {
          traverseAndCollectPaths(obj[key], newPath);
        }
      });
    }
    
    traverseAndCollectPaths(displayState);
    expandedPaths = expandedPaths; // Trigger reactivity
  }
  
  // Collapse all paths
  function collapseAll() {
    expandedPaths.clear();
    expandedPaths = expandedPaths; // Trigger reactivity
  }
  
  // Get icon for value type
  function getTypeIcon(value: any): string {
    if (value === null) return 'null';
    if (Array.isArray(value)) return 'array';
    if (typeof value === 'object') return 'object';
    if (typeof value === 'string') return 'string';
    if (typeof value === 'number') return 'number';
    if (typeof value === 'boolean') return 'boolean';
    return 'unknown';
  }
  
  // Format value for display
  function formatValue(value: any): string {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value === 'string') return `"${value}"`;
    if (typeof value === 'object') {
      if (Array.isArray(value)) {
        return `Array(${value.length})`;
      }
      return `Object {${Object.keys(value).length} keys}`;
    }
    return String(value);
  }
  
  // Check if object is empty
  function isEmpty(obj: any): boolean {
    if (obj === null || obj === undefined) return true;
    if (Array.isArray(obj) && obj.length === 0) return true;
    if (typeof obj === 'object' && Object.keys(obj).length === 0) return true;
    return false;
  }
  
  // Render object tree recursively
  function renderTree(data: any, basePath = '', level = 0) {
    if (!data || typeof data !== 'object') return null;
    
    // Filter keys based on search term
    const keys = Object.keys(data).filter(key => {
      if (searchTerm === '') return true;
      return key.toLowerCase().includes(searchTerm.toLowerCase());
    });
    
    if (keys.length === 0) return null;
    
    return keys.map(key => {
      const value = data[key];
      const path = basePath ? `${basePath}.${key}` : key;
      const isExpanded = expandedPaths.has(path);
      const isObject = value !== null && typeof value === 'object';
      
      // Skip empty objects/arrays if filter is enabled
      if (filterEmpty && isObject && isEmpty(value)) return null;
      
      return `
        <div class="tree-node" style="padding-left: ${level * 16}px">
          <div class="node-content ${isObject ? 'expandable' : ''}" onclick="${isObject ? `togglePath('${path}')` : ''}">
            <span class="expand-icon ${isObject ? '' : 'invisible'}">
              ${isObject ? (isExpanded ? '▼' : '▶') : ''}
            </span>
            <span class="key-name">${key}</span>
            <span class="key-value ${getTypeIcon(value)}">
              ${formatValue(value)}
            </span>
          </div>
          ${isObject && isExpanded ? 
            `<div class="children">${renderTree(value, path, level + 1) || ''}</div>` : 
            ''}
        </div>
      `;
    }).filter(Boolean).join('');
  }
  
  // Update the DOM with rendered tree
  $: if (displayState) {
    onMount(() => {
      updateRenderedTree();
    });
  }
  
  $: if (displayState) {
    updateRenderedTree();
  }
  
  $: if (searchTerm !== undefined || expandedPaths || filterEmpty !== undefined) {
    if (displayState) {
      updateRenderedTree();
    }
  }
  
  function updateRenderedTree() {
    const treeContainer = document.getElementById('state-tree-container');
    if (treeContainer && displayState) {
      const treeContent = renderTree(displayState);
      
      // Use a documentFragment for better performance
      const temp = document.createElement('div');
      temp.innerHTML = treeContent || '<div class="empty-state">No data to display</div>';
      
      // Clear existing content and append new content
      treeContainer.innerHTML = '';
      while (temp.firstChild) {
        treeContainer.appendChild(temp.firstChild);
      }
      
      // Add event listeners for expand/collapse
      treeContainer.querySelectorAll('.expandable').forEach(node => {
        node.addEventListener('click', (e) => {
          const path = node.closest('.tree-node').getAttribute('data-path');
          if (path) togglePath(path);
          e.stopPropagation();
        });
      });
    }
  }
  
  // Format timestamp
  function formatTime(timestamp: number): string {
    return new Date(timestamp).toLocaleTimeString();
  }
  
  // Handle timeline change
  function handleTimelineChange(e: Event) {
    const input = e.target as HTMLInputElement;
    timelinePosition = parseInt(input.value);
  }
</script>

<div class="state-viewer">
  {#if displayState || serviceState}
    <div class="state-controls">
      <div class="tabs">
        <button 
          class="tab-button" 
          class:active={activeTab === 'state'} 
          on:click={() => activeTab = 'state'}
        >
          State
        </button>
        
        <button 
          class="tab-button" 
          class:active={activeTab === 'memory'} 
          on:click={() => activeTab = 'memory'}
        >
          Memory
        </button>
        
        <button 
          class="tab-button" 
          class:active={activeTab === 'environment'} 
          on:click={() => activeTab = 'environment'}
        >
          Environment
        </button>
        
        <button 
          class="tab-button" 
          class:active={activeTab === 'connections'} 
          on:click={() => activeTab = 'connections'}
        >
          Connections
        </button>
        
        <button 
          class="tab-button" 
          class:active={activeTab === 'history'} 
          on:click={() => activeTab = 'history'}
        >
          History
        </button>
      </div>
      
      <div class="toolbar">
        <div class="search-container">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input 
            type="text" 
            placeholder="Search keys..." 
            bind:value={searchTerm}
          />
        </div>
        
        <label class="filter-toggle">
          <input 
            type="checkbox" 
            bind:checked={filterEmpty} 
          />
          <span>Hide empty</span>
        </label>
        
        <button 
          class="btn-sm" 
          on:click={expandAll}
        >
          Expand All
        </button>
        
        <button 
          class="btn-sm" 
          on:click={collapseAll}
        >
          Collapse All
        </button>
      </div>
    </div>
    
    {#if snapshots.length > 0}
      <div class="timeline-container">
        <div class="timeline-controls">
          <span class="timeline-label">Past</span>
          <input 
            type="range" 
            min="0" 
            max="100" 
            bind:value={timelinePosition} 
            on:input={handleTimelineChange}
          />
          <span class="timeline-label">Current</span>
        </div>
        
        <div class="timeline-info">
          {#if currentSnapshot}
            <div class="current-snapshot">
              Viewing state at: {formatTime(currentSnapshot.timestamp)}
              <button 
                class="btn-sm" 
                on:click={() => timelinePosition = 100}
              >
                Return to Current
              </button>
            </div>
          {:else}
            <div class="current-snapshot">
              Viewing current state
            </div>
          {/if}
        </div>
      </div>
    {/if}
    
    <div class="state-content">
      {#if activeTab === 'state'}
        <div 
          id="state-tree-container" 
          class="tree-container"
        >
          <!-- Tree will be rendered here -->
          <div class="loading">Loading state data...</div>
        </div>
      {:else if activeTab === 'memory'}
        <div class="memory-stats">
          {#if displayState?.memory}
            <div class="memory-chart">
              <div class="chart-header">
                <h3>Memory Usage</h3>
                <div class="memory-total">
                  {Math.round(displayState.memory.used / 1024 / 1024)} MB / 
                  {Math.round(displayState.memory.total / 1024 / 1024)} MB
                </div>
              </div>
              
              <div class="memory-bar">
                <div 
                  class="memory-used" 
                  style="width: {(displayState.memory.used / displayState.memory.total) * 100}%"
                >
                </div>
              </div>
              
              <div class="memory-details">
                <div class="memory-detail">
                  <span class="detail-label">Heap Used:</span>
                  <span class="detail-value">
                    {Math.round(displayState.memory.heapUsed / 1024 / 1024)} MB
                  </span>
                </div>
                
                <div class="memory-detail">
                  <span class="detail-label">External:</span>
                  <span class="detail-value">
                    {Math.round(displayState.memory.external / 1024 / 1024)} MB
                  </span>
                </div>
                
                <div class="memory-detail">
                  <span class="detail-label">Array Buffers:</span>
                  <span class="detail-value">
                    {Math.round(displayState.memory.arrayBuffers / 1024 / 1024)} MB
                  </span>
                </div>
              </div>
            </div>
            
            {#if displayState.memory.gc}
              <div class="gc-stats">
                <h3>Garbage Collection</h3>
                
                <div class="gc-details">
                  <div class="gc-detail">
                    <span class="detail-label">Last GC:</span>
                    <span class="detail-value">
                      {formatTime(displayState.memory.gc.lastRun)}
                    </span>
                  </div>
                  
                  <div class="gc-detail">
                    <span class="detail-label">Total Pauses:</span>
                    <span class="detail-value">
                      {displayState.memory.gc.totalPauses.toFixed(2)} ms
                    </span>
                  </div>
                  
                  <div class="gc-detail">
                    <span class="detail-label">GC Count:</span>
                    <span class="detail-value">
                      {displayState.memory.gc.count}
                    </span>
                  </div>
                </div>
              </div>
            {/if}
          {:else}
            <div class="empty-state">
              No memory data available
            </div>
          {/if}
        </div>
      {:else if activeTab === 'environment'}
        <div class="environment-variables">
          {#if displayState?.environment}
            <div class="env-section">
              <h3>Runtime Environment</h3>
              
              <div class="env-details">
                <div class="env-detail">
                  <span class="detail-label">Node Version:</span>
                  <span class="detail-value">
                    {displayState.environment.runtime?.version || 'Unknown'}
                  </span>
                </div>
                
                <div class="env-detail">
                  <span class="detail-label">Platform:</span>
                  <span class="detail-value">
                    {displayState.environment.os?.platform || 'Unknown'} ({displayState.environment.os?.arch || 'Unknown'})
                  </span>
                </div>
                
                <div class="env-detail">
                  <span class="detail-label">Process Uptime:</span>
                  <span class="detail-value">
                    {Math.floor((displayState.environment.uptime || 0) / 60)} minutes
                  </span>
                </div>
              </div>
            </div>
            
            {#if displayState.environment.variables}
              <div class="env-section">
                <h3>Environment Variables</h3>
                
                <div class="env-vars">
                  {#each Object.entries(displayState.environment.variables) as [key, value]}
                    <div class="env-var">
                      <span class="env-key">{key}</span>
                      <span class="env-value">
                        {key.includes('SECRET') || key.includes('KEY') || key.includes('TOKEN') || key.includes('PASSWORD') 
                          ? '********' 
                          : value}
                      </span>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          {:else}
            <div class="empty-state">
              No environment data available
            </div>
          {/if}
        </div>
      {:else if activeTab === 'connections'}
        <div class="connections">
          {#if displayState?.connections}
            <div class="connections-summary">
              <div class="summary-card">
                <div class="summary-value">
                  {displayState.connections.current}
                </div>
                <div class="summary-label">
                  Active Connections
                </div>
              </div>
              
              <div class="summary-card">
                <div class="summary-value">
                  {displayState.connections.total}
                </div>
                <div class="summary-label">
                  Total Connections
                </div>
              </div>
              
              <div class="summary-card">
                <div class="summary-value">
                  {displayState.connections.peak}
                </div>
                <div class="summary-label">
                  Peak Connections
                </div>
              </div>
            </div>
            
            {#if displayState.connections.list && displayState.connections.list.length > 0}
              <div class="connections-list">
                <table class="connections-table">
                  <thead>
                    <tr>
                      <th>ID</th>
                      <th>Type</th>
                      <th>Remote</th>
                      <th>Status</th>
                      <th>Duration</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each displayState.connections.list as connection}
                      <tr>
                        <td class="connection-id">{connection.id}</td>
                        <td>{connection.type}</td>
                        <td>{connection.remote}</td>
                        <td>
                          <span class="status-badge" class:{connection.status}>
                            {connection.status}
                          </span>
                        </td>
                        <td>{connection.duration}ms</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {:else}
              <div class="empty-state">
                No active connections
              </div>
            {/if}
          {:else}
            <div class="empty-state">
              No connection data available
            </div>
          {/if}
        </div>
      {:else if activeTab === 'history'}
        <div class="history">
          {#if snapshots && snapshots.length > 0}
            <div class="history-list">
              <table class="history-table">
                <thead>
                  <tr>
                    <th>Timestamp</th>
                    <th>Type</th>
                    <th>Changes</th>
                    <th>Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {#each snapshots as snapshot}
                    <tr class={currentSnapshot?.id === snapshot.id ? 'current-snapshot-row' : ''}>
                      <td>{formatTime(snapshot.timestamp)}</td>
                      <td>
                        <span class="tag">{snapshot.type}</span>
                      </td>
                      <td>
                        {snapshot.changedPaths?.length || 0} paths changed
                      </td>
                      <td>
                        <button 
                          class="btn-sm" 
                          on:click={() => {
                            const index = snapshots.findIndex(s => s.id === snapshot.id);
                            if (index >= 0) {
                              timelinePosition = index / (snapshots.length - 1) * 100;
                            }
                          }}
                        >
                          View
                        </button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {:else}
            <div class="empty-state">
              No history data available
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {:else}
    <div class="empty-state centered">
      <p>No service state available</p>
    </div>
  {/if}
</div>

<style>
  .state-viewer {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-radius: 8px;
    overflow: hidden;
  }
  
  .state-controls {
    padding: 0.5rem;
    border-bottom: 1px solid var(--border-light);
  }
  
  .tabs {
    display: flex;
    margin-bottom: 0.5rem;
  }
  
  .tab-button {
    padding: 0.5rem 1rem;
    background: none;
    border: none;
    border-radius: 4px;
    color: var(--text-light);
    font-size: 0.9rem;
    cursor: pointer;
    margin-right: 0.25rem;
  }
  
  .tab-button:hover {
    background-color: var(--bg-tertiary);
  }
  
  .tab-button.active {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--electric-teal);
    color: var(--electric-teal);
  }
  
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .search-container {
    position: relative;
    flex: 0 0 200px;
  }
  
  .search-container svg {
    position: absolute;
    left: 8px;
    top: 50%;
    transform: translateY(-50%);
    color: rgba(255, 255, 255, 0.5);
  }
  
  .search-container input {
    width: 100%;
    padding: 0.4rem 0.5rem 0.4rem 2rem;
    border-radius: 4px;
    border: 1px solid var(--border-light);
    background-color: var(--bg-tertiary);
    color: var(--text-light);
    font-size: 0.85rem;
  }
  
  .search-container input:focus {
    outline: none;
    border-color: var(--electric-teal);
    box-shadow: 0 0 0 1px var(--electric-teal);
  }
  
  .filter-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    cursor: pointer;
  }
  
  .filter-toggle input {
    width: 16px;
    height: 16px;
    accent-color: var(--electric-teal);
  }
  
  .btn-sm {
    padding: 0.4rem 0.75rem;
    background: none;
    border: 1px solid var(--border-light);
    border-radius: 4px;
    color: var(--text-light);
    font-size: 0.85rem;
    cursor: pointer;
  }
  
  .btn-sm:hover {
    background-color: var(--bg-tertiary);
    border-color: var(--border-medium);
  }
  
  .timeline-container {
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--border-light);
  }
  
  .timeline-controls {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .timeline-label {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.7);
  }
  
  .timeline-controls input[type="range"] {
    flex: 1;
    accent-color: var(--electric-teal);
  }
  
  .timeline-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 0.5rem;
  }
  
  .current-snapshot {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.7);
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .state-content {
    flex: 1;
    overflow: auto;
    padding: 0.5rem;
  }
  
  .tree-container {
    font-family: monospace;
    font-size: 0.9rem;
    height: 100%;
    overflow: auto;
  }
  
  .tree-node {
    line-height: 1.5;
  }
  
  .node-content {
    display: flex;
    align-items: center;
    padding: 0.2rem 0;
    border-radius: 2px;
    cursor: default;
  }
  
  .node-content.expandable {
    cursor: pointer;
  }
  
  .node-content:hover {
    background-color: var(--bg-tertiary);
  }
  
  .expand-icon {
    display: inline-block;
    width: 16px;
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.7);
  }
  
  .expand-icon.invisible {
    visibility: hidden;
  }
  
  .key-name {
    color: var(--electric-teal);
    margin-right: 0.5rem;
  }
  
  .key-value {
    color: var(--text-light);
  }
  
  .key-value.string {
    color: var(--plasma-pink);
  }
  
  .key-value.number {
    color: var(--signal-yellow);
  }
  
  .key-value.boolean {
    color: var(--quantum-purple);
  }
  
  .key-value.null {
    color: rgba(255, 255, 255, 0.5);
    font-style: italic;
  }
  
  .empty-state {
    color: rgba(255, 255, 255, 0.5);
    font-size: 0.95rem;
    padding: 1rem;
  }
  
  .empty-state.centered {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
  }
  
  .loading {
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.95rem;
    padding: 1rem;
  }
  
  .memory-stats {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    padding: 1rem;
  }
  
  .memory-chart, .gc-stats {
    background-color: var(--bg-tertiary);
    border-radius: 8px;
    padding: 1rem;
  }
  
  .chart-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  
  .chart-header h3, .gc-stats h3 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0;
  }
  
  .memory-total {
    font-size: 0.9rem;
    color: rgba(255, 255, 255, 0.7);
  }
  
  .memory-bar {
    height: 24px;
    background-color: rgba(0, 0, 0, 0.2);
    border-radius: 12px;
    overflow: hidden;
    margin-bottom: 1rem;
  }
  
  .memory-used {
    height: 100%;
    background: linear-gradient(90deg, var(--electric-teal), var(--quantum-purple));
    border-radius: 12px;
  }
  
  .memory-details, .gc-details {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 1rem;
  }
  
  .memory-detail, .gc-detail, .env-detail {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .detail-label {
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.6);
  }
  
  .detail-value {
    font-size: 0.9rem;
  }
  
  .environment-variables {
    padding: 1rem;
  }
  
  .env-section {
    margin-bottom: 2rem;
  }
  
  .env-section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 1rem 0;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--border-light);
  }
  
  .env-details {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
    background-color: var(--bg-tertiary);
    border-radius: 8px;
    padding: 1rem;
  }
  
  .env-vars {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    background-color: var(--bg-tertiary);
    border-radius: 8px;
    padding: 1rem;
  }
  
  .env-var {
    display: flex;
    font-size: 0.9rem;
  }
  
  .env-key {
    flex: 0 0 240px;
    color: var(--electric-teal);
    font-family: monospace;
  }
  
  .env-value {
    flex: 1;
    font-family: monospace;
  }
  
  .connections {
    padding: 1rem;
  }
  
  .connections-summary {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
    margin-bottom: 1.5rem;
  }
  
  .summary-card {
    background-color: var(--bg-tertiary);
    border-radius: 8px;
    padding: 1rem;
    text-align: center;
  }
  
  .summary-value {
    font-size: 2rem;
    font-weight: 700;
    color: var(--electric-teal);
    margin-bottom: 0.5rem;
  }
  
  .summary-label {
    font-size: 0.9rem;
    color: rgba(255, 255, 255, 0.7);
  }
  
  .connections-table, .history-table {
    width: 100%;
    border-collapse: collapse;
  }
  
  .connections-table th, 
  .connections-table td,
  .history-table th,
  .history-table td {
    padding: 0.75rem;
    text-align: left;
    font-size: 0.9rem;
    border-bottom: 1px solid var(--border-light);
  }
  
  .connections-table th,
  .history-table th {
    font-weight: 600;
    color: rgba(255, 255, 255, 0.9);
    background-color: var(--bg-tertiary);
  }
  
  .connection-id {
    font-family: monospace;
    font-size: 0.85rem;
  }
  
  .status-badge {
    display: inline-block;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    text-transform: uppercase;
    font-size: 0.75rem;
    font-weight: 500;
  }
  
  .status-badge.active, 
  .status-badge.open, 
  .status-badge.connected {
    background-color: rgba(0, 230, 118, 0.2);
    color: var(--success);
  }
  
  .status-badge.idle,
  .status-badge.waiting {
    background-color: rgba(66, 165, 245, 0.2);
    color: var(--info);
  }
  
  .status-badge.closing, 
  .status-badge.warning {
    background-color: rgba(255, 214, 0, 0.2);
    color: var(--warning);
  }
  
  .status-badge.error,
  .status-badge.closed {
    background-color: rgba(255, 56, 100, 0.2);
    color: var(--error);
  }
  
  .tag {
    display: inline-block;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    background-color: var(--bg-tertiary);
  }
  
  .history {
    padding: 1rem;
  }
  
  .current-snapshot-row {
    background-color: rgba(0, 245, 212, 0.1);
  }
</style>