use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 一个用于并行任务处理的简单任务执行器
#[allow(dead_code)]
pub struct TaskExecutor<T: Clone> {
    tasks: Arc<RwLock<HashMap<String, TaskStatus<T>>>>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum TaskStatus<T: Clone> {
    Running,
    Completed(T),
    Failed(String),
}

#[allow(dead_code)]
impl<T: Clone + Send + Sync + 'static> TaskExecutor<T> {
    pub fn new() -> Self {
        TaskExecutor {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn submit<F>(&self, future: F) -> String
    where
        F: std::future::Future<Output = T> + Send + 'static,
    {
        let task_id = Uuid::new_v4().to_string();
        let task_id_clone = task_id.clone();
        let tasks = self.tasks.clone();

        // 将任务标记为运行中
        {
            let mut t = tasks.write().await;
            t.insert(task_id_clone.clone(), TaskStatus::Running);
        }

        // 在后台启动异步任务
        tokio::spawn(async move {
            let result = future.await;
            let mut t = tasks.write().await;
            t.insert(task_id_clone, TaskStatus::Completed(result));
        });

        task_id
    }

    pub async fn get_status(&self, task_id: &str) -> Option<TaskStatus<T>> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }
}

impl<T: Clone + Send + Sync + 'static> Default for TaskExecutor<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_execution() {
        let executor = TaskExecutor::new();
        let task_id = executor.submit(async { 42 }).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        assert!(matches!(
            executor.get_status(&task_id).await,
            Some(TaskStatus::Completed(42))
        ));
    }
}
