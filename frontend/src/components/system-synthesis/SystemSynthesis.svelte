<script lang="ts">
  import { onMount } from 'svelte';
  import type { 
    RootCauseAnalysis, 
    Recommendation, 
    SimulationResults,
    SystemSynthesisProps
  } from './types';

  // Props
  export let rootCauseData: RootCauseAnalysis | null = null;
  export let showActions: boolean = true;

  // Internal state
  let activeTab: 'analysis' | 'recommendations' | 'history' = 'analysis';
  let isExpanded: boolean = false;
  let applyingFix: boolean = false;
  let fixApplied: boolean = false;
  let simulationRunning: boolean = false;
  let simulationResults: SimulationResults | null = null;

  // Sample data (replace with actual data in production)
  const rootCauseAnalysis: RootCauseAnalysis = {
    rootCause: "Database connection pool saturation in payments-service",
    impact: "High latency in checkout flow, affecting 23% of transactions",
    confidence: 87,
    triggeredBy: "Increase in traffic from marketing campaign at 14:32",
    details: {
      symptom: "Increased response time in payment processing API",
      affectedServices: ["payments-service", "checkout-service", "order-service"],
      evidencePoints: [
        "Connection pool at 100% utilization (50/50 connections)",
        "32 queued connection requests",
        "Average connection wait time increased from 120ms to 870ms",
        "Database CPU utilization stable at 45%",
        "No memory pressure detected"
      ],
      timeline: [
        { time: "14:32:15", event: "Marketing campaign email sent to 150,000 users" },
        { time: "14:34:22", event: "Traffic increased by 245% over baseline" },
        { time: "14:35:47", event: "First connection pool queue detected" },
        { time: "14:36:10", event: "First transaction timeout reported" }
      ]
    }
  };

  const recommendations: Recommendation[] = [
    {
      id: "rec-1",
      type: "immediate",
      action: "Increase connection pool size in payments-service from 50 to 150",
      impact: "High",
      confidence: 95,
      difficulty: "Low",
      implementation: {
        type: "config",
        file: "/config/database.yml",
        changes: [
          { before: "pool_size: 50", after: "pool_size: 150" }
        ]
      }
    },
    {
      id: "rec-2",
      type: "short-term",
      action: "Implement connection pooling cache in payment processor client",
      impact: "Medium",
      confidence: 82,
      difficulty: "Medium",
      implementation: {
        type: "code",
        file: "/src/services/paymentProcessor.js",
        description: "Add connection pooling wrapper with caching"
      }
    },
    {
      id: "rec-3",
      type: "long-term",
      action: "Migrate payment processing to async workflow to reduce DB pressure",
      impact: "High",
      confidence: 78,
      difficulty: "High",
      implementation: {
        type: "architecture",
        description: "Implement message queue and worker service for payment processing"
      }
    }
  ];

  const simulationData: SimulationResults = {
    originalLatency: 870,
    predictedLatency: 210,
    improvementPercent: 76,
    throughputIncrease: "205%",
    potentialIssues: ["Might increase database CPU utilization", "Monitor for connection leaks"]
  };

  // Use provided data or fallback to sample data
  $: activeRootCauseData = rootCauseData || rootCauseAnalysis;

  // Handle tab switching
  function setTab(tab: 'analysis' | 'recommendations' | 'history') {
    activeTab = tab;
  }

  // Handle fix application
  function applyFix() {
    applyingFix = true;
    // Simulate API call to apply fix
    setTimeout(() => {
      applyingFix = false;
      fixApplied = true;
    }, 2000);
  }

  // Handle fix simulation
  function simulateFix() {
    simulationRunning = true;
    // Simulate API call to run simulation
    setTimeout(() => {
      simulationRunning = false;
      simulationResults = simulationData;
    }, 2500);
  }

  // Reset state
  function resetSimulation() {
    simulationResults = null;
  }
</script>

