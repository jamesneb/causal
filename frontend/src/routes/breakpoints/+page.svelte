<script lang="ts">
  import { ServiceMap } from '../../components/service-map';
  import type { Service, Connection } from '../../components/service-map/types';
  
  // Sample data
  let sampleBreakpoints = [
    {
      id: 'bp-1',
      name: 'Database Query Latency',
      condition: 'span.duration > 50 && span.service == "database"',
      service: 'database',
      status: 'active',
      triggerCount: 8,
      lastTriggered: Date.now() - 1000 * 60 * 5, // 5 minutes ago
      actions: ['pause', 'log', 'notify']
    },
    {
      id: 'bp-2',
      name: 'Payment Service Error',
      condition: 'span.status == "error" && span.service == "payment-service"',
      service: 'payment-service',
      status: 'active',
      triggerCount: 3,
      lastTriggered: Date.now() - 1000 * 60 * 30, // 30 minutes ago
      actions: ['pause', 'log', 'capture-state']
    },
    {
      id: 'bp-3',
      name: 'Auth Token Missing',
      condition: 'request.headers["Authorization"] == null && path.startsWith("/api/secure")',
      service: 'api-gateway',
      status: 'active',
      triggerCount: 12,
      lastTriggered: Date.now() - 1000 * 60 * 15, // 15 minutes ago
      actions: ['log', 'notify']
    },
    {
      id: 'bp-4',
      name: 'High Traffic Alert',
      condition: 'metrics.requestRate > 500',
      service: 'api-gateway',
      status: 'inactive',
      triggerCount: 2,
      lastTriggered: Date.now() - 1000 * 60 * 60 * 2, // 2 hours ago
      actions: ['log', 'notify']
    },
    {
      id: 'bp-5',
      name: 'Product Not Found',
      condition: 'response.status == 404 && path.contains("/products/")',
      service: 'product-service',
      status: 'active',
      triggerCount: 20,
      lastTriggered: Date.now() - 1000 * 60 * 10, // 10 minutes ago
      actions: ['log']
    }
  ];
  
  // State
  let selectedBreakpoint = null;
  let isCreatingBreakpoint = false;
  let searchQuery = '';
  let serviceFilter = 'all';
  let statusFilter = 'all';
  
  // Services for the service map
  const services: Service[] = [
    {
      id: 's1',
      name: 'API Gateway',
      type: 'gateway',
      status: 'healthy',
      x: 300,
      y: 150
    },
    {
      id: 's2',
      name: 'Auth Service',
      type: 'service',
      status: 'healthy',
      x: 500,
      y: 100
    },
    {
      id: 's3',
      name: 'User Service',
      type: 'service',
      status: 'healthy',
      x: 500,
      y: 250
    },
    {
      id: 's4',
      name: 'Product Service',
      type: 'service',
      status: 'warning',
      x: 700,
      y: 150
    },
    {
      id: 's5',
      name: 'Order Service',
      type: 'service',
      status: 'error',
      x: 700,
      y: 300
    },
    {
      id: 's6',
      name: 'Payment Service',
      type: 'service',
      status: 'healthy',
      x: 700,
      y: 450
    },
    {
      id: 's7',
      name: 'Database',
      type: 'database',
      status: 'healthy',
      x: 900,
      y: 250
    }
  ];
  
  const connections: Connection[] = [
    {
      id: 'c1',
      source: 's1',
      target: 's2',
      status: 'active',
      protocol: 'http'
    },
    {
      id: 'c2',
      source: 's1',
      target: 's3',
      status: 'active',
      protocol: 'http'
    },
    {
      id: 'c3',
      source: 's1',
      target: 's4',
      status: 'active',
      protocol: 'http'
    },
    {
      id: 'c4',
      source: 's1',
      target: 's5',
      status: 'slow',
      protocol: 'http'
    },
    {
      id: 'c5',
      source: 's3',
      target: 's7',
      status: 'active',
      protocol: 'db'
    },
    {
      id: 'c6',
      source: 's4',
      target: 's7',
      status: 'active',
      protocol: 'db'
    },
    {
      id: 'c7',
      source: 's5',
      target: 's7',
      status: 'error',
      protocol: 'db'
    },
    {
      id: 'c8',
      source: 's5',
      target: 's6',
      status: 'active',
      protocol: 'http'
    }
  ];
  
  // Filtered breakpoints based on search and filters
  $: filteredBreakpoints = sampleBreakpoints.filter(bp => {
    const matchesSearch = searchQuery === '' || 
                         bp.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         bp.condition.toLowerCase().includes(searchQuery.toLowerCase());
                         
    const matchesService = serviceFilter === 'all' || bp.service === serviceFilter;
    const matchesStatus = statusFilter === 'all' || bp.status === statusFilter;
    
    return matchesSearch && matchesService && matchesStatus;
  });
  
  // Format timestamp
  function formatTime(timestamp: number): string {
    const now = Date.now();
    const diffSeconds = Math.floor((now - timestamp) / 1000);
    
    if (diffSeconds < 60) return `${diffSeconds} seconds ago`;
    if (diffSeconds < 3600) return `${Math.floor(diffSeconds / 60)} minutes ago`;
    if (diffSeconds < 86400) return `${Math.floor(diffSeconds / 3600)} hours ago`;
    return `${Math.floor(diffSeconds / 86400)} days ago`;
  }
  
  // Toggle breakpoint status
  function toggleBreakpointStatus(id: string, event: Event) {
    event.stopPropagation();
    sampleBreakpoints = sampleBreakpoints.map(bp => {
      if (bp.id === id) {
        return {
          ...bp,
          status: bp.status === 'active' ? 'inactive' : 'active'
        };
      }
      return bp;
    });
  }
  
  // Handle service selection from map
  function handleServiceClick(serviceId: string) {
    const service = services.find(s => s.id === serviceId);
    if (service) {
      serviceFilter = service.name.toLowerCase().replace(' ', '-');
    }
  }
  
  // Handle create breakpoint
  function startCreateBreakpoint() {
    isCreatingBreakpoint = true;
    selectedBreakpoint = null;
  }
  
  // Cancel create/edit
  function cancelEdit() {
    isCreatingBreakpoint = false;
  }
