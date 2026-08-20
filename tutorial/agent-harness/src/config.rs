use std::{collections::HashMap, env, fmt};

pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_TIMEOUT_S: f64 = 60.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub timeout_s: f64,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let values: HashMap<String, String> = env::vars().collect();
        Self::from_values(&values)
    }

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

    pub fn endpoint(&self) -> String {
        format!("{}/responses", self.base_url.trim_end_matches('/'))
    }
}

fn required(values: &HashMap<String, String>, name: &'static str) -> Result<String, ConfigError> {
    values
        .get(name)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or(ConfigError::Missing(name))
}

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
