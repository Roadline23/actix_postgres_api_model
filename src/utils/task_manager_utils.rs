use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

pub struct TaskManager {
    tasks: Arc<Mutex<HashMap<i64, JoinHandle<()>, RandomState>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_task(&self, task_id: i64, task: JoinHandle<()>) -> Result<(), String> {
        let mut tasks = match self.tasks.lock() {
            Ok(t) => t,
            Err(_) => return Err("Mutex lock error".to_string()),
        };

        if tasks.contains_key(&task_id) {
            Err("Task with the same ID already exists".to_string())
        } else {
            tasks.insert(task_id, task);
            Ok(())
        }
    }

    pub fn cancel_task(&self, task_id: i64) -> Result<(), String> {
        let mut tasks = match self.tasks.lock() {
            Ok(t) => t,
            Err(_) => return Err("Mutex lock error".to_string()),
        };

        if let Some(task) = tasks.remove(&task_id) {
            task.abort();
            Ok(())
        } else {
            Err("Task not found".to_string())
        }
    }
}
