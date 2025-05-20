// agent/core/lib/transport/retry.rs

use std::io;
use std::time::Duration;

use reqwest::{blocking::Client, header};
use tracing::{debug, error, info, warn};

/// Ships data to an endpoint with retry logic
///
/// # Arguments
/// * `url` - The endpoint URL to ship to
/// * `data` - The data to ship
/// * `max_attempts` - Maximum number of retry attempts
/// * `retry_delay` - Delay between retry attempts
///
/// # Returns
/// * `Result<(), io::Error>` - Ok if shipping was successful, Err otherwise
pub fn ship_with_retry(
    url: &str,
    data: &[u8],
    max_attempts: u32,
    retry_delay: Duration,
) -> Result<(), io::Error> {
    let client = Client::new();
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/octet-stream"),
    );

    for attempt in 1..=max_attempts {
        debug!("Shipping telemetry data to {}, attempt {}/{}", url, attempt, max_attempts);
        
        let result = client
            .post(url)
            .headers(headers.clone())
            .body(data.to_vec())
            .send();
            
        match result {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Successfully shipped telemetry data to {}", url);
                    return Ok(());
                } else {
                    warn!(
                        "Failed to ship telemetry data: HTTP status {}, attempt {}/{}",
                        response.status(),
                        attempt,
                        max_attempts
                    );
                }
            }
            Err(e) => {
                warn!(
                    "Failed to ship telemetry data: {}, attempt {}/{}",
                    e,
                    attempt,
                    max_attempts
                );
            }
        }
        
        if attempt < max_attempts {
            // Exponential backoff with jitter
            let jitter = rand::random::<f64>() * 0.5 + 0.75; // 0.75 to 1.25
            let delay = retry_delay.mul_f64(jitter * attempt as f64);
            debug!("Retrying in {:?}", delay);
            std::thread::sleep(delay);
        }
    }
    
    error!("Failed to ship telemetry data after {} attempts", max_attempts);
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("Failed to ship telemetry data after {} attempts", max_attempts),
    ))
}

/// Asynchronous version of ship_with_retry
#[cfg(feature = "async")]
pub async fn ship_with_retry_async(
    url: &str,
    data: &[u8],
    max_attempts: u32,
    retry_delay: Duration,
) -> Result<(), io::Error> {
    use reqwest::Client;
    use tokio::time::sleep;

    let client = Client::new();
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/octet-stream"),
    );

    for attempt in 1..=max_attempts {
        debug!("Shipping telemetry data to {}, attempt {}/{}", url, attempt, max_attempts);
        
        let result = client
            .post(url)
            .headers(headers.clone())
            .body(data.to_vec())
            .send()
            .await;
            
        match result {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Successfully shipped telemetry data to {}", url);
                    return Ok(());
                } else {
                    warn!(
                        "Failed to ship telemetry data: HTTP status {}, attempt {}/{}",
                        response.status(),
                        attempt,
                        max_attempts
                    );
                }
            }
            Err(e) => {
                warn!(
                    "Failed to ship telemetry data: {}, attempt {}/{}",
                    e,
                    attempt,
                    max_attempts
                );
            }
        }
        
        if attempt < max_attempts {
            // Exponential backoff with jitter
            let jitter = rand::random::<f64>() * 0.5 + 0.75; // 0.75 to 1.25
            let delay = retry_delay.mul_f64(jitter * attempt as f64);
            debug!("Retrying in {:?}", delay);
            sleep(delay).await;
        }
    }
    
    error!("Failed to ship telemetry data after {} attempts", max_attempts);
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("Failed to ship telemetry data after {} attempts", max_attempts),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};
    use std::time::Duration;

    #[test]
    fn test_ship_success() {
        let data = b"test data";
        
        let _m = mock("POST", "/telemetry")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body("OK")
            .create();
        
        let result = ship_with_retry(
            &format!("{}/telemetry", server_url()),
            data,
            3,
            Duration::from_millis(10),
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_ship_retry_then_success() {
        let data = b"test data";
        
        let _m1 = mock("POST", "/telemetry")
            .with_status(500)
            .with_header("content-type", "application/octet-stream")
            .create();
        
        let _m2 = mock("POST", "/telemetry")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body("OK")
            .create();
        
        let result = ship_with_retry(
            &format!("{}/telemetry", server_url()),
            data,
            3,
            Duration::from_millis(10),
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_ship_all_retries_fail() {
        let data = b"test data";
        
        let _m = mock("POST", "/telemetry")
            .with_status(500)
            .with_header("content-type", "application/octet-stream")
            .expect(3)
            .create();
        
        let result = ship_with_retry(
            &format!("{}/telemetry", server_url()),
            data,
            3,
            Duration::from_millis(10),
        );
        
        assert!(result.is_err());
    }
}
