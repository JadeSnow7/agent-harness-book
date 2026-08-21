use std::{collections::HashMap, env, fmt};

/// OpenAI API 的默认根地址；可通过 `OPENAI_BASE_URL` 替换为兼容服务地址。
pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

/// 没有显式配置超时时使用的秒数，避免请求无限等待。
pub const DEFAULT_TIMEOUT_S: f64 = 60.0;

/// 一次模型调用所需的配置；`api_key` 只在内存中传递，调用方不应打印或记录它。
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub timeout_s: f64,
}

impl Config {
    /// 从当前进程环境变量读取配置；不发起网络请求。
    pub fn from_env() -> Result<Self, ConfigError> {
        let values: HashMap<String, String> = env::vars().collect();
        Self::from_values(&values)
    }

    /// 从任意 key-value map 读取配置；测试可以传入固定 map，不依赖真实进程环境。
    pub fn from_values(values: &HashMap<String, String>) -> Result<Self, ConfigError> {
        let api_key = required(values, "OPENAI_API_KEY")?;
        let model = required(values, "OPENAI_MODEL")?;
        let base_url = match values.get("OPENAI_BASE_URL") {
            None => DEFAULT_BASE_URL.to_owned(),
            Some(value) if !value.trim().is_empty() => value.clone(),
            Some(_) => return Err(ConfigError::InvalidBaseUrl),
        };
        let timeout_s = match values.get("OPENAI_TIMEOUT_S") {
            None => DEFAULT_TIMEOUT_S,
            Some(value) => value
                .parse::<f64>()
                .map_err(|_| ConfigError::InvalidTimeout)?,
        };
        // 必须同时排除 NaN/inf 与非正数，并确认能转换成 Duration；否则一个畸形超时会在
        // Transport 层才失败，报错位置会离真正的用户输入错误更远。
        if !timeout_s.is_finite()
            || timeout_s <= 0.0
            || std::time::Duration::try_from_secs_f64(timeout_s).is_err()
        {
            return Err(ConfigError::InvalidTimeout);
        }

        Ok(Self {
            api_key,
            model,
            base_url,
            timeout_s,
        })
    }

    /// 拼出 Responses API 端点，不让调用流程重复处理斜杠。
    pub fn endpoint(&self) -> String {
        format!("{}/responses", self.base_url.trim_end_matches('/'))
    }
}

/// 读取一个必需的非空白配置项；缺失或全是空白都归类为 `Missing`。
fn required(values: &HashMap<String, String>, name: &'static str) -> Result<String, ConfigError> {
    values
        .get(name)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or(ConfigError::Missing(name))
}

/// 配置校验失败的分类；`Display` 只描述缺失或非法的字段名，不包含任何配置值本身。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigError {
    Missing(&'static str),
    InvalidBaseUrl,
    InvalidTimeout,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(name) => write!(f, "missing {name}"),
            Self::InvalidBaseUrl => f.write_str("OPENAI_BASE_URL must not be empty"),
            Self::InvalidTimeout => {
                f.write_str("OPENAI_TIMEOUT_S must be a positive finite number")
            }
        }
    }
}

impl std::error::Error for ConfigError {}
