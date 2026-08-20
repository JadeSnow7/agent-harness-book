pub mod config;
pub mod model_call;
pub mod openai_responses;
pub mod protocol;
pub mod transport;

pub use config::{Config, ConfigError, DEFAULT_BASE_URL, DEFAULT_TIMEOUT_S};
pub use model_call::{
    AppError, DEFAULT_INPUT, build_model_request, chat_once, complete, format_response,
};
pub use openai_responses::{
    decode_response, decode_response_json, encode_request, parse_http_response,
};
pub use protocol::*;
pub use transport::{HttpResponse, ReqwestTransport, Transport, TransportError};
