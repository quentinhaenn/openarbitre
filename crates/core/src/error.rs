//! Outcome categories, action errors and the result document (ST-03 specs RG-ST03-2.1, 3.1, 3.4).

use alloc::collections::BTreeMap;
use alloc::string::String;

use crate::value::Value;

/// Kind of an outcome; each has one exit code (RG-ST03-3.1, frozen).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Success,
    Refusal,
    Usage,
    ToolError,
    ModuleNotEnabled,
}

impl Category {
    /// 0 to 4, injective, `0 ⇔ Success` (RG-ST03-3.1).
    pub fn exit_code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Refusal => 1,
            Self::Usage => 2,
            Self::ToolError => 3,
            Self::ModuleNotEnabled => 4,
        }
    }

    /// JSON `category` name; `None` for `Success`, which has no error document.
    pub fn json_name(self) -> Option<&'static str> {
        match self {
            Self::Success => None,
            Self::Refusal => Some("refusal"),
            Self::Usage => Some("usage"),
            Self::ToolError => Some("tool_error"),
            Self::ModuleNotEnabled => Some("module_not_enabled"),
        }
    }
}

/// A failure: never `Success` (fields private, built by the constructors below; RG-ST03-3.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionError {
    category: Category,
    id: String,
    message: String,
    data: Option<Value>,
}

impl ActionError {
    fn new(category: Category, id: &str, message: impl Into<String>) -> Self {
        Self {
            category,
            id: id.into(),
            message: message.into(),
            data: None,
        }
    }

    pub fn refusal(id: &str, message: impl Into<String>) -> Self {
        Self::new(Category::Refusal, id, message)
    }

    pub fn tool_error(id: &str, message: impl Into<String>) -> Self {
        Self::new(Category::ToolError, id, message)
    }

    /// The action's module is disabled on this device (exit 4).
    pub fn module_not_enabled(id: &str, message: impl Into<String>) -> Self {
        Self::new(Category::ModuleNotEnabled, id, message)
    }

    /// CLI only: an action returning it is a contract breach (RG-ST03-3.4).
    pub fn usage(id: &str, message: impl Into<String>) -> Self {
        Self::new(Category::Usage, id, message)
    }

    /// Attaches structured details, the failure document's `data` (Q-DA1 (a)).
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn category(&self) -> Category {
        self.category
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn data(&self) -> Option<&Value> {
        self.data.as_ref()
    }
}

/// Result document of RG-ST03-2.1 (frozen shape; failure `data` only when `Some`, plan P-12).
pub fn document(r: &Result<Value, ActionError>) -> Value {
    let s = |x: &str| Value::Str(x.into());
    let mut d = BTreeMap::new();
    d.insert("format".into(), Value::Int(1));
    match r {
        Ok(v) => {
            d.insert("ok".into(), Value::Bool(true));
            d.insert("data".into(), v.clone());
        }
        Err(e) => {
            d.insert("ok".into(), Value::Bool(false));
            let mut err = BTreeMap::new();
            // `Success` is unrepresentable in an error (private fields), so `json_name` is `Some`.
            err.insert("category".into(), s(e.category.json_name().unwrap_or("")));
            err.insert("code".into(), Value::Int(e.category.exit_code().into()));
            err.insert("id".into(), s(&e.id));
            err.insert("message".into(), s(&e.message));
            d.insert("error".into(), Value::Object(err));
            if let Some(x) = &e.data {
                d.insert("data".into(), x.clone());
            }
        }
    }
    Value::Object(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_accessor() {
        let e = ActionError::refusal("x", "y");
        assert_eq!(e.data(), None);
        assert_eq!(e.with_data(Value::Int(1)).data(), Some(&Value::Int(1)));
    }
}
