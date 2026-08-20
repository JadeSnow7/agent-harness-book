use reqwest::blocking::Client;
use serde_json::Value;
use std::{collections::BTreeMap, fmt, time::Duration};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransportError {
    Network,
    InvalidTimeout,
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network => f.write_str("request failed"),
            Self::InvalidTimeout => f.write_str("invalid request timeout"),
        }
    }
}

impl std::error::Error for TransportError {}

pub trait Transport {
    fn send(
        &self,
        endpoint: &str,
        headers: &BTreeMap<String, String>,
        request: &Value,
        timeout_s: f64,
    ) -> Result<HttpResponse, TransportError>;
}

#[derive(Clone, Debug, Default)]
pub struct ReqwestTransport;

impl Transport for ReqwestTransport {
    fn send(
        &self,
        endpoint: &str,
        headers: &BTreeMap<String, String>,
        request: &Value,
        timeout_s: f64,
    ) -> Result<HttpResponse, TransportError> {
        let timeout =
            Duration::try_from_secs_f64(timeout_s).map_err(|_| TransportError::InvalidTimeout)?;
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|_| TransportError::Network)?;
        let response = client
            .post(endpoint)
            .headers(
                headers
                    .iter()
                    .try_fold(
                        reqwest::header::HeaderMap::new(),
                        |mut map, (key, value)| {
                            let name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                                .map_err(|_| ())?;
                            let value =
                                reqwest::header::HeaderValue::from_str(value).map_err(|_| ())?;
                            map.insert(name, value);
                            Ok::<_, ()>(map)
                        },
                    )
                    .map_err(|_| TransportError::Network)?,
            )
            .json(request)
            .send()
            .map_err(|_| TransportError::Network)?;
        let status = response.status().as_u16();
        let body = response.text().map_err(|_| TransportError::Network)?;
        Ok(HttpResponse { status, body })
    }
}
