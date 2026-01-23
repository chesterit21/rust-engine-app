use std::collections::VecDeque;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub document_id: i32,
    pub priority: TaskPriority,
    pub retry_count: u32,
}

pub struct TaskQueue {
    queue: Mutex<VecDeque<Task>>,
    max_size: usize,
}

impl TaskQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            max_size,
        }
    }
    
    /// Enqueue task (sorted by priority)
    pub async fn enqueue(&self, task: Task) {
        let mut queue = self.queue.lock().await;
        
        // Check if document already in queue
        if queue.iter().any(|t| t.document_id == task.document_id) {
            return; // Skip duplicate
        }
        
        // Check max size
        if queue.len() >= self.max_size {
            // Remove lowest priority task
            if let Some(pos) = queue.iter().position(|t| t.priority == TaskPriority::Low) {
                queue.remove(pos);
            }
        }
        
        // Insert based on priority
        let insert_pos = queue
            .iter()
            .position(|t| t.priority < task.priority)
            .unwrap_or(queue.len());
        
        queue.insert(insert_pos, task);
    }
    
    /// Dequeue next task
    pub async fn dequeue(&self) -> Option<Task> {
        let mut queue = self.queue.lock().await;
        queue.pop_front()
    }
    
    /// Get queue size
    pub async fn size(&self) -> usize {
        let queue = self.queue.lock().await;
        queue.len()
    }
}
