//! Shared provider retry helper for transient network failures.

use rand::Rng;
use reqwest::{RequestBuilder, Response, StatusCode};

const MAX_ATTEMPTS: usize = 3;

pub async fn send_with_retry(
    request: RequestBuilder,
    provider_name: &str,
) -> anyhow::Result<Response> {
    let mut last_error = None;

    for attempt in 0..MAX_ATTEMPTS {
        let Some(builder) = request.try_clone() else {
            anyhow::bail!("{} request could not be cloned for retry", provider_name);
        };

        match builder.send().await {
            Ok(response) => {
                if !should_retry_status(response.status()) || attempt + 1 == MAX_ATTEMPTS {
                    return Ok(response);
                }

                let delay = retry_delay(attempt, response.headers().get("retry-after"));
                tracing::warn!(
                    provider = provider_name,
                    status = %response.status(),
                    attempt = attempt + 1,
                    delay_ms = delay.as_millis(),
                    "retrying transient provider error"
                );
                tokio::time::sleep(delay).await;
            }
            Err(error) => {
                if !is_transient_error(&error) || attempt + 1 == MAX_ATTEMPTS {
                    return Err(error.into());
                }

                let delay = retry_delay(attempt, None);
                tracing::warn!(
                    provider = provider_name,
                    attempt = attempt + 1,
                    delay_ms = delay.as_millis(),
                    error = %error,
                    "retrying transient provider transport error"
                );
                last_error = Some(error);
                tokio::time::sleep(delay).await;
            }
        }
    }

    match last_error {
        Some(error) => Err(error.into()),
        None => anyhow::bail!("{} request failed without a response", provider_name),
    }
}

fn should_retry_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    )
}

fn is_transient_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect() || error.is_request() || error.is_body()
}

fn retry_delay(
    attempt: usize,
    retry_after: Option<&reqwest::header::HeaderValue>,
) -> std::time::Duration {
    if let Some(header) = retry_after {
        if let Ok(value) = header.to_str() {
            if let Ok(seconds) = value.parse::<u64>() {
                return std::time::Duration::from_secs(seconds.min(30));
            }
        }
    }

    let base_ms = 250_u64.saturating_mul(1_u64 << attempt.min(4));
    let jitter_ms = rand::thread_rng().gen_range(0..=150);
    std::time::Duration::from_millis(base_ms + jitter_ms)
}
