use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get_service,
    Router,
};
use models::{Todo, TodoListFilter, TodoToggleAction};
use repository::{TodoRepo, TodoRepoError};
use tokio::sync::RwLock;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub mod models;
pub mod repository;

// Types
pub type SharedState = Arc<RwLock<AppError>>;

// Enums
pub enum AppError {
    TodoRepo(TodoRepoError),
}

// struct
#[derive(Debug)]
pub struct AppState {
    pub selected_filter: TodoListFilter,
    pub toggle_action: TodoToggleAction,
    pub todo_repo: TodoRepo,
}

struct GetIndexResponse;

struct ListTodosResponse {
    num_completed_items: u32,
    num_active_items: u32,
    num_all_items: u32,
    is_disabled_delete: bool,
    is_disabled_toggle: bool,
    action: TodoToggleAction,
    items: Vec<Todo>,
}

struct ListTodosQuery {
    pub filter: TodoListFilter,
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

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::TodoRepo(TodoRepoError::NotFound) => (StatusCode::NOT_FOUND, "Todo not found"),
        };

        (status, message).into_response()
    }
}

// Fucntions
pub fn app(shared_state: SharedState) -> Router {
    Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get_service(get_index))
        .route(
            "/todo",
            get_service(list_todos)
                .post_service(create_todo)
                .patch_service(toggle_completed_todos)
                .delete_service(delete_completed_todos),
        )
        .route(
            "/todo/:id",
            get_service(edit_todo)
                .patch_service(update_todo)
                .delete_service(delete_todo),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state)
}

async fn get_index() -> Result<GetIndexResponse, AppError> {
    Ok(GetIndexResponse)
}

async fn list_todo(
    State(shared_state): State<SharedState>,
    Query(ListTodosQuery { filter }): Query<ListTodosQuery>,
) -> Result<ListTodosResponse, AppError> {
    Err(AppError::TodoRepo(TodoRepoError::NotFound))
}
