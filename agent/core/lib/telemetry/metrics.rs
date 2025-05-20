// agent/core/lib/telemetry/metrics.rs

use anyhow::Result;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

// Type for storing metric values
#[derive(Debug, Clone, Serialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Timer(Duration),
    Histogram(Vec<f64>),
    Set(Vec<String>),
}

// Statistical aggregation functions for histograms
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum AggregationType {
    Sum,
    Count,
    Min,
    Max,
    Average,
    Median,
    P90,
    P95,
    P99,
}

// Core metrics collector
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    metrics: Arc<Mutex<HashMap<String, MetricValue>>>,
    timers: Arc<Mutex<HashMap<String, Instant>>>,
    dimensions: HashMap<String, String>,
}

impl MetricsCollector {
    // Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            timers: Arc::new(Mutex::new(HashMap::new())),
            dimensions: HashMap::new(),
        }
    }

    // Add a dimension to all metrics
    pub fn with_dimension(mut self, key: &str, value: &str) -> Self {
        self.dimensions.insert(key.to_string(), value.to_string());
        self
    }

    // Add multiple dimensions
    pub fn with_dimensions(mut self, dimensions: HashMap<String, String>) -> Self {
        self.dimensions.extend(dimensions);
        self
    }

    // Increment a counter
    pub async fn increment_counter(&self, name: &str, value: u64) -> Result<()> {
        let mut metrics = self.metrics.lock().await;

        match metrics.get_mut(name) {
            Some(MetricValue::Counter(counter)) => {
                *counter += value;
            }
            Some(_) => {
                warn!("Metric {} exists but is not a counter", name);
                return Ok(());
            }
            None => {
                metrics.insert(name.to_string(), MetricValue::Counter(value));
            }
        }

        Ok(())
    }

    // Set a gauge value
    pub async fn set_gauge(&self, name: &str, value: f64) -> Result<()> {
        let mut metrics = self.metrics.lock().await;
        metrics.insert(name.to_string(), MetricValue::Gauge(value));
        Ok(())
    }

    // Start a timer
    pub async fn start_timer(&self, name: &str) -> Result<()> {
        let mut timers = self.timers.lock().await;
        timers.insert(name.to_string(), Instant::now());
        Ok(())
    }

    // Stop a timer and record the duration
    pub async fn stop_timer(&self, name: &str) -> Result<Duration> {
        let duration = {
            let mut timers = self.timers.lock().await;
            let start_time = timers
                .remove(name)
                .ok_or_else(|| anyhow::anyhow!("Timer {} not found", name))?;
            start_time.elapsed()
        };

        let mut metrics = self.metrics.lock().await;
        metrics.insert(name.to_string(), MetricValue::Timer(duration));

        Ok(duration)
    }

    // Record a value in a histogram
    pub async fn record_histogram_value(&self, name: &str, value: f64) -> Result<()> {
        let mut metrics = self.metrics.lock().await;

        match metrics.get_mut(name) {
            Some(MetricValue::Histogram(values)) => {
                values.push(value);
            }
            Some(_) => {
                warn!("Metric {} exists but is not a histogram", name);
                return Ok(());
            }
            None => {
                metrics.insert(name.to_string(), MetricValue::Histogram(vec![value]));
            }
        }

        Ok(())
    }

    // Add a value to a set
    pub async fn add_to_set(&self, name: &str, value: &str) -> Result<()> {
        let mut metrics = self.metrics.lock().await;

        match metrics.get_mut(name) {
            Some(MetricValue::Set(values)) => {
                if !values.contains(&value.to_string()) {
                    values.push(value.to_string());
                }
            }
            Some(_) => {
                warn!("Metric {} exists but is not a set", name);
                return Ok(());
            }
            None => {
                metrics.insert(name.to_string(), MetricValue::Set(vec![value.to_string()]));
            }
        }

        Ok(())
    }

    // Get all metrics as JSON
    pub async fn get_metrics_json(&self) -> Value {
        let metrics = self.metrics.lock().await;
        let mut result = json!({});

        for (name, value) in metrics.iter() {
            match value {
                MetricValue::Counter(count) => {
                    result[name] = json!(*count);
                }
                MetricValue::Gauge(gauge) => {
                    result[name] = json!(*gauge);
                }
                MetricValue::Timer(duration) => {
                    result[name] = json!(duration.as_millis());
                }
                MetricValue::Histogram(values) => {
                    // Include raw values and basic statistics
                    result[name] = json!({
                        "values": values,
                        "count": values.len(),
                        "sum": values.iter().sum::<f64>(),
                        "min": values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                        "max": values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
                        "avg": if values.is_empty() { 0.0 } else { values.iter().sum::<f64>() / values.len() as f64 },
                    });
                }
                MetricValue::Set(values) => {
                    result[name] = json!({
                        "values": values,
                        "count": values.len(),
                    });
                }
            }
        }

        // Add dimensions
        if !self.dimensions.is_empty() {
            result["dimensions"] = json!(self.dimensions);
        }

        result
    }

    // Reset all metrics
    pub async fn reset(&self) -> Result<()> {
        let mut metrics = self.metrics.lock().await;
        metrics.clear();

        let mut timers = self.timers.lock().await;
        timers.clear();

        Ok(())
    }

    // Compute histogram aggregation
    pub async fn get_histogram_aggregation(
        &self,
        name: &str,
        aggregation_type: AggregationType,
    ) -> Result<f64> {
        let metrics = self.metrics.lock().await;

        match metrics.get(name) {
            Some(MetricValue::Histogram(values)) => {
                if values.is_empty() {
                    return Ok(0.0);
                }

                match aggregation_type {
                    AggregationType::Sum => Ok(values.iter().sum()),
                    AggregationType::Count => Ok(values.len() as f64),
                    AggregationType::Min => Ok(values.iter().fold(f64::INFINITY, |a, &b| a.min(b))),
                    AggregationType::Max => {
                        Ok(values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)))
                    }
                    AggregationType::Average => {
                        Ok(values.iter().sum::<f64>() / values.len() as f64)
                    }
                    AggregationType::Median => {
                        let mut sorted = values.clone();
                        sorted
                            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        let mid = sorted.len() / 2;
                        if sorted.len() % 2 == 0 {
                            Ok((sorted[mid - 1] + sorted[mid]) / 2.0)
                        } else {
                            Ok(sorted[mid])
                        }
                    }
                    AggregationType::P90 => {
                        let mut sorted = values.clone();
                        sorted
                            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        let idx = (values.len() as f64 * 0.9).ceil() as usize - 1;
                        Ok(sorted[idx.min(sorted.len() - 1)])
                    }
                    AggregationType::P95 => {
                        let mut sorted = values.clone();
                        sorted
                            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        let idx = (values.len() as f64 * 0.95).ceil() as usize - 1;
                        Ok(sorted[idx.min(sorted.len() - 1)])
                    }
                    AggregationType::P99 => {
                        let mut sorted = values.clone();
                        sorted
                            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        let idx = (values.len() as f64 * 0.99).ceil() as usize - 1;
                        Ok(sorted[idx.min(sorted.len() - 1)])
                    }
                }
            }
            Some(_) => Err(anyhow::anyhow!("Metric {} is not a histogram", name)),
            None => Err(anyhow::anyhow!("Metric {} not found", name)),
        }
    }
}

