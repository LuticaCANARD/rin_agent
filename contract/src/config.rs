use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// 환경변수 로드 전략
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvLoadStrategy {
    /// .env 파일만 사용
    DotenvOnly,
    /// 시스템 환경변수만 사용
    SystemOnly,
    /// .env 파일 우선, 시스템 환경변수로 폴백
    DotenvPreferred,
    /// 시스템 환경변수 우선, .env 파일로 폴백
    SystemPreferred,
}

impl Default for EnvLoadStrategy {
    fn default() -> Self {
        Self::SystemPreferred
    }
}

/// 환경변수 설정 빌더
pub struct EnvConfigBuilder {
    strategy: EnvLoadStrategy,
    dotenv_path: Option<PathBuf>,
    ignore_missing: bool,
}

impl Default for EnvConfigBuilder {
    fn default() -> Self {
        Self {
            strategy: EnvLoadStrategy::SystemPreferred,
            dotenv_path: None,
            ignore_missing: false,
        }
    }
}

impl EnvConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// 환경변수 로드 전략 설정
    pub fn strategy(mut self, strategy: EnvLoadStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// .env 파일 경로 지정 (기본: ./.env)
    pub fn dotenv_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.dotenv_path = Some(path.into());
        self
    }

    /// 누락된 환경변수 무시 (기본값 사용)
    pub fn ignore_missing(mut self, ignore: bool) -> Self {
        self.ignore_missing = ignore;
        self
    }

    /// 환경변수 로드 실행
    pub fn load(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.strategy {
            EnvLoadStrategy::DotenvOnly => {
                self.load_dotenv()?;
            }
            EnvLoadStrategy::SystemOnly => {
                // 시스템 환경변수만 사용 (아무것도 하지 않음)
            }
            EnvLoadStrategy::DotenvPreferred => {
                // .env 우선, 이미 설정된 시스템 환경변수는 덮어쓰지 않음
                if let Err(e) = self.load_dotenv() {
                    if !self.ignore_missing {
                        return Err(e);
                    }
                }
            }
            EnvLoadStrategy::SystemPreferred => {
                // 시스템 환경변수 우선, .env는 없는 것만 채움
                if let Some(path) = &self.dotenv_path {
                    dotenv::from_path(path).ok();
                } else {
                    dotenv::dotenv().ok();
                }
            }
        }
        Ok(())
    }

    fn load_dotenv(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = &self.dotenv_path {
            dotenv::from_path(path)?;
        } else {
            dotenv::dotenv()?;
        }
        Ok(())
    }
}

/// 공통 환경변수 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonConfig {
    /// Redis URL
    pub redis_url: String,
    /// PostgreSQL URL
    pub database_url: String,
    /// MQTT 호스트
    pub mqtt_host: String,
    /// MQTT 포트
    pub mqtt_port: u16,
}

impl CommonConfig {
    /// 환경변수에서 로드
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            database_url: env::var("DATABASE_URL")?,
            mqtt_host: env::var("MQTT_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            mqtt_port: env::var("MQTT_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(1883),
        })
    }
}

/// Manager 전용 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerConfig {
    pub common: CommonConfig,
    pub mqtt_client_id: String,
    pub log_batch_size: usize,
    pub log_flush_interval_secs: u64,
    pub health_check_interval_secs: u64,
    pub monitored_process_name: Option<String>,
}

impl ManagerConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            common: CommonConfig::from_env()?,
            mqtt_client_id: env::var("MQTT_CLIENT_ID")
                .unwrap_or_else(|_| "rin_manager".to_string()),
            log_batch_size: env::var("LOG_BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            log_flush_interval_secs: env::var("LOG_FLUSH_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            health_check_interval_secs: env::var("HEALTH_CHECK_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            monitored_process_name: env::var("MONITORED_PROCESS_NAME").ok(),
        })
    }
}

/// RinAgent 전용 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RinAgentConfig {
    pub common: CommonConfig,
    pub discord_token: String,
    pub discord_client_id: String,
    pub gemini_api_key: String,
    pub manager_id: Option<String>,
}

impl RinAgentConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            common: CommonConfig::from_env()?,
            discord_token: env::var("DISCORD_TOKEN")?,
            discord_client_id: env::var("DISCORD_CLIENT_ID")?,
            gemini_api_key: env::var("GEMINI_API_KEY")?,
            manager_id: env::var("MANAGER_ID").ok(),
        })
    }
}

/// CLI 인자 파싱 헬퍼
pub fn parse_env_strategy_from_args() -> EnvLoadStrategy {
    let args: Vec<String> = env::args().collect();
    
    for (i, arg) in args.iter().enumerate() {
        if arg == "--env-strategy" {
            if let Some(strategy) = args.get(i + 1) {
                return match strategy.as_str() {
                    "dotenv-only" => EnvLoadStrategy::DotenvOnly,
                    "system-only" => EnvLoadStrategy::SystemOnly,
                    "dotenv-preferred" => EnvLoadStrategy::DotenvPreferred,
                    "system-preferred" => EnvLoadStrategy::SystemPreferred,
                    _ => EnvLoadStrategy::default(),
                };
            }
        }
    }
    
    EnvLoadStrategy::default()
}

/// CLI에서 .env 파일 경로 파싱
pub fn parse_dotenv_path_from_args() -> Option<PathBuf> {
    let args: Vec<String> = env::args().collect();
    
    for (i, arg) in args.iter().enumerate() {
        if arg == "--env-file" {
            if let Some(path) = args.get(i + 1) {
                return Some(PathBuf::from(path));
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_config_builder() {
        let builder = EnvConfigBuilder::new()
            .strategy(EnvLoadStrategy::DotenvOnly)
            .ignore_missing(true);

        assert_eq!(builder.strategy, EnvLoadStrategy::DotenvOnly);
        assert!(builder.ignore_missing);
    }
}
