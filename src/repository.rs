use std::collections::HashMap;

use uuid::Uuid;

use crate::models::{Todo, TodoListFilter};

// Enums
pub enum TodoRepoError {
    NotFound,
}

// Structs
pub struct TodoRepo {
    pub num_completed_items: u32,
    pub num_active_items: u32,
    pub num_all_items: u32,
    pub items: HashMap<Uuid, Todo>,
}

// Impls
impl TodoRepo {
    pub fn get(&self, id: &Uuid) -> Result<Todo, TodoRepoError> {
        self.items.get(id).ok_or(TodoRepoError::NotFound).cloned()
    }

    pub fn list(&self, filter: &TodoListFilter) -> Vec<Todo> {
        let mut todos: Vec<_> = self
            .items
            .values()
            .filter(|t| match filter {
                TodoListFilter::All => true,
                TodoListFilter::Completed => t.is_completed,
                TodoListFilter::Active => !t.is_completed,
            })
            .cloned()
            .collect();

        todos.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        todos
    }
}
