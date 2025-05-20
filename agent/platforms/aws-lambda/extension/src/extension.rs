use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use std::time::Duration;

const LAMBDA_EXTENSION_NAME: &str = "probe";
const EXTENSIONS_API_VERSION: &str = "2020-01-01";

#[derive(Debug, Serialize, Clone)]
struct RegistrationRequest {
    events: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RegistrationResponse {
    pub extension_id: String,
    pub function_name: String,
    pub handler: String,
    pub state: String,
}

pub struct LambdaExtension {
    client: Client,
    base_url: String,
    extension_id: Option<String>,
}

impl LambdaExtension {
    pub fn new() -> Result<Self> {
        let runtime_api_address = std::env::var("AWS_LAMBDA_RUNTIME_API")
            .context("AWS_LAMBDA_RUNTIME_API must be set")?;
        
        // Use HTTP for Lambda internal API (HTTPS not needed and adds overhead)
        let base_url = format!("http://{}/2020-01-01/extension", runtime_api_address);
        
        // Optimize HTTP client for Lambda environment
        let client = Client::builder()
            .pool_max_idle_per_host(0)
            .timeout(Duration::from_secs(2))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            client,
            base_url,
            extension_id: None,
        })
    }
    
    pub async fn register(&mut self) -> Result<RegistrationResponse> {
        let url = format!("{}/register", self.base_url);
        
        // Prepare the request body only once (avoid repetitive allocations)
        let events = vec!["INVOKE".to_string(), "SHUTDOWN".to_string()];
        let request_body = RegistrationRequest { events };
        
        // Use an optimized HTTP request with reasonable timeouts
        let response = self.client
            .post(&url)
            .header("Lambda-Extension-Name", LAMBDA_EXTENSION_NAME)
            .json(&request_body)
            .send()
            .await
            .context("Failed to register extension")?;
            
        // Early status check to avoid parsing non-OK responses
        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await
                .context("Failed to read error response body")?;
            return Err(anyhow::anyhow!(
                "Extension registration failed with status {}: {}", 
                status, error_body
            ));
        }
            
        let registration = response
            .json::<RegistrationResponse>()
            .await
            .context("Failed to parse registration response")?;
            
        // Store extension ID for later use
        self.extension_id = Some(registration.extension_id.clone());
        
        Ok(registration)
    }
    
    pub fn get_extension_id(&self) -> Option<&str> {
        self.extension_id.as_deref()
    }
}
