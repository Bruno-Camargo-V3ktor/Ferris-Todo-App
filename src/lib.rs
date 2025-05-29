use models::{TodoListFilter, TodoToggleAction};
use repository::{TodoRepo, TodoRepoError};

pub mod models;
pub mod repository;

// Enums
enum AppError {
    TodoRepo(TodoRepoError),
}

// struct
#[derive(Debug)]
pub struct AppState {
    pub selected_filter: TodoListFilter,
    pub toggle_action: TodoToggleAction,
    pub todo_repo: TodoRepo,
}

// Impls
impl Default for AppState {
    fn default() -> Self {
        Self {
            selected_filter: TodoListFilter::All,
            toggle_action: TodoToggleAction::Check,
            todo_repo: TodoRepo::default(),
        }
    }
}

impl From<TodoRepoError> for AppError {
    fn from(value: TodoRepoError) -> Self {
        Self::TodoRepo(value)
    }
}
