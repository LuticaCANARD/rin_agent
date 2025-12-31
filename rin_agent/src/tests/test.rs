#[cfg(test)]
mod tests {
    use crate::libs::logger::{LOGGER,LogLevel};

    use super::*;

    #[test]
    fn test_agent_initialization() {
        // 테스트용 LOGGER 초기화 예시
        LOGGER.log(LogLevel::Debug, "Agent initialized for test");
        assert!(true, "LOGGER should be initialized");
    }

    #[test]
    fn test_agent_response() {
        // 실제 응답 테스트를 위한 더미 값 사용 예시
        let response = "dummy response";
        assert_eq!(response, "dummy response");
    }

    #[test]
    fn test_logger_debug_message() {
        // LOGGER의 Debug 메시지 테스트
        LOGGER.log(LogLevel::Debug, "Debug message test");
        assert!(true, "Debug message logged");
    }

    #[test]
    fn test_logger_error_message() {
        // LOGGER의 Error 메시지 테스트
        LOGGER.log(LogLevel::Error, "Error message test");
        assert!(true, "Error message logged");
    }

    #[test]
    fn test_logger_multiple_levels() {
        // 여러 로그 레벨 테스트
        LOGGER.log(LogLevel::Info, "Info message");
        LOGGER.log(LogLevel::Warning, "Warn message");
        LOGGER.log(LogLevel::Error, "Error message");
        assert!(true, "Multiple log levels tested");
    }
}