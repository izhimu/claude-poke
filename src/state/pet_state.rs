use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PetState {
    /// Claude Code 未运行或已关闭
    Sleeping,
    /// 等待用户输入（UserPromptSubmit）
    Waiting,
    /// 正在执行工具（PreToolUse）
    Working,
    /// 思考中（PostToolUse / 处理完成后的短暂状态）
    Thinking,
    /// 收到通知（Notification）
    Notify(String),
    /// 子代理工作中（SubAgentStart）
    SubAgentWorking,
    /// 出错
    Error,
    /// 停止
    Stopped,
}

impl Default for PetState {
    fn default() -> Self {
        PetState::Sleeping
    }
}

impl fmt::Display for PetState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PetState::Sleeping => write!(f, "Sleeping"),
            PetState::Waiting => write!(f, "Waiting"),
            PetState::Working => write!(f, "Working"),
            PetState::Thinking => write!(f, "Thinking"),
            PetState::Notify(msg) => write!(f, "Notify: {}", msg),
            PetState::SubAgentWorking => write!(f, "SubAgentWorking"),
            PetState::Error => write!(f, "Error"),
            PetState::Stopped => write!(f, "Stopped"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusFile {
    pub state: PetState,
    pub timestamp: u64,
    pub session_id: String,
    pub message: Option<String>,
}

impl StatusFile {
    #[allow(dead_code)]
    pub fn is_expired(&self, timeout_ms: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        now.saturating_sub(self.timestamp) > timeout_ms
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pet_state_default() {
        assert_eq!(PetState::default(), PetState::Sleeping);
    }

    #[test]
    fn test_status_file_parse() {
        let json = r#"{"state":"Working","timestamp":1717411200000,"session_id":"abc123","message":null}"#;
        let status = StatusFile::from_json(json).unwrap();
        assert_eq!(status.state, PetState::Working);
        assert_eq!(status.session_id, "abc123");
        assert!(status.message.is_none());
    }

    #[test]
    fn test_status_file_parse_notify() {
        let json = r#"{"state":{"Notify":"Hello"},"timestamp":1717411200000,"session_id":"test","message":"Hello"}"#;
        let status = StatusFile::from_json(json).unwrap();
        assert_eq!(status.state, PetState::Notify("Hello".to_string()));
    }

    #[test]
    fn test_status_file_expired() {
        let status = StatusFile {
            state: PetState::Working,
            timestamp: 1000,
            session_id: "test".to_string(),
            message: None,
        };
        // Very old timestamp should be expired
        assert!(status.is_expired(1000));
    }

    #[test]
    fn test_pet_state_display() {
        assert_eq!(PetState::Sleeping.to_string(), "Sleeping");
        assert_eq!(PetState::Working.to_string(), "Working");
        assert_eq!(
            PetState::Notify("test".to_string()).to_string(),
            "Notify: test"
        );
    }
}
