use std::time::Duration;

use media_resolver_core::{PreparedRequest, RequestMethod, TransportFailure};
use reqwest::header::RETRY_AFTER;
use tokio::time::sleep;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportError {
    Network,
    Timeout,
}

impl TransportError {
    pub const fn as_core_failure(self) -> TransportFailure {
        match self {
            Self::Network => TransportFailure::Network,
            Self::Timeout => TransportFailure::Timeout,
        }
    }
}

#[derive(Clone)]
pub struct RemoteClient {
    client: reqwest::Client,
}

impl RemoteClient {
    pub fn new(timeout: Duration) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::limited(8))
            .user_agent(concat!("media-resolver/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { client })
    }

    pub async fn execute(
        &self,
        request: &PreparedRequest,
    ) -> Result<(u16, Vec<u8>), TransportError> {
        for attempt in 0..=request.retry_policy.max_retries {
            let response = match self.send_once(request).await {
                Ok(response) => response,
                Err(error) => {
                    if attempt >= request.retry_policy.max_retries {
                        return Err(error);
                    }
                    sleep(retry_delay(request, attempt, None)).await;
                    continue;
                }
            };

            let status = response.status().as_u16();
            let retry_after = retry_after_delay(&response);
            if request.retry_policy.retry_statuses.contains(&status)
                && attempt < request.retry_policy.max_retries
            {
                drop(response);
                sleep(retry_delay(request, attempt, retry_after)).await;
                continue;
            }

            let bytes = response.bytes().await.map_err(map_reqwest_error)?;
            return Ok((status, bytes.to_vec()));
        }

        Err(TransportError::Network)
    }

    async fn send_once(
        &self,
        request: &PreparedRequest,
    ) -> Result<reqwest::Response, TransportError> {
        let mut builder = match request.method {
            RequestMethod::Get => self.client.get(request.url.as_str()),
        };
        for header in &request.headers {
            builder = builder.header(header.name.as_str(), header.value.as_str());
        }
        builder.send().await.map_err(map_reqwest_error)
    }
}

fn map_reqwest_error(error: reqwest::Error) -> TransportError {
    if error.is_timeout() {
        TransportError::Timeout
    } else {
        TransportError::Network
    }
}

fn retry_after_delay(response: &reqwest::Response) -> Option<Duration> {
    let seconds = response
        .headers()
        .get(RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;
    Some(Duration::from_secs(seconds.min(300)))
}

fn retry_delay(request: &PreparedRequest, attempt: u8, retry_after: Option<Duration>) -> Duration {
    if let Some(delay) = retry_after {
        return delay;
    }
    let index = usize::from(attempt);
    let milliseconds = request
        .retry_policy
        .delays_ms
        .get(index)
        .copied()
        .or_else(|| request.retry_policy.delays_ms.last().copied())
        .unwrap_or(500);
    Duration::from_millis(milliseconds)
}
