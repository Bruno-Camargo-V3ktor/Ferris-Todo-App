use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Form, Router,
};
use models::{Todo, TodoListFilter, TodoToggleAction};
use repository::{TodoRepo, TodoRepoError};
use tokio::sync::RwLock;
use tower_http::{services::ServeDir, trace::TraceLayer};
use uuid::Uuid;

pub mod models;
pub mod repository;

// Types
pub type SharedState = Arc<RwLock<AppState>>;

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

struct ToggleCompletedTodosResponse {
    num_completed_items: u32,
    num_active_items: u32,
    num_all_items: u32,
    is_disabled_delete: bool,
    is_disabled_toggle: bool,
    action: TodoToggleAction,
    items: Vec<Todo>,
}

struct ToggleCompletedTodosQuery {
    action: TodoToggleAction,
}

struct DeletedCompletedTodosResponse {
    num_completed_items: u32,
    num_active_items: u32,
    num_all_items: u32,
    is_disabled_delete: bool,
    is_disabled_toggle: bool,
    action: TodoToggleAction,
    items: Vec<Todo>,
}

struct EditTodoResponse {
    item: Todo,
}

struct UpdateTodoResponse {
    num_completed_items: u32,
    num_active_items: u32,
    num_all_items: u32,
    is_disabled_delete: bool,
    is_disabled_toggle: bool,
    action: TodoToggleAction,
    item: Option<Todo>,
}

struct UpdateTodoForm {
    is_completed: Option<bool>,
    text: Option<String>,
}

struct DeleteTodoResponse {
    num_completed_items: u32,
    num_active_items: u32,
    num_all_items: u32,
    is_disabled_delete: bool,
    is_disabled_toggle: bool,
    action: TodoToggleAction,
}

struct CreateTodoResponse {
    num_completed_items: u32,
    num_active_items: u32,
    num_all_items: u32,
    is_disabled_delete: bool,
    is_disabled_toggle: bool,
    action: TodoToggleAction,
    item: Option<Todo>,
}

struct CreateTodoForm {
    text: String,
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
        .route("/", get(get_index))
        .route(
            "/todo",
            get(list_todos)
                .post(create_todo)
                .patch(toggle_completed_todos)
                .delete(delete_completed_todos),
        )
        .route(
            "/todo/:id",
            get(edit_todo).patch(update_todo).delete(delete_todo),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state)
}

async fn get_index() -> Result<GetIndexResponse, AppError> {
    Ok(GetIndexResponse)
}

async fn list_todos(
    State(shared_state): State<SharedState>,
    Query(ListTodosQuery { filter }): Query<ListTodosQuery>,
) -> Result<ListTodosResponse, AppError> {
    shared_state.write().await.selected_filter = filter;
    let state = shared_state.read().await;
    let items = state.todo_repo.list(&filter);

    Ok(ListTodosResponse {
        num_completed_items: state.todo_repo.num_completed_items,
        num_active_items: state.todo_repo.num_active_items,
        num_all_items: state.todo_repo.num_all_items,
        is_disabled_delete: state.todo_repo.num_completed_items == 0,
        is_disabled_toggle: state.todo_repo.num_all_items == 0,
        action: state.toggle_action,
        items,
    })
}

async fn toggle_completed_todos(
    State(shared_state): State<SharedState>,
    Query(ToggleCompletedTodosQuery { action }): Query<ToggleCompletedTodosQuery>,
) -> Result<ToggleCompletedTodosResponse, AppError> {
    let mut state = shared_state.write().await;
    state.toggle_action = match action {
        TodoToggleAction::Uncheck => TodoToggleAction::Check,
        TodoToggleAction::Check => TodoToggleAction::Uncheck,
    };

    state.todo_repo.toggle_completed(&action);
    let items = state.todo_repo.list(&state.selected_filter);

    Ok(ToggleCompletedTodosResponse {
        num_completed_items: state.todo_repo.num_completed_items,
        num_active_items: state.todo_repo.num_active_items,
        num_all_items: state.todo_repo.num_all_items,
        is_disabled_delete: state.todo_repo.num_completed_items == 0,
        is_disabled_toggle: state.todo_repo.num_all_items == 0,
        action: state.toggle_action,
        items,
    })
}

