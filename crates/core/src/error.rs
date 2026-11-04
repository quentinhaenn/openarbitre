//! Outcome categories, action errors and the result document (ST-03 specs RG-ST03-2.1, 3.1, 3.4).

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
        todo!("T-2: Category::exit_code — RG-ST03-3.1, INV-ST03-3")
    }

    /// JSON `category` name; `None` for `Success`, which has no error document.
    pub fn json_name(self) -> Option<&'static str> {
        todo!("T-2: Category::json_name — RG-ST03-2.1, 3.1")
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

    /// Used by `test.not-enabled` until ST-04 owns enabling.
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
    let _ = r;
    todo!("T-2: document — RG-ST03-2.1, INV-ST03-4")
}
