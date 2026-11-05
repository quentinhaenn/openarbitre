//! `Ai` port: declared in ST-04, adapter in ST-30 (ST-04 specs RG-ST04-3.5).

use crate::error::ActionError;
use crate::module::LocalBoxFuture;
use crate::value::Value;

/// An AI provider: one structured request, one structured answer (frozen).
pub trait Ai {
    fn complete<'a>(
        &'a self,
        request: &'a Value,
        timeout_ms: u64,
    ) -> LocalBoxFuture<'a, Result<Value, AiError>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiError {
    NotConfigured,
    Unreachable,
    KeyRefused,
    Timeout,
}

impl From<AiError> for ActionError {
    /// `ToolError` with ids `ai_not_configured`, `ai_unreachable`, `ai_key_refused`, `ai_timeout`.
    fn from(e: AiError) -> Self {
        let id = match e {
            AiError::NotConfigured => "ai_not_configured",
            AiError::Unreachable => "ai_unreachable",
            AiError::KeyRefused => "ai_key_refused",
            AiError::Timeout => "ai_timeout",
        };
        ActionError::tool_error(id, id.replace('_', " "))
    }
}
