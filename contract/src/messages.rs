use serde::{Deserialize, Serialize};

/// MQTT 메시지 버전
pub const CONTRACT_VERSION: &str = "1.0.0";

/// MQTT 토픽 정의
#[cfg(feature = "mqtt")]
pub mod topics {
    // Manager 토픽
    pub const MANAGER_COMMAND: &str = "manager/command";
    pub const MANAGER_HEALTH_REPORT: &str = "manager/health/report";
    pub const MANAGER_PROCESS_STATUS: &str = "manager/process/status";
    pub const MANAGER_PROCESS_ALERT: &str = "manager/process/alert";

    // RinAgent 토픽 (현재는 사용되지 않음)
    pub const RIN_AGENT_COMMAND: &str = "rin_agent/command";
    pub const RIN_AGENT_STATUS: &str = "rin_agent/status";
    pub const RIN_AGENT_ALERT: &str = "rin_agent/alert";
}

/// 로그 레벨 정의
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// Redis에 저장되는 로그 패킷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPacket {
    /// Unix timestamp (초)
    pub timestamp: u64,
    /// 로그 레벨
    pub level: LogLevel,
    /// 로그 메시지
    pub message: String,
    /// 로그 출처 (rin_agent, manager, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// 추가 메타데이터
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Manager 명령 타입 (MQTT feature 필요)
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ManagerCommand {
    /// 프로세스 재시작 요청
    RestartProcess {
        process_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        force: Option<bool>,
    },
    /// 헬스 체크 요청
    HealthCheck,
    /// 시스템 정보 요청
    SystemInfo,
    /// 프로세스 모니터링 시작
    StartMonitoring {
        process_name: String,
        interval_secs: u64,
    },
    /// 프로세스 모니터링 중지
    StopMonitoring { process_name: String },
}

/// Manager 응답 타입 (MQTT feature 필요)
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ManagerResponse {
    /// 성공 응답
    Success {
        command: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },
    /// 오류 응답
    Error { command: String, error: String },
    /// 헬스 체크 응답
    HealthReport {
        cpu_usage: f32,
        memory_usage_percent: f64,
        total_memory_mb: u64,
        used_memory_mb: u64,
        timestamp: u64,
    },
    /// 프로세스 상태 응답
    ProcessStatus {
        process_name: String,
        is_running: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pid: Option<u32>,
        timestamp: u64,
    },
}

/// RinAgent 명령 타입 (MQTT feature 필요 - 현재 사용되지 않음)
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RinAgentCommand {
    /// 알람 트리거
    TriggerAlarm {
        alarm_id: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        custom_message: Option<String>,
    },
    /// 서비스 재시작
    Restart { graceful: bool },
    /// 설정 리로드
    ReloadConfig,
    /// Discord 봇 상태 확인
    BotStatus,
}

/// RinAgent 상태 응답 (MQTT feature 필요 - 현재 사용되지 않음)
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RinAgentStatus {
    /// 정상 운영 중
    Running {
        uptime_secs: u64,
        connected_guilds: usize,
        active_voice_sessions: usize,
        timestamp: u64,
    },
    /// 시작 중
    Starting { timestamp: u64 },
    /// 종료 중
    ShuttingDown { reason: String, timestamp: u64 },
    /// 오류 상태
    Error { error: String, timestamp: u64 },
}

/// 프로세스 재시작 결과 (MQTT feature 필요)
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartResult {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

impl LogPacket {
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now().timestamp() as u64,
            level,
            message: message.into(),
            source: None,
            metadata: None,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_packet_serialization() {
        let packet = LogPacket::new(LogLevel::Info, "Test message")
            .with_source("rin_agent");

        let json = serde_json::to_string(&packet).unwrap();
        let deserialized: LogPacket = serde_json::from_str(&json).unwrap();

        assert_eq!(packet.level, deserialized.level);
        assert_eq!(packet.message, deserialized.message);
        assert_eq!(packet.source, deserialized.source);
    }

    #[test]
    #[cfg(feature = "mqtt")]
    fn test_manager_command_serialization() {
        let cmd = ManagerCommand::RestartProcess {
            process_name: "rin_agent".to_string(),
            force: Some(true),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: ManagerCommand = serde_json::from_str(&json).unwrap();

        match deserialized {
            ManagerCommand::RestartProcess { process_name, force } => {
                assert_eq!(process_name, "rin_agent");
                assert_eq!(force, Some(true));
            }
            _ => panic!("Wrong command type"),
        }
    }
}
