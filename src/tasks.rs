use tokio::task::JoinHandle;
use std::future::Future;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Task status enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Generic task type for parallel processing
pub struct Task<T: Send + 'static> {
    pub id: String,
    pub status: TaskStatus,
    pub result: Option<T>,
    pub error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl<T: Send + 'static> Task<T> {
    pub fn new() -> Self {
        Task {
            id: Uuid::new_v4().to_string(),
            status: TaskStatus::Pending,
            result: None,
            error: None,
            created_at: chrono::Utc::now(),
        }
    }
}

/// Task executor for concurrent operations
pub struct TaskExecutor<T: Send + 'static> {
    tasks: Arc<RwLock<HashMap<String, Task<T>>>>,
    active_handles: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
}

impl<T: Send + Sync + 'static> TaskExecutor<T> {
    pub fn new() -> Self {
        TaskExecutor {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            active_handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Submit a task for async execution
    pub async fn submit<F>(&self, future: F) -> String
    where
        F: Future<Output = T> + Send + 'static,
    {
        let task_id = Uuid::new_v4().to_string();
        let task = Task::new();
        let task_id_clone = task_id.clone();
        let tasks = self.tasks.clone();

        // Register task
        {
            let mut tasks_lock = tasks.write().await;
            tasks_lock.insert(task_id.clone(), task);
        }

        // Spawn async task
        let handle = tokio::spawn(async move {
            // Update status to Running
            {
                let mut tasks_lock = tasks.write().await;
                if let Some(task) = tasks_lock.get_mut(&task_id_clone) {
                    task.status = TaskStatus::Running;
                }
            }

            // Execute the future
            let result = future.await;

            // Update with result
            {
                let mut tasks_lock = tasks.write().await;
                if let Some(task) = tasks_lock.get_mut(&task_id_clone) {
                    task.result = Some(result);
                    task.status = TaskStatus::Completed;
                }
            }
        });

        let mut handles = self.active_handles.write().await;
        handles.insert(task_id.clone(), handle);

        task_id
    }

    /// Get task status
    pub async fn get_status(&self, task_id: &str) -> Option<TaskStatus> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).map(|t| t.status.clone())
    }

    /// Wait for multiple tasks in parallel
    pub async fn wait_all(&self, task_ids: Vec<String>) -> Vec<Option<TaskStatus>> {
        futures::future::join_all(
            task_ids
                .iter()
                .map(|id| async {
                    let tasks = self.tasks.read().await;
                    tasks.get(id).map(|t| t.status.clone())
                })
                .collect::<Vec<_>>(),
        )
        .await
    }

    /// Cancel a running task
    pub async fn cancel(&self, task_id: &str) -> bool {
        let mut handles = self.active_handles.write().await;
        if let Some(handle) = handles.remove(task_id) {
            handle.abort();
            return true;
        }
        false
    }
}

/// Batch task processor for handling multiple independent operations
pub struct BatchProcessor<T: Send + 'static> {
    executor: TaskExecutor<T>,
}

impl<T: Send + Sync + 'static> BatchProcessor<T> {
    pub fn new() -> Self {
        BatchProcessor {
            executor: TaskExecutor::new(),
        }
    }

    /// Process multiple futures concurrently
    pub async fn process_batch<F>(&self, futures: Vec<F>) -> Vec<String>
    where
        F: Future<Output = T> + Send + 'static,
    {
        let mut task_ids = Vec::new();
        for future in futures {
            let task_id = self.executor.submit(future).await;
            task_ids.push(task_id);
        }
        task_ids
    }

    /// Wait for all tasks to complete
    pub async fn wait_all_complete(&self, task_ids: Vec<String>) {
        loop {
            let statuses = self.executor.wait_all(task_ids.clone()).await;
            if statuses.iter().all(|s| {
                matches!(s, Some(TaskStatus::Completed) | Some(TaskStatus::Failed))
            }) {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_execution() {
        let executor = TaskExecutor::<i32>::new();
        let task_id = executor.submit(async { 42 }).await;

        // Wait a bit for task to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let status = executor.get_status(&task_id).await;
        assert_eq!(status, Some(TaskStatus::Completed));
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let processor = BatchProcessor::<i32>::new();
        let futures = vec![
            async { 1 },
            async { 2 },
            async { 3 },
            async { 4 },
            async { 5 },
        ];

        let task_ids = processor.process_batch(futures).await;
        assert_eq!(task_ids.len(), 5);

        processor.wait_all_complete(task_ids).await;
    }
}
