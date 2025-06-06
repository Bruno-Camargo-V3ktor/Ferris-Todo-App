use crate::models::{Todo, TodoListFilter, TodoToggleAction};
use std::collections::HashMap;
use uuid::Uuid;

// Enums
#[derive(Debug, PartialEq, Eq)]
pub enum TodoRepoError {
    NotFound,
}

// Structs
#[derive(Debug, Default)]
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

    pub fn create(&mut self, text: impl Into<String>) -> Todo {
        let todo = Todo::new(text);
        self.items.insert(todo.id, todo.clone());

        self.num_active_items += 1;
        self.num_all_items += 1;

        todo
    }

    pub fn delete(&mut self, id: &Uuid) -> Result<(), TodoRepoError> {
        let old_todo = self.items.remove(id).ok_or(TodoRepoError::NotFound)?;

        self.num_all_items -= 1;
        if old_todo.is_completed {
            self.num_completed_items -= 1;
        } else {
            self.num_active_items -= 1;
        }

        Ok(())
    }

    pub fn update(
        &mut self,
        id: &Uuid,
        text: Option<String>,
        is_completed: Option<bool>,
    ) -> Result<Todo, TodoRepoError> {
        let todo = self.items.get_mut(id).ok_or(TodoRepoError::NotFound)?;

        if let Some(completed) = is_completed {
            if todo.is_completed != completed {
                todo.is_completed = completed;
                if completed {
                    self.num_completed_items += 1;
                    self.num_active_items -= 1;
                } else {
                    self.num_completed_items -= 1;
                    self.num_active_items += 1;
                }
            }
        }

        if let Some(text) = text {
            todo.text = text;
        }

        Ok(todo.clone())
    }

    pub fn delete_completed(&mut self) {
        self.items.retain(|_, todo| !todo.is_completed);
        self.num_all_items -= self.num_completed_items;
        self.num_completed_items = 0;
    }

    pub fn toggle_completed(&mut self, action: &TodoToggleAction) {
        let is_completed: bool = match action {
            TodoToggleAction::Check => {
                self.num_active_items = 0;
                self.num_completed_items = self.num_all_items;
                true
            }
            TodoToggleAction::Uncheck => {
                self.num_completed_items = 0;
                self.num_active_items = self.num_all_items;
                false
            }
        };

        for todo in self.items.values_mut() {
            todo.is_completed = is_completed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_non_existing_todo() {
        let repo = TodoRepo::default();

        let result_todo = repo.get(&Uuid::new_v4());

        assert_eq!(result_todo, Err(TodoRepoError::NotFound));
    }

    #[test]
    fn test_get_existing_todo() {
        let mut repo = TodoRepo::default();
        let todo = repo.create("Teste");

        assert_eq!(Ok(todo.clone()), repo.get(&todo.id));
    }

    #[test]
    fn test_list_repo_empty() {
        let repo = TodoRepo::default();
        let empty_list: Vec<Todo> = Vec::new();

        let result_completed = repo.list(&TodoListFilter::Completed);
        let result_active = repo.list(&TodoListFilter::Active);
        let result_all = repo.list(&TodoListFilter::All);

        assert_eq!(empty_list, result_completed);
        assert_eq!(empty_list, result_active);
        assert_eq!(empty_list, result_all);
    }

    #[test]
    fn test_list_filled_repo_active() {
        let mut repo = TodoRepo::default();
        let mut filled = vec![
            repo.create("Task A"),
            repo.create("Task B"),
            repo.create("Task C"),
        ];

        filled.reverse();

        assert_eq!(filled, repo.list(&TodoListFilter::Active));
        assert_eq!(Vec::<Todo>::new(), repo.list(&TodoListFilter::Completed));
    }

    #[test]
    fn test_list_filled_repo_complete() {
        let mut repo = TodoRepo::default();
        let mut filled = vec![
            repo.create("Task A"),
            repo.create("Task B"),
            repo.create("Task C"),
        ];
        filled.reverse();

        for t in filled.iter_mut() {
            let _ = repo.update(&t.id, None, Some(true));
            t.is_completed = true;
        }

        assert_eq!(filled, repo.list(&TodoListFilter::Completed));
        assert_eq!(Vec::<Todo>::new(), repo.list(&TodoListFilter::Active));
        assert_eq!(
            repo.list(&TodoListFilter::All),
            repo.list(&TodoListFilter::Completed)
        );
    }

    #[test]
    fn test_repo_propreties() {
        let mut repo = TodoRepo::default();
        let mut todos = vec![repo.create("Task A"), repo.create("Task B")];
        todos.reverse();

        assert_eq!(todos, repo.list(&TodoListFilter::All));
        assert_eq!(todos, repo.list(&TodoListFilter::Active));
        assert_eq!(Vec::<Todo>::new(), repo.list(&TodoListFilter::Completed));
        assert_eq!(2, repo.num_all_items);
        assert_eq!(0, repo.num_completed_items);
        assert_eq!(2, repo.num_active_items);
    }

    #[test]
    fn test_delete_non_existing_todo() {
        let mut repo = TodoRepo::default();

        let delete_todo = repo.delete(&Uuid::new_v4());
        assert_eq!(delete_todo, Err(TodoRepoError::NotFound));
    }

    #[test]
    fn test_delete_one_todo() {
        let mut repo = TodoRepo::default();
        let todo = repo.create("Task A");

        let result = repo.delete(&todo.id);
        assert_eq!(result, Ok(()));
        assert_eq!(0, repo.num_all_items);
    }

    #[test]
    fn test_update_non_existing_todo() {
        let mut repo = TodoRepo::default();
        let result = repo.update(&Uuid::new_v4(), Some("Task A".into()), None);

        assert_eq!(result, Err(TodoRepoError::NotFound));
    }

    #[test]
    fn test_update_one_existing_todo() {
        let mut repo = TodoRepo::default();
        let old_todo = repo.create("Task A");
        let new_todo = repo
            .update(&old_todo.id, Some("Task AB".into()), None)
            .unwrap();

        assert_ne!(old_todo, new_todo);
    }

    #[test]
    fn test_update_is_completed_true_existing_todo() {
        let mut repo = TodoRepo::default();

        let todo = repo.create("Task A");
        let result = repo.update(&todo.id, None, Some(true)).unwrap();

        assert_eq!(1, repo.num_completed_items);
        assert_eq!(0, repo.num_active_items);
        assert_eq!(1, repo.num_all_items);
        assert!(result.is_completed);
    }

    #[test]
    fn test_update_is_completed_false_existing_todo() {
        let mut repo = TodoRepo::default();

        let todo = repo.create("Task A");
        let result = repo.update(&todo.id, None, Some(true)).unwrap();
        assert!(result.is_completed);

        let result = repo.update(&todo.id, None, Some(false)).unwrap();

        assert_eq!(0, repo.num_completed_items);
        assert_eq!(1, repo.num_active_items);
        assert_eq!(1, repo.num_all_items);
        assert!(!result.is_completed);
    }

    #[test]
    fn test_delete_completed_todos() {
        let mut repo = TodoRepo::default();
        let task_a = repo.create("Task A");
        let task_b = repo.create("Task B");
        let task_c = repo.create("Task C");

        let _task_a = repo.update(&task_a.id, None, Some(true)).unwrap();
        let _task_c = repo.update(&task_c.id, None, Some(true)).unwrap();

        repo.delete_completed();
        assert_eq!(0, repo.num_completed_items);
        assert_eq!(1, repo.num_all_items);
        assert_eq!(1, repo.num_active_items);
        assert_eq!(vec![task_b.clone()], repo.list(&TodoListFilter::All));
    }
}
