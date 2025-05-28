<script lang="ts">
  import { onMount } from 'svelte';
  import SystemSynthesis from '../../components/system-synthesis';
  import type { RootCauseAnalysis } from '../../components/system-synthesis/types';

  // Example root cause data
  const sampleRootCauseData: RootCauseAnalysis = {
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

  let showSystemActions = true;
</script>

<div class="synthesis-page">
  <header class="page-header">
    <h1>System Synthesis</h1>
    <p class="page-description">
      Automated root cause analysis and intelligent system optimization
    </p>
  </header>

  <div class="synthesis-container">
    <SystemSynthesis 
      rootCauseData={sampleRootCauseData} 
      showActions={showSystemActions} 
    />
  </div>

  <div class="page-sections">
    <section class="section">
      <h2>Recent System Incidents</h2>
      <div class="incident-list">
        <div class="incident-card">
          <div class="incident-header">
            <span class="incident-time">Today, 14:32</span>
            <span class="incident-status error">Critical</span>
          </div>
          <h3 class="incident-title">Checkout Flow Latency Spike</h3>
          <p class="incident-description">
            Connection pool saturation caused by marketing campaign traffic
          </p>
          <div class="incident-metrics">
            <div class="metric">
              <span class="metric-label">Duration</span>
              <span class="metric-value">24 minutes</span>
            </div>
            <div class="metric">
              <span class="metric-label">Affected Users</span>
              <span class="metric-value">~2,100</span>
            </div>
            <div class="metric">
              <span class="metric-label">Revenue Impact</span>
              <span class="metric-value">$12,500</span>
            </div>
          </div>
          <div class="incident-actions">
            <button class="view-button">View Details</button>
          </div>
        </div>

        <div class="incident-card">
          <div class="incident-header">
            <span class="incident-time">Yesterday, 08:15</span>
            <span class="incident-status warning">Warning</span>
          </div>
          <h3 class="incident-title">Auth Service Intermittent Failures</h3>
          <p class="incident-description">
            Intermittent token validation errors caused by Redis cache eviction
          </p>
          <div class="incident-metrics">
            <div class="metric">
              <span class="metric-label">Duration</span>
              <span class="metric-value">47 minutes</span>
            </div>
            <div class="metric">
              <span class="metric-label">Affected Users</span>
              <span class="metric-value">~850</span>
            </div>
            <div class="metric">
              <span class="metric-label">Revenue Impact</span>
              <span class="metric-value">$3,200</span>
            </div>
          </div>
          <div class="incident-actions">
            <button class="view-button">View Details</button>
          </div>
        </div>
      </div>
    </section>

    <section class="section">
      <h2>System Health Insights</h2>
      <div class="insights-container">
        <div class="insight-card">
          <div class="insight-icon">📈</div>
          <div class="insight-content">
            <h3 class="insight-title">Traffic Pattern Change</h3>
            <p class="insight-description">
              User traffic patterns have shifted 22% toward evening hours (6-9pm) over the last week
            </p>
            <div class="insight-recommendation">
              <strong>Recommendation:</strong> Consider adjusting auto-scaling rules to increase capacity during evening hours
            </div>
          </div>
        </div>

        <div class="insight-card">
          <div class="insight-icon">⚠️</div>
          <div class="insight-content">
            <h3 class="insight-title">Potential Resource Constraint</h3>
            <p class="insight-description">
              Order service approaching memory limits during peak periods (85% utilization)
            </p>
            <div class="insight-recommendation">
              <strong>Recommendation:</strong> Increase memory allocation by 50% or implement pagination in order history API
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</div>

<style>
  /* SigNoz-inspired styling */
  .synthesis-page {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .page-header {
    margin-bottom: 16px;
  }

  .page-header h1 {
    font-size: 1.6rem;
    font-weight: 700;
    margin: 0 0 8px 0;
    color: var(--text-brightest);
  }

  .page-description {
    color: var(--text-medium);
    font-size: 0.95rem;
    margin: 0;
  }

  .synthesis-container {
    margin-bottom: 24px;
  }

  .page-sections {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section h2 {
    font-size: 1.3rem;
    font-weight: 600;
    margin: 0 0 8px 0;
    color: var(--text-brighter);
    position: relative;
    padding-left: 16px;
  }

  .section h2::before {
    content: '';
    position: absolute;
    left: 0;
    top: 4px;
    bottom: 4px;
    width: 4px;
    background-color: var(--primary);
    border-radius: 2px;
  }

  .incident-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
    gap: 16px;
  }

  .incident-card {
    background-color: var(--bg-darker);
    border-radius: var(--radius-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: var(--shadow-sm);
    transition: all 0.2s ease;
  }

  .incident-card:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
    border-color: rgba(86, 97, 242, 0.2);
  }

  .incident-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .incident-time {
    font-size: 0.85rem;
    color: var(--text-dim);
  }

  .incident-status {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .incident-status.error {
    background-color: rgba(245, 85, 74, 0.15);
    color: var(--error);
  }

  .incident-status.warning {
    background-color: rgba(255, 179, 0, 0.15);
    color: var(--warning);
  }

  .incident-title {
    font-size: 1.05rem;
    font-weight: 600;
    margin: 0;
    color: var(--text-bright);
  }

  .incident-description {
    color: var(--text-medium);
    font-size: 0.9rem;
    margin: 0;
    line-height: 1.5;
  }

  .incident-metrics {
    display: flex;
    gap: 16px;
    margin-top: 8px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    padding-top: 12px;
  }

  .metric {
    display: flex;
    flex-direction: column;
  }

  .metric-label {
    font-size: 0.75rem;
    color: var(--text-dim);
    margin-bottom: 4px;
  }

  .metric-value {
    font-weight: 600;
    color: var(--text-bright);
  }

  .incident-actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 8px;
  }

  .view-button {
    background-color: var(--bg-medium);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--text-medium);
    padding: 6px 12px;
    border-radius: var(--radius-md);
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.2s ease;
    font-weight: 500;
  }

  .view-button:hover {
    background-color: rgba(86, 97, 242, 0.1);
    color: var(--primary-light);
    border-color: var(--primary);
  }

  .insights-container {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
    gap: 16px;
  }

  .insight-card {
    background-color: var(--bg-darker);
    border-radius: var(--radius-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
    padding: 16px;
    display: flex;
    gap: 16px;
    box-shadow: var(--shadow-sm);
    transition: all 0.2s ease;
  }

  .insight-card:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
    border-color: rgba(86, 97, 242, 0.2);
  }

  .insight-icon {
    font-size: 1.3rem;
    min-width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: rgba(86, 97, 242, 0.1);
    border-radius: var(--radius-sm);
    color: var(--primary-light);
  }

  .insight-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .insight-title {
    font-size: 1.05rem;
    font-weight: 600;
    margin: 0;
    color: var(--text-bright);
  }

  .insight-description {
    color: var(--text-medium);
    font-size: 0.9rem;
    margin: 0;
    line-height: 1.5;
  }

  .insight-recommendation {
    margin-top: 8px;
    font-size: 0.85rem;
    color: var(--text-bright);
    background-color: rgba(0, 191, 167, 0.1);
    padding: 12px;
    border-radius: var(--radius-md);
    border-left: 3px solid var(--secondary);
    line-height: 1.5;
  }

  .insight-recommendation strong {
    color: var(--secondary);
    font-weight: 600;
  }
</style>