//! ch02 / M0 基线：一次非流式模型调用，不依赖任何统一协议层。
//!
//! 只做配置读取、请求构造、HTTP 传输和响应解析；不涉及统一消息结构、内容块、工具声明这些
//! 后续章节才引入的概念，也没有会话历史或工具执行。

pub mod config;
pub mod model_call;
pub mod transport;

pub use config::{Config, ConfigError, DEFAULT_BASE_URL, DEFAULT_TIMEOUT_S};
pub use model_call::{AppError, DEFAULT_INPUT, build_request, chat_once, parse_http_response};
pub use transport::{HttpResponse, ReqwestTransport, Transport, TransportError};