</script>

<div class="breakpoints-page">
  <div class="breakpoints-header">
    <h1>Distributed Breakpoints</h1>
    
    <div class="header-actions">
      <button class="btn-primary" on:click={startCreateBreakpoint}>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="16" />
          <line x1="8" y1="12" x2="16" y2="12" />
        </svg>
        <span>Add Breakpoint</span>
      </button>
    </div>
  </div>
  
  <div class="breakpoints-filters">
    <div class="search-input">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <input 
        type="text" 
        placeholder="Search breakpoints..." 
        bind:value={searchQuery}
      />
    </div>
    
    <div class="filter-actions">
      <select class="filter-select" bind:value={serviceFilter}>
        <option value="all">All Services</option>
        <option value="api-gateway">API Gateway</option>
        <option value="auth-service">Auth Service</option>
        <option value="user-service">User Service</option>
        <option value="product-service">Product Service</option>
        <option value="order-service">Order Service</option>
        <option value="payment-service">Payment Service</option>
        <option value="database">Database</option>
      </select>
      
      <select class="filter-select" bind:value={statusFilter}>
        <option value="all">All Statuses</option>
        <option value="active">Active</option>
        <option value="inactive">Inactive</option>
      </select>
    </div>
  </div>
  
  <div class="breakpoints-content">
    <div class="breakpoints-list-container">
      <div class="list-header">
        <h2>
          Breakpoints
          <span class="count-badge">{filteredBreakpoints.length}</span>
        </h2>
      </div>
      
      <div class="breakpoints-list">
        {#if filteredBreakpoints.length > 0}
          {#each filteredBreakpoints as bp}
            <div 
              class="breakpoint-item"
              class:selected={selectedBreakpoint?.id === bp.id}
              on:click={() => {
                selectedBreakpoint = bp;
                isCreatingBreakpoint = false;
              }}
            >
              <div class="bp-header">
                <div class="bp-name">{bp.name}</div>
                <div 
                  class="status-toggle" 
                  class:active={bp.status === 'active'}
                  on:click={(e) => toggleBreakpointStatus(bp.id, e)}
                >
                  <div class="toggle-slider"></div>
                </div>
              </div>
              
              <div class="bp-service">{bp.service}</div>
              
              <div class="bp-condition">
                <code>{bp.condition}</code>
              </div>
              
              <div class="bp-stats">
                <div class="bp-stat">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
                  </svg>
                  <span>{bp.triggerCount} triggers</span>
                </div>
                
                <div class="bp-stat">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10" />
                    <polyline points="12 6 12 12 16 14" />
                  </svg>
                  <span>Last: {formatTime(bp.lastTriggered)}</span>
                </div>
              </div>
            </div>
          {/each}
        {:else}
          <div class="no-results">
            <p>No breakpoints found matching your filters</p>
          </div>
        {/if}
      </div>
    </div>
    
    <div class="breakpoint-detail-container">
      {#if isCreatingBreakpoint}
        <div class="card create-breakpoint">
          <div class="card-header">
            <h2>Create New Breakpoint</h2>
            <button class="btn-icon" on:click={cancelEdit} title="Cancel">
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
          
          <div class="form-container">
            <div class="form-group">
              <label for="bp-name">Breakpoint Name</label>
              <input type="text" id="bp-name" placeholder="Enter a descriptive name" />
            </div>
            
            <div class="form-group">
              <label for="bp-service">Service</label>
              <select id="bp-service">
                <option value="">Select a service</option>
                <option value="api-gateway">API Gateway</option>
                <option value="auth-service">Auth Service</option>
                <option value="user-service">User Service</option>
                <option value="product-service">Product Service</option>
                <option value="order-service">Order Service</option>
                <option value="payment-service">Payment Service</option>
                <option value="database">Database</option>
              </select>
            </div>
            
            <div class="form-group">
              <label for="bp-condition">Condition</label>
              <textarea 
                id="bp-condition" 
                rows="5" 
                placeholder="Enter condition expression, e.g: span.duration > 50"
              ></textarea>
              <div class="hint">
                Use <code>span</code>, <code>request</code>, <code>response</code>, 
                <code>service</code>, or <code>metrics</code> variables in your condition.
              </div>
            </div>
            
            <div class="form-group">
              <label>Actions</label>
              <div class="checkbox-group">
                <label class="checkbox">
                  <input type="checkbox" checked />
                  <span>Pause Execution</span>
                </label>
                
                <label class="checkbox">
                  <input type="checkbox" checked />
                  <span>Log Details</span>
                </label>
                
                <label class="checkbox">
                  <input type="checkbox" />
                  <span>Capture State</span>
                </label>
                
                <label class="checkbox">
                  <input type="checkbox" />
                  <span>Send Notification</span>
                </label>
              </div>
            </div>
            
            <div class="form-actions">
              <button class="btn-secondary" on:click={cancelEdit}>Cancel</button>
              <button class="btn-primary">Create Breakpoint</button>
            </div>
          </div>
        </div>
      {:else if selectedBreakpoint}
        <div class="card breakpoint-details">
          <div class="card-header">
            <h2>{selectedBreakpoint.name}</h2>
            <div class="card-actions">
              <button class="btn-icon" title="Edit">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" />
                </svg>
              </button>
              
              <button class="btn-icon" title="Delete">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="3 6 5 6 21 6" />
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                </svg>
              </button>
            </div>
          </div>
          
          <div class="detail-content">
            <div class="detail-section">
              <div class="detail-row">
                <div class="detail-key">Status</div>
                <div class="detail-value">
                  <div 
                    class="status-toggle large" 
                    class:active={selectedBreakpoint.status === 'active'}
                    on:click={(e) => toggleBreakpointStatus(selectedBreakpoint.id, e)}
                  >
                    <div class="toggle-slider"></div>
                  </div>
                </div>
              </div>
              
              <div class="detail-row">
                <div class="detail-key">Service</div>
                <div class="detail-value">
                  <span class="service-badge">{selectedBreakpoint.service}</span>
                </div>
              </div>
              
              <div class="detail-row">
                <div class="detail-key">Condition</div>
                <div class="detail-value">
                  <div class="code-block">
                    <pre><code>{selectedBreakpoint.condition}</code></pre>
                  </div>
                </div>
              </div>
              
              <div class="detail-row">
                <div class="detail-key">Actions</div>
                <div class="detail-value">
                  <div class="action-tags">
                    {#each selectedBreakpoint.actions as action}
                      <span class="action-tag">{action}</span>
                    {/each}
                  </div>
                </div>
              </div>
              
              <div class="detail-row">
                <div class="detail-key">Trigger Count</div>
                <div class="detail-value">
                  {selectedBreakpoint.triggerCount} times
                </div>
              </div>
              
              <div class="detail-row">
                <div class="detail-key">Last Triggered</div>
                <div class="detail-value">
                  {formatTime(selectedBreakpoint.lastTriggered)}
                  <span class="timestamp">
                    ({new Date(selectedBreakpoint.lastTriggered).toLocaleString()})
                  </span>
                </div>
              </div>
            </div>
            
            <div class="detail-section">
              <h3>Recent Triggers</h3>
              
              <div class="triggers-list">
                <div class="trigger-item">
                  <div class="trigger-time">
                    {formatTime(selectedBreakpoint.lastTriggered)}
                  </div>
                  <div class="trigger-details">
                    <div class="trigger-title">Condition matched in Order Service</div>
                    <div class="trigger-data">
                      <pre><code>span.duration = 87ms (> 50ms threshold)</code></pre>
                    </div>
                  </div>
                </div>
                
                <div class="trigger-item">
                  <div class="trigger-time">
                    {formatTime(selectedBreakpoint.lastTriggered + 1000 * 60 * 10)}
                  </div>
                  <div class="trigger-details">
                    <div class="trigger-title">Condition matched in Order Service</div>
                    <div class="trigger-data">
                      <pre><code>span.duration = 65ms (> 50ms threshold)</code></pre>
                    </div>
                  </div>
                </div>
                
                <div class="trigger-item">
                  <div class="trigger-time">
                    {formatTime(selectedBreakpoint.lastTriggered + 1000 * 60 * 25)}
                  </div>
                  <div class="trigger-details">
                    <div class="trigger-title">Condition matched in Order Service</div>
                    <div class="trigger-data">
                      <pre><code>span.duration = 72ms (> 50ms threshold)</code></pre>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      {:else}
        <div class="map-container card">
          <div class="card-header">
            <h2>Service Map</h2>
            <div class="card-description">
              Click on a service to filter breakpoints
            </div>
          </div>
          
          <div class="service-map-wrapper">
            <ServiceMap
              services={services}
              connections={connections}
              onServiceClick={handleServiceClick}
            />
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .breakpoints-page {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .breakpoints-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  h1 {
    font-size: 1.8rem;
    font-weight: 700;
    margin: 0;
  }
  
  .header-actions {
    display: flex;
    gap: 0.75rem;
  }
  
  .btn-primary {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background-color: var(--electric-teal);
    color: var(--deep-space);
    border: none;
    border-radius: 4px;
    padding: 0.6rem 1rem;
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .btn-primary:hover {
    filter: brightness(1.1);
  }
  
  .btn-secondary {
    background-color: transparent;
    color: var(--text-light);
    border: 1px solid var(--border-medium);
    border-radius: 4px;
    padding: 0.6rem 1rem;
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .btn-secondary:hover {
    background-color: var(--bg-tertiary);
    border-color: var(--border-active);
  }
  
  .breakpoints-filters {
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
  
  .breakpoints-content {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
    flex: 1;
    min-height: 0;
  }
  
  .breakpoints-list-container, 
  .breakpoint-detail-container,
  .map-container {
    background-color: var(--bg-secondary);
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    border: 1px solid var(--border-light);
    display: flex;
    flex-direction: column;
  }
  
  .breakpoint-detail-container,
  .map-container {
    min-height: 500px;
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
    display: flex;
    align-items: center;
  }
  
  .card-description {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.6);
  }
  
  .count-badge {
    background-color: var(--bg-tertiary);
    padding: 0.2rem 0.5rem;
    border-radius: 50px;
    font-size: 0.75rem;
    margin-left: 0.75rem;
    color: rgba(255, 255, 255, 0.7);
  }
  
  .card-actions {
    display: flex;
    gap: 0.5rem;
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
  
  .list-header {
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-light);
  }
  
  .breakpoints-list {
    padding: 0.5rem;
    overflow-y: auto;
    flex: 1;
  }
  
  .breakpoint-item {
    padding: 1rem;
    border-radius: 6px;
    margin-bottom: 0.5rem;
    cursor: pointer;
    border-left: 3px solid var(--plasma-pink);
    background-color: rgba(18, 16, 35, 0.8);
    transition: all 0.2s ease;
  }
  
  .breakpoint-item:hover {
    background-color: var(--bg-tertiary);
  }
  
  .breakpoint-item.selected {
    background-color: var(--bg-tertiary);
    border-left-color: var(--electric-teal);
  }
  
  .bp-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }
  
  .bp-name {
    font-weight: 600;
    font-size: 0.95rem;
  }
  
  .status-toggle {
    width: 36px;
    height: 20px;
    background-color: rgba(255, 255, 255, 0.2);
    border-radius: 10px;
    position: relative;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }
  
  .status-toggle.active {
    background-color: var(--electric-teal);
  }
  
  .toggle-slider {
    width: 16px;
    height: 16px;
    background-color: white;
    border-radius: 50%;
    position: absolute;
    top: 2px;
    left: 2px;
    transition: transform 0.2s ease;
  }
  
  .status-toggle.active .toggle-slider {
    transform: translateX(16px);
  }
  
  .status-toggle.large {
    width: 46px;
    height: 24px;
  }
  
  .status-toggle.large .toggle-slider {
    width: 20px;
    height: 20px;
    top: 2px;
  }
  
  .status-toggle.large.active .toggle-slider {
    transform: translateX(22px);
  }
  
  .bp-service {
    font-size: 0.8rem;
    background-color: var(--bg-tertiary);
    display: inline-block;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    margin-bottom: 0.75rem;
    color: rgba(255, 255, 255, 0.8);
  }
  
  .bp-condition {
    margin-bottom: 0.75rem;
    background-color: rgba(0, 0, 0, 0.2);
    padding: 0.5rem;
    border-radius: 4px;
    overflow-x: auto;
  }
  
  .bp-condition code {
    font-family: monospace;
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.9);
  }
  
  .bp-stats {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.6);
  }
  
  .bp-stat {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  
  .no-results {
    display: flex;
    justify-content: center;
    padding: 2rem 0;
    color: rgba(255, 255, 255, 0.5);
  }
  
  .service-map-wrapper {
    flex: 1;
    min-height: 0;
  }
  
  .detail-content {
    padding: 1.25rem;
    overflow-y: auto;
    flex: 1;
  }
  
  .detail-section {
    margin-bottom: 2rem;
  }
  
  .detail-section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 1rem 0;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--border-light);
  }
  
  .detail-row {
    display: flex;
    margin-bottom: 1.25rem;
  }
  
  .detail-key {
    flex: 0 0 120px;
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.9rem;
  }
  
  .detail-value {
    flex: 1;
  }
  
  .service-badge {
    display: inline-block;
    padding: 0.3rem 0.6rem;
    background-color: var(--bg-tertiary);
    border-radius: 4px;
    font-size: 0.85rem;
  }
  
  .code-block {
    background-color: rgba(0, 0, 0, 0.2);
    padding: 0.75rem;
    border-radius: 4px;
    overflow-x: auto;
  }
  
  .code-block pre {
    margin: 0;
  }
  
  .code-block code {
    font-family: monospace;
    font-size: 0.9rem;
    color: rgba(255, 255, 255, 0.9);
  }
  
  .action-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  
  .action-tag {
    display: inline-block;
    padding: 0.3rem 0.6rem;
    background-color: rgba(138, 43, 226, 0.2);
    color: var(--quantum-purple);
    border-radius: 4px;
    font-size: 0.8rem;
  }
  
  .timestamp {
    margin-left: 0.5rem;
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.6);
  }
  
  .triggers-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .trigger-item {
    display: flex;
    gap: 1rem;
    padding: 0.75rem;
    background-color: var(--bg-tertiary);
    border-radius: 6px;
  }
  
  .trigger-time {
    flex: 0 0 100px;
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.7);
  }
  
  .trigger-details {
    flex: 1;
  }
  
  .trigger-title {
    font-size: 0.9rem;
    margin-bottom: 0.5rem;
  }
  
  .trigger-data {
    font-size: 0.85rem;
    background-color: rgba(0, 0, 0, 0.2);
    padding: 0.5rem;
    border-radius: 4px;
  }
  
  .trigger-data pre {
    margin: 0;
  }
  
  .form-container {
    padding: 1.25rem;
    overflow-y: auto;
  }
  
  .form-group {
    margin-bottom: 1.5rem;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-size: 0.9rem;
    font-weight: 500;
  }
  
  .form-group input[type="text"],
  .form-group select,
  .form-group textarea {
    width: 100%;
    padding: 0.6rem 0.75rem;
    border-radius: 6px;
    border: 1px solid var(--border-light);
    background-color: var(--bg-tertiary);
    color: var(--text-light);
    font-size: 0.9rem;
  }
  
  .form-group input[type="text"]:focus,
  .form-group select:focus,
  .form-group textarea:focus {
    outline: none;
    border-color: var(--electric-teal);
    box-shadow: 0 0 0 1px var(--electric-teal);
  }
  
  .form-group textarea {
    resize: vertical;
    font-family: monospace;
  }
  
  .hint {
    margin-top: 0.5rem;
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.6);
  }
  
  .checkbox-group {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }
  
  .checkbox {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    cursor: pointer;
  }
  
  .checkbox input {
    width: 16px;
    height: 16px;
    accent-color: var(--electric-teal);
  }
  
  .form-actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
    margin-top: 2rem;
  }
</style>