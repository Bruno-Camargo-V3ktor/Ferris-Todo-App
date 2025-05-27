use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

// Enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TodoListFilter {
    Completed,
    Active,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TodoToggleAction {
    Uncheck,
    Check,
}

// Structs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    pub is_completed: bool,
    pub created_at: SystemTime,
    pub text: String,
    pub id: Uuid,
}

// Impls
impl Todo {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            is_completed: false,
            text: text.into(),
            created_at: SystemTime::now(),
            id: Uuid::new_v4(),
        }
    }
}

impl std::fmt::Display for TodoListFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Completed => write!(f, "Completed"),
            Self::Active => write!(f, "Active"),
            Self::All => write!(f, "All"),
        }
    }
}

impl std::fmt::Display for TodoToggleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uncheck => write!(f, "Uncheck"),
            Self::Check => write!(f, "Check"),
        }
    }
}
