<script lang="ts">
  import { TraceViewer } from '../../components/trace-viewer';
  import type { Trace, TimelineEvent } from '../../components/timeline/types';
  import { onMount } from 'svelte';
  
  // Sample trace data
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
  
  // State
  let selectedTrace: Trace | null = sampleTrace;
  let filteredEvents: TimelineEvent[] = sampleEvents;
  let expandedSpanId: string | null = null;
  
  // Sample list of traces
  const traces = [
    {
      id: 'trace-123456789',
      name: 'GET /api/products',
      timestamp: Date.now() - 350,
      duration: 350,
      status: 'success'
    },
    {
      id: 'trace-987654321',
      name: 'POST /api/orders',
      timestamp: Date.now() - 5000,
      duration: 620,
      status: 'error'
    },
    {
      id: 'trace-456789123',
      name: 'GET /api/users/profile',
      timestamp: Date.now() - 15000,
      duration: 180,
      status: 'success'
    },
    {
      id: 'trace-789123456',
      name: 'PUT /api/cart',
      timestamp: Date.now() - 25000,
      duration: 270,
      status: 'success'
    },
    {
      id: 'trace-321654987',
      name: 'DELETE /api/cart/item',
      timestamp: Date.now() - 35000,
      duration: 140,
      status: 'success'
    }
  ];
</script>

<div class="traces-page">
  <div class="traces-header">
    <h1>Distributed Traces</h1>
    
    <div class="search-filters">
      <div class="search-input">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input type="text" placeholder="Search traces..." />
      </div>
      
      <div class="filter-actions">
        <select class="filter-select">
          <option>All Services</option>
          <option>API Gateway</option>
          <option>Auth Service</option>
          <option>Product Service</option>
          <option>User Service</option>
          <option>Database</option>
        </select>
        
        <select class="filter-select">
          <option>All Statuses</option>
          <option>Success</option>
          <option>Warning</option>
          <option>Error</option>
        </select>
        
        <select class="filter-select">
          <option>Last 15 minutes</option>
          <option>Last 1 hour</option>
          <option>Last 6 hours</option>
          <option>Last 24 hours</option>
          <option>Custom time range</option>
        </select>
      </div>
    </div>
  </div>
  
  <div class="traces-content">
    <div class="traces-list-container">
      <div class="list-header">
        <h2>Recent Traces</h2>
        <span class="count-badge">{traces.length} traces</span>
        
        <div class="list-actions">
          <button class="btn-icon refresh" title="Refresh">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M1 4v6h6"></path>
              <path d="M23 20v-6h-6"></path>
              <path d="M20.49 9A9 9 0 005.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 013.51 15"></path>
            </svg>
          </button>
        </div>
      </div>
      
      <div class="traces-list">
        {#each traces as trace}
          <div 
            class="trace-item" 
            class:selected={selectedTrace?.id === trace.id}
            class:error={trace.status === 'error'} 
            on:click={() => selectedTrace = trace.id === 'trace-123456789' ? sampleTrace : null}
          >
            <div class="trace-item-header">
              <div class="trace-name">{trace.name}</div>
              <div class="trace-time">{new Date(trace.timestamp).toLocaleTimeString()}</div>
            </div>
            
            <div class="trace-item-details">
              <div class="trace-duration">
                <span class="label">Duration:</span>
                <span class="value">{trace.duration}ms</span>
              </div>
              
              <div class="trace-status" class:error={trace.status === 'error'}>
                <span class="status-indicator"></span>
                <span>{trace.status}</span>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>
    
    <div class="trace-detail-container">
      <TraceViewer 
        trace={selectedTrace} 
        events={filteredEvents}
        expandedSpanId={expandedSpanId}
      />
    </div>
  </div>
</div>

<style>
  .traces-page {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .traces-header {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  h1 {
    font-size: 1.8rem;
    font-weight: 700;
    margin: 0;
  }
  
  .search-filters {
    display: flex;
    align-items: center;
    gap: 1rem;
  }
  
  .search-input {
    position: relative;
    flex: 0 0 300px;
  }
  
  .search-input svg {
    position: absolute;
    left: 10px;
    top: 50%;
    transform: translateY(-50%);
    color: rgba(255, 255, 255, 0.5);
  }
  
  .search-input input {
    width: 100%;
    padding: 0.6rem 0.75rem 0.6rem 2.25rem;
    border-radius: 6px;
    border: 1px solid var(--border-light);
    background-color: var(--bg-tertiary);
    color: var(--text-light);
    font-size: 0.9rem;
  }
  
  .search-input input:focus {
    outline: none;
    border-color: var(--electric-teal);
    box-shadow: 0 0 0 1px var(--electric-teal);
  }
  
  .filter-actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .filter-select {
    padding: 0.6rem 0.75rem;
    border-radius: 6px;
    border: 1px solid var(--border-light);
    background-color: var(--bg-tertiary);
    color: var(--text-light);
    font-size: 0.9rem;
    cursor: pointer;
  }
  
  .filter-select:focus {
    outline: none;
    border-color: var(--electric-teal);
  }
  
  .traces-content {
    display: grid;
    grid-template-columns: 350px 1fr;
    gap: 1.5rem;
    flex: 1;
    overflow: hidden;
  }
  
  .traces-list-container {
    display: flex;
    flex-direction: column;
    background-color: var(--bg-secondary);
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    border: 1px solid var(--border-light);
  }
  
  .list-header {
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-light);
    display: flex;
    align-items: center;
  }
  
  .list-header h2 {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0;
  }
  
  .count-badge {
    background-color: var(--bg-tertiary);
    padding: 0.2rem 0.5rem;
    border-radius: 50px;
    font-size: 0.75rem;
    margin-left: 0.75rem;
    color: rgba(255, 255, 255, 0.7);
  }
  
  .list-actions {
    margin-left: auto;
  }
  
  .btn-icon {
    background: none;
    border: none;
    width: 28px;
    height: 28px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-light);
    cursor: pointer;
    transition: background-color 0.2s ease;
  }
  
  .btn-icon:hover {
    background-color: var(--bg-tertiary);
  }
  
  .traces-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }
  
  .trace-item {
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 0.5rem;
    cursor: pointer;
    border-left: 3px solid transparent;
    transition: all 0.2s ease;
  }
  
  .trace-item:hover {
    background-color: var(--bg-tertiary);
  }
  
  .trace-item.selected {
    background-color: var(--bg-tertiary);
    border-left-color: var(--electric-teal);
  }
  
  .trace-item.error {
    border-left-color: var(--error);
  }
  
  .trace-item-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }
  
  .trace-name {
    font-size: 0.9rem;
    font-weight: 500;
  }
  
  .trace-time {
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.6);
  }
  
  .trace-item-details {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.7);
  }
  
  .trace-duration {
    display: flex;
    gap: 0.25rem;
  }
  
  .trace-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--success);
  }
  
  .trace-status.error .status-indicator {
    background-color: var(--error);
  }
  
  .trace-detail-container {
    flex: 1;
    min-height: 0;
  }
</style>