<div class="system-synthesis {isExpanded ? 'expanded' : ''}">
  <div class="synthesis-header">
    <div class="header-left">
      <div class="synthesis-badge">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2v2m0 16v2M4 12H2m20 0h-2m-9 7l-3-3m3-11l-3 3m12 0l-3-3m0 14l3-3" />
          <circle cx="12" cy="12" r="4" />
        </svg>
        <span>System Synthesis</span>
      </div>
      
      <div class="confidence-indicator">
        <div class="confidence-bar">
          <div 
            class="confidence-value" 
            style="width: {activeRootCauseData.confidence}%;"
          ></div>
        </div>
        <span class="confidence-label">{activeRootCauseData.confidence}% Confidence</span>
      </div>
    </div>
    
    <div class="header-actions">
      <button 
        class="toggle-button"
        on:click={() => isExpanded = !isExpanded}
      >
        {isExpanded ? 'Collapse' : 'Expand'}
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          {#if isExpanded}
            <polyline points="18 15 12 9 6 15"></polyline>
          {:else}
            <polyline points="6 9 12 15 18 9"></polyline>
          {/if}
        </svg>
      </button>
    </div>
  </div>
  
  <div class="synthesis-content">
    <div class="synthesis-tabs">
      <button 
        class="tab-button {activeTab === 'analysis' ? 'active' : ''}" 
        on:click={() => setTab('analysis')}
      >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12" y2="16" />
        </svg>
        Issue Analysis
      </button>
      
      <button 
        class="tab-button {activeTab === 'recommendations' ? 'active' : ''}" 
        on:click={() => setTab('recommendations')}
      >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
          <polyline points="14 2 14 8 20 8" />
          <line x1="16" y1="13" x2="8" y2="13" />
          <line x1="16" y1="17" x2="8" y2="17" />
          <polyline points="10 9 9 9 8 9" />
        </svg>
        Recommendations
      </button>
      
      <button 
        class="tab-button {activeTab === 'history' ? 'active' : ''}" 
        on:click={() => setTab('history')}
      >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" />
          <polyline points="12 6 12 12 16 14" />
        </svg>
        History
      </button>
    </div>
    
    <div class="synthesis-tab-content">
      {#if activeTab === 'analysis'}
        <div class="analysis-tab">
          <div class="analysis-summary">
            <div class="summary-item root-cause">
              <div class="summary-label">Root Cause</div>
              <div class="summary-value">{activeRootCauseData.rootCause}</div>
            </div>
            
            <div class="summary-item impact">
              <div class="summary-label">Impact</div>
              <div class="summary-value">{activeRootCauseData.impact}</div>
            </div>
            
            <div class="summary-item trigger">
              <div class="summary-label">Triggered By</div>
              <div class="summary-value">{activeRootCauseData.triggeredBy}</div>
            </div>
          </div>
          
          {#if isExpanded}
            <div class="analysis-details">
              <div class="details-section">
                <h4 class="section-title">Evidence</h4>
                <ul class="evidence-list">
                  {#each activeRootCauseData.details.evidencePoints as evidence}
                    <li class="evidence-item">{evidence}</li>
                  {/each}
                </ul>
              </div>
              
              <div class="details-section">
                <h4 class="section-title">Affected Services</h4>
                <div class="affected-services">
                  {#each activeRootCauseData.details.affectedServices as service}
                    <div class="service-tag">{service}</div>
                  {/each}
                </div>
              </div>
              
              <div class="details-section">
                <h4 class="section-title">Timeline</h4>
                <div class="timeline">
                  {#each activeRootCauseData.details.timeline as event}
                    <div class="timeline-event">
                      <div class="event-time">{event.time}</div>
                      <div class="event-description">{event.event}</div>
                    </div>
                  {/each}
                </div>
              </div>
            </div>
          {/if}
        </div>
      {:else if activeTab === 'recommendations'}
        <div class="recommendations-tab">
          <div class="recommendations-list">
            {#each recommendations as recommendation, index}
              <div class="recommendation-card {recommendation.type}">
                <div class="recommendation-header">
                  <div class="recommendation-type">{recommendation.type}</div>
                  <div class="recommendation-metrics">
                    <div class="metric" title="Impact">
                      <span class="metric-label">Impact:</span>
                      <span class="metric-value impact-{recommendation.impact.toLowerCase()}">{recommendation.impact}</span>
                    </div>
                    <div class="metric" title="Confidence">
                      <span class="metric-label">Confidence:</span>
                      <span class="metric-value">{recommendation.confidence}%</span>
                    </div>
                    <div class="metric" title="Difficulty">
                      <span class="metric-label">Difficulty:</span>
                      <span class="metric-value difficulty-{recommendation.difficulty.toLowerCase()}">{recommendation.difficulty}</span>
                    </div>
                  </div>
                </div>
                
                <div class="recommendation-body">
                  <div class="recommendation-action">{recommendation.action}</div>
                  
                  {#if isExpanded}
                    <div class="recommendation-details">
                      <div class="implementation-details">
                        <h5>Implementation</h5>
                        {#if recommendation.implementation.type === 'config'}
                          <div class="config-changes">
                            <div class="file-path">{recommendation.implementation.file}</div>
                            {#each recommendation.implementation.changes as change}
                              <div class="change">
                                <div class="before">{change.before}</div>
                                <div class="arrow">→</div>
                                <div class="after">{change.after}</div>
                              </div>
                            {/each}
                          </div>
                        {:else}
                          <div class="implementation-description">
                            {#if recommendation.implementation.file}
                              <div class="file-path">{recommendation.implementation.file}</div>
                            {/if}
                            <div class="description-text">{recommendation.implementation.description}</div>
                          </div>
                        {/if}
                      </div>
                    </div>
                  {/if}
                </div>
                
                {#if showActions && index === 0}
                  <div class="recommendation-actions">
                    {#if fixApplied}
                      <div class="fix-applied">
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
                          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                          <polyline points="22 4 12 14.01 9 11.01" />
                        </svg>
                        Fix Applied
                      </div>
                    {:else}
                      <button 
                        class="apply-button" 
                        on:click={applyFix}
                        disabled={applyingFix}
                      >
                        {#if applyingFix}
                          <div class="spinner small"></div>
                          Applying...
                        {:else}
                          Apply Fix
                        {/if}
                      </button>
                    {/if}
                    
                    <button 
                      class="simulate-button" 
                      on:click={simulateFix}
                      disabled={simulationRunning}
                    >
                      {#if simulationRunning}
                        <div class="spinner small"></div>
                        Simulating...
                      {:else}
                        Simulate
                      {/if}
                    </button>
                  </div>
                  
                  {#if simulationResults}
                    <div class="simulation-results">
                      <div class="simulation-header">
                        <h5>Simulation Results</h5>
                        <button class="close-button" on:click={resetSimulation}>
                          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                            <line x1="18" y1="6" x2="6" y2="18" />
                            <line x1="6" y1="6" x2="18" y2="18" />
                          </svg>
                        </button>
                      </div>
                      
                      <div class="simulation-metrics">
                        <div class="simulation-metric">
                          <div class="metric-value improvement">{simulationResults.improvementPercent}%</div>
                          <div class="metric-label">Latency Improvement</div>
                        </div>
                        
                        <div class="simulation-comparison">
                          <div class="comparison-item">
                            <div class="comparison-label">Original Latency</div>
                            <div class="comparison-value original">{simulationResults.originalLatency}ms</div>
                          </div>
                          <div class="comparison-arrow">→</div>
                          <div class="comparison-item">
                            <div class="comparison-label">Predicted Latency</div>
                            <div class="comparison-value predicted">{simulationResults.predictedLatency}ms</div>
                          </div>
                        </div>
                        
                        <div class="simulation-metric">
                          <div class="metric-value">{simulationResults.throughputIncrease}</div>
                          <div class="metric-label">Throughput Increase</div>
                        </div>
                      </div>
                      
                      {#if simulationResults.potentialIssues.length > 0}
                        <div class="potential-issues">
                          <h6>Potential Issues to Monitor</h6>
                          <ul>
                            {#each simulationResults.potentialIssues as issue}
                              <li>{issue}</li>
                            {/each}
                          </ul>
                        </div>
                      {/if}
                    </div>
                  {/if}
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {:else if activeTab === 'history'}
        <div class="history-tab">
          <div class="empty-state">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="64" height="64" fill="none" stroke="currentColor" stroke-width="1">
              <circle cx="12" cy="12" r="10" />
              <polyline points="12 6 12 12 16 14" />
            </svg>
            <p>No historical synthesis data available yet</p>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  /* Edgy high-performance styling */
  .system-synthesis {
    background-color: var(--bg-darker);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.05);
    overflow: hidden;
    transition: box-shadow var(--transition-fast);
    margin-bottom: 1.5rem;
    box-shadow: var(--shadow-sm);
    position: relative;
  }
  
  .system-synthesis::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(90deg, var(--primary), var(--accent));
    z-index: 5;
  }
  
  .system-synthesis.expanded {
    box-shadow: var(--shadow-md);
  }
  
  .system-synthesis.expanded::before {
    background: linear-gradient(90deg, var(--primary), var(--highlight), var(--accent));
  }
  
  .synthesis-header {
    padding: 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    background-color: var(--bg-dark);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    position: relative;
  }
  
  .synthesis-header::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 20%;
    right: 20%;
    height: 1px;
    background: linear-gradient(to right, transparent, var(--primary), transparent);
    opacity: 0.3;
  }
  
  .header-left {
    display: flex;
    align-items: center;
    gap: 1.5rem;
  }
  
  .synthesis-badge {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background-color: rgba(0, 255, 170, 0.08);
    color: var(--primary);
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    font-weight: 700;
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    position: relative;
    overflow: hidden;
  }
  
  .synthesis-badge::after {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 50%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(0, 255, 170, 0.2), transparent);
    animation: badge-shine 3s infinite;
  }
  
  @keyframes badge-shine {
    0% { left: -100%; }
    20% { left: 100%; }
    100% { left: 100%; }
  }
  
  .confidence-indicator {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .confidence-bar {
    width: 120px;
    height: 4px;
    background-color: rgba(255, 255, 255, 0.05);
    border-radius: 0;
    overflow: hidden;
    position: relative;
  }
  
  .confidence-value {
    height: 100%;
    background: linear-gradient(90deg, var(--primary), var(--accent));
    border-radius: 0;
    position: relative;
  }
  
  .confidence-value::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
    animation: confidence-pulse 2s infinite;
  }
  
  @keyframes confidence-pulse {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(100%); }
  }
  
  .confidence-label {
    font-size: 0.8rem;
    color: var(--text-medium);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .header-actions {
    display: flex;
    gap: 0.75rem;
  }
  
  .toggle-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background-color: transparent;
    border: 1px solid var(--primary);
    border-radius: var(--radius-sm);
    color: var(--primary);
    padding: 6px 12px;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    transition: all var(--transition-fast);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    position: relative;
    overflow: hidden;
  }
  
  .toggle-button::after {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(0, 255, 170, 0.2), transparent);
    transition: var(--transition-normal);
  }
  
  .toggle-button:hover {
    background-color: rgba(0, 255, 170, 0.1);
    box-shadow: var(--glow-primary);
  }
  
  .toggle-button:hover::after {
    left: 100%;
  }
  
  .synthesis-content {
    overflow: hidden;
  }
  
  .synthesis-tabs {
    display: flex;
    padding: 0 16px;
    background-color: var(--bg-medium);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    position: relative;
  }
  
  .synthesis-tabs::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 1px;
    background-color: rgba(0, 255, 170, 0.1);
  }
  
  .tab-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 12px 16px;
    background: none;
    border: none;
    color: var(--text-medium);
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: all var(--transition-fast);
    border-bottom: 2px solid transparent;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    position: relative;
  }
  
  .tab-button:hover {
    color: var(--text-bright);
  }
  
  .tab-button.active {
    color: var(--primary);
    border-bottom-color: var(--primary);
  }
  
  .tab-button.active::before {
    content: '';
    position: absolute;
    bottom: -2px;
    left: 0;
    width: 100%;
    height: 2px;
    background: linear-gradient(90deg, var(--primary), var(--accent));
    z-index: 1;
  }
  
  .synthesis-tab-content {
    padding: 16px;
  }
  
  .analysis-summary {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 16px;
    margin-bottom: 24px;
  }
  
  .summary-item {
    background-color: var(--bg-dark);
    border-radius: var(--radius-md);
    padding: 16px;
    border-left: 3px solid transparent;
  }
  
  .summary-item.root-cause {
    border-left-color: var(--error);
  }
  
  .summary-item.impact {
    border-left-color: var(--warning);
  }
  
  .summary-item.trigger {
    border-left-color: var(--primary);
  }
  
  .summary-label {
    font-size: 0.8rem;
    color: var(--text-medium);
    margin-bottom: 8px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 600;
  }
  
  .summary-value {
    font-size: 0.95rem;
    line-height: 1.4;
    color: var(--text-bright);
  }
  
  .analysis-details {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
  }
  
  .details-section:nth-child(3) {
    grid-column: 1 / -1;
  }
  
  .section-title {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 12px 0;
    color: var(--text-brighter);
  }
  
  .evidence-list {
    padding-left: 24px;
    margin: 0;
  }
  
  .evidence-item {
    margin-bottom: 8px;
    color: var(--text-medium);
  }
  
  .affected-services {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  
  .service-tag {
    background-color: rgba(86, 97, 242, 0.1);
    border: 1px solid rgba(86, 97, 242, 0.2);
    color: var(--primary-light);
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 500;
  }
  
  .timeline {
    display: flex;
    flex-direction: column;
    position: relative;
    margin-left: 16px;
  }
  
  .timeline::before {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: -16px;
    width: 2px;
    background-color: rgba(255, 255, 255, 0.1);
  }
  
  .timeline-event {
    position: relative;
    padding-left: 8px;
    padding-bottom: 16px;
  }
  
  .timeline-event:last-child {
    padding-bottom: 0;
  }
  
  .timeline-event::before {
    content: '';
    position: absolute;
    left: -20px;
    top: 8px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background-color: var(--primary);
    z-index: 1;
  }
  
  .event-time {
    font-weight: 600;
    font-size: 0.9rem;
    margin-bottom: 4px;
    color: var(--text-bright);
  }
  
  .event-description {
    font-size: 0.9rem;
    color: var(--text-medium);
  }
  
  .recommendations-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  
  .recommendation-card {
    background-color: var(--bg-dark);
    border-radius: var(--radius-md);
    border-left: 4px solid transparent;
    overflow: hidden;
  }
  
  .recommendation-card.immediate {
    border-left-color: var(--error);
  }
  
  .recommendation-card.short-term {
    border-left-color: var(--warning);
  }
  
  .recommendation-card.long-term {
    border-left-color: var(--secondary);
  }
  
  .recommendation-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    background-color: var(--bg-medium);
  }
  
  .recommendation-type {
    text-transform: uppercase;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.5px;
  }
  
  .recommendation-card.immediate .recommendation-type {
    color: var(--error);
  }
  
  .recommendation-card.short-term .recommendation-type {
    color: var(--warning);
  }
  
  .recommendation-card.long-term .recommendation-type {
    color: var(--secondary);
  }
  
  .recommendation-metrics {
    display: flex;
    gap: 16px;
  }
  
  .metric {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.8rem;
  }
  
  .metric-label {
    color: var(--text-medium);
  }
  
  .metric-value {
    font-weight: 600;
  }
  
  .impact-high {
    color: var(--success);
  }
  
  .impact-medium {
    color: var(--warning);
  }
  
  .impact-low {
    color: var(--text-medium);
  }
  
  .difficulty-low {
    color: var(--success);
  }
  
  .difficulty-medium {
    color: var(--warning);
  }
  
  .difficulty-high {
    color: var(--error);
  }
  
  .recommendation-body {
    padding: 16px;
  }
  
  .recommendation-action {
    font-size: 1rem;
    font-weight: 500;
    margin-bottom: 12px;
    color: var(--text-bright);
  }
  
  .recommendation-details {
    background-color: var(--bg-medium);
    border-radius: var(--radius-sm);
    padding: 16px;
    margin-top: 12px;
  }
  
  .implementation-details h5 {
    font-size: 0.9rem;
    font-weight: 600;
    margin: 0 0 12px 0;
    color: var(--text-brighter);
  }
  
  .file-path {
    font-family: monospace;
    font-size: 0.85rem;
    color: var(--secondary);
    background-color: rgba(0, 191, 167, 0.1);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    display: inline-block;
    margin-bottom: 8px;
  }
  
  .config-changes {
    font-family: monospace;
    font-size: 0.85rem;
  }
  
  .change {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }
  
  .before {
    color: var(--error);
    text-decoration: line-through;
  }
  
  .after {
    color: var(--success);
  }
  
  .arrow {
    color: var(--text-medium);
  }
  
  .implementation-description {
    font-size: 0.9rem;
    color: var(--text-medium);
  }
  
  .recommendation-actions {
    display: flex;
    gap: 12px;
    padding: 0 16px 16px 16px;
  }
  
  .apply-button, .simulate-button {
    padding: 8px 16px;
    border-radius: var(--radius-sm);
    font-weight: 600;
    font-size: 0.8rem;
    cursor: pointer;
    transition: all var(--transition-fast);
    display: flex;
    align-items: center;
    gap: 8px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    position: relative;
    overflow: hidden;
  }
  
  .apply-button::after, .simulate-button::after {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.2), transparent);
    transition: var(--transition-normal);
  }
  
  .apply-button:hover::after, .simulate-button:hover::after {
    left: 100%;
  }
  
  .apply-button {
    background-color: var(--primary);
    color: var(--bg-darkest);
    border: none;
    flex: 1;
    box-shadow: 0 0 0 1px rgba(0, 255, 170, 0.3);
  }
  
  .apply-button:hover:not([disabled]) {
    background-color: var(--primary-light);
    box-shadow: var(--glow-primary);
  }
  
  .simulate-button {
    background-color: transparent;
    border: 1px solid var(--accent);
    color: var(--accent);
  }
  
  .simulate-button:hover:not([disabled]) {
    background-color: rgba(51, 153, 255, 0.1);
    box-shadow: var(--glow-accent);
  }
  
  .apply-button:disabled, .simulate-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .fix-applied {
    display: flex;
    align-items: center;
    gap: 8px;
    background-color: rgba(0, 255, 153, 0.1);
    color: var(--success);
    padding: 8px 16px;
    border-radius: var(--radius-sm);
    font-weight: 600;
    font-size: 0.8rem;
    flex: 1;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border: 1px solid var(--success);
    position: relative;
    overflow: hidden;
  }
  
  .fix-applied::after {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 50%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(0, 255, 153, 0.2), transparent);
    animation: success-shine 3s infinite;
  }
  
  @keyframes success-shine {
    0% { left: -50%; }
    100% { left: 150%; }
  }
  
  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(0, 255, 170, 0.2);
    border-radius: 2px;
    border-top-color: var(--primary);
    animation: edgy-spin 0.6s linear infinite;
  }
  
  .spinner.small {
    width: 12px;
    height: 12px;
    border-width: 2px;
  }
  
  @keyframes edgy-spin {
    to { transform: rotate(360deg); }
  }
  
  .simulation-results {
    margin: 0 16px 16px 16px;
    background-color: var(--bg-dark);
    border-radius: var(--radius-sm);
    padding: 16px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    position: relative;
    overflow: hidden;
  }
  
  .simulation-results::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(90deg, var(--accent), var(--primary));
    z-index: 1;
  }
  
  .simulation-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }
  
  .simulation-header h5 {
    font-size: 0.95rem;
    font-weight: 600;
    margin: 0;
    color: var(--text-brighter);
  }
  
  .close-button {
    background: none;
    border: none;
    color: var(--text-medium);
    cursor: pointer;
    padding: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
  }
  
  .close-button:hover {
    background-color: rgba(255, 255, 255, 0.1);
    color: var(--text-bright);
  }
  
  .simulation-metrics {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 16px;
    margin-bottom: 16px;
  }
  
  .simulation-metric {
    text-align: center;
  }
  
  .simulation-metric .metric-value {
    font-size: 1.75rem;
    font-weight: 800;
    margin-bottom: 4px;
    background: linear-gradient(90deg, var(--primary), var(--accent));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    display: inline-block;
    letter-spacing: -0.5px;
  }
  
  .simulation-metric .metric-value.improvement {
    color: var(--success);
    background: linear-gradient(90deg, var(--success), var(--primary));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  .simulation-metric .metric-label {
    font-size: 0.7rem;
    color: var(--text-medium);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 600;
  }
  
  .simulation-comparison {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
  }
  
  .comparison-item {
    text-align: center;
  }
  
  .comparison-label {
    font-size: 0.8rem;
    color: var(--text-medium);
    margin-bottom: 4px;
  }
  
  .comparison-value {
    font-weight: 600;
  }
  
  .comparison-value.original {
    color: var(--error);
    text-decoration: line-through;
    opacity: 0.8;
  }
  
  .comparison-value.predicted {
    color: var(--success);
    position: relative;
  }
  
  .comparison-value.predicted::after {
    content: '';
    position: absolute;
    bottom: -2px;
    left: 0;
    width: 100%;
    height: 2px;
    background-color: var(--success);
    opacity: 0.5;
  }
  
  .comparison-arrow {
    color: var(--text-medium);
    font-size: 1.25rem;
    position: relative;
    width: 24px;
    text-align: center;
  }
  
  .comparison-arrow::before {
    content: '';
    position: absolute;
    top: 50%;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, var(--error), var(--success));
    z-index: 0;
  }
  
  .potential-issues {
    background-color: rgba(255, 170, 0, 0.08);
    border-radius: var(--radius-sm);
    padding: 12px 16px;
    border-left: 2px solid var(--warning);
    position: relative;
  }
  
  .potential-issues::before {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: -2px;
    width: 2px;
    background-color: var(--warning);
  }
  
  .potential-issues h6 {
    font-size: 0.75rem;
    font-weight: 700;
    margin: 0 0 8px 0;
    color: var(--warning);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    display: flex;
    align-items: center;
  }
  
  .potential-issues h6::before {
    content: '⚠️';
    margin-right: 6px;
    font-size: 0.9rem;
  }
  
  .potential-issues ul {
    margin: 0;
    padding-left: 24px;
  }
  
  .potential-issues li {
    font-size: 0.85rem;
    color: var(--text-medium);
    margin-bottom: 4px;
  }
  
  .potential-issues li:last-child {
    margin-bottom: 0;
  }
  
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 16px;
    color: var(--text-medium);
  }
  
  .empty-state svg {
    margin-bottom: 16px;
    opacity: 0.5;
  }
  
  .empty-state p {
    font-size: 0.95rem;
    margin: 0;
  }
</style>