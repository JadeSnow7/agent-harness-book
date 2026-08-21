use reqwest::blocking::Client;
use serde_json::Value;
use std::{collections::BTreeMap, fmt, time::Duration};

/// Transport 返回的最小 HTTP 结果；不携带请求 Header 或其他调用方不需要的元数据。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

/// Transport 失败的分类；`Display` 不暴露底层 HTTP 库的错误细节，避免意外泄漏请求内容。
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

/// HTTP 传输的最小边界；生产代码使用 `ReqwestTransport`，测试可以实现同一 trait 注入 Fake。
pub trait Transport {
    fn send(
        &self,
        endpoint: &str,
        headers: &BTreeMap<String, String>,
        request: &Value,
        timeout_s: f64,
    ) -> Result<HttpResponse, TransportError>;
}

/// 生产 Transport 实现：用阻塞版 `reqwest::Client` 发送一次非流式 POST。
#[derive(Clone, Debug, Default)]
pub struct ReqwestTransport;

impl Transport for ReqwestTransport {
    /// 发送请求；除超时转换失败外的错误一律归类为 `Network`，不向上暴露底层 reqwest 错误细节。
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