// Default metrics for Lambda
pub fn default_lambda_metrics() -> MetricsCollector {
    let mut dimensions = HashMap::new();

    // Add Lambda dimensions
    if let Ok(function_name) = std::env::var("AWS_LAMBDA_FUNCTION_NAME") {
        dimensions.insert("function_name".to_string(), function_name);
    }

    if let Ok(function_version) = std::env::var("AWS_LAMBDA_FUNCTION_VERSION") {
        dimensions.insert("function_version".to_string(), function_version);
    }

    if let Ok(region) = std::env::var("AWS_REGION") {
        dimensions.insert("region".to_string(), region);
    }

    MetricsCollector::new().with_dimensions(dimensions)
}

// Helper to create a timer that automatically records when dropped
pub struct AutoTimer<'a> {
    collector: &'a MetricsCollector,
    name: String,
    start_time: Instant,
}

impl<'a> AutoTimer<'a> {
    pub fn new(collector: &'a MetricsCollector, name: &str) -> Self {
        Self {
            collector,
            name: name.to_string(),
            start_time: Instant::now(),
        }
    }

    // Get elapsed time so far
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    // Stop and record now, without waiting for Drop
    pub async fn stop(self) -> Result<Duration> {
        let duration = self.elapsed();
        let mut metrics = self.collector.metrics.lock().await;
        metrics.insert(self.name, MetricValue::Timer(duration));
        Ok(duration)
    }
}

impl<'a> Drop for AutoTimer<'a> {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        let name = self.name.clone();
        let collector = self.collector.clone();

        // Spawn a task to record the timing
        // This is necessary because we can't use async in drop
        tokio::spawn(async move {
            let mut metrics = collector.metrics.lock().await;
            metrics.insert(name, MetricValue::Timer(duration));
        });
    }
}
