use std::time::Duration;

use media_resolver_core::{PreparedRequest, RequestMethod};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportError;

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
        let mut builder = match request.method {
            RequestMethod::Get => self.client.get(request.url.as_str()),
        };
        for header in &request.headers {
            builder = builder.header(header.name.as_str(), header.value.as_str());
        }

        let response = builder.send().await.map_err(|_| TransportError)?;
        let status = response.status().as_u16();
        let bytes = response.bytes().await.map_err(|_| TransportError)?;
        Ok((status, bytes.to_vec()))
    }
}