async fn delete_completed_todos(
    State(shared_state): State<SharedState>,
) -> Result<DeletedCompletedTodosResponse, AppError> {
    let mut state = shared_state.write().await;

    state.todo_repo.delete_completed();
    state.toggle_action = TodoToggleAction::Check;
    let items = state.todo_repo.list(&state.selected_filter);

    Ok(DeletedCompletedTodosResponse {
        num_completed_items: state.todo_repo.num_completed_items,
        num_active_items: state.todo_repo.num_active_items,
        num_all_items: state.todo_repo.num_all_items,
        is_disabled_delete: true,
        is_disabled_toggle: state.todo_repo.num_all_items == 0,
        action: state.toggle_action,
        items,
    })
}

async fn edit_todo(
    State(shared_state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<EditTodoResponse, AppError> {
    let item = shared_state.read().await.todo_repo.get(&id)?;

    Ok(EditTodoResponse { item })
}

async fn update_todo(
    State(shared_state): State<SharedState>,
    Path(id): Path<Uuid>,
    Form(todo_update): Form<UpdateTodoForm>,
) -> Result<UpdateTodoResponse, AppError> {
    let mut state = shared_state.write().await;
    let item = state
        .todo_repo
        .update(&id, todo_update.text, todo_update.is_completed)?;

    state.toggle_action = if state.todo_repo.num_completed_items == state.todo_repo.num_all_items {
        TodoToggleAction::Uncheck
    } else {
        TodoToggleAction::Check
    };

    let item = match state.selected_filter {
        TodoListFilter::Active if item.is_completed => None,
        TodoListFilter::Active | TodoListFilter::All => Some(item),
        TodoListFilter::Completed if item.is_completed => Some(item),
        TodoListFilter::Completed => None,
    };

    Ok(UpdateTodoResponse {
        num_completed_items: state.todo_repo.num_completed_items,
        num_active_items: state.todo_repo.num_active_items,
        num_all_items: state.todo_repo.num_all_items,
        is_disabled_delete: state.todo_repo.num_completed_items == 0,
        is_disabled_toggle: state.todo_repo.num_all_items == 0,
        action: state.toggle_action,
        item,
    })
}

async fn delete_todo(
    State(shared_state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<DeleteTodoResponse, AppError> {
    let mut state = shared_state.write().await;
    state.todo_repo.delete(&id)?;

    state.toggle_action = if state.todo_repo.num_all_items == 0 {
        TodoToggleAction::Check
    } else {
        TodoToggleAction::Uncheck
    };

    Ok(DeleteTodoResponse {
        num_completed_items: state.todo_repo.num_completed_items,
        num_active_items: state.todo_repo.num_active_items,
        num_all_items: state.todo_repo.num_all_items,
        is_disabled_delete: state.todo_repo.num_completed_items == 0,
        is_disabled_toggle: state.todo_repo.num_all_items == 0,
        action: state.toggle_action,
    })
}

async fn create_todo(
    State(shared_state): State<SharedState>,
    Form(CreateTodoForm { text }): Form<CreateTodoForm>,
) -> Result<CreateTodoResponse, AppError> {
    let mut state = shared_state.write().await;
    let item = state.todo_repo.create(&text);

    let item = if state.selected_filter == TodoListFilter::Completed {
        None
    } else {
        Some(item)
    };

    state.toggle_action = TodoToggleAction::Check;
    Ok(CreateTodoResponse {
        num_completed_items: state.todo_repo.num_completed_items,
        num_active_items: state.todo_repo.num_active_items,
        num_all_items: state.todo_repo.num_all_items,
        is_disabled_delete: state.todo_repo.num_completed_items == 0,
        is_disabled_toggle: state.todo_repo.num_all_items == 0,
        action: state.toggle_action,
        item,
    })
}
