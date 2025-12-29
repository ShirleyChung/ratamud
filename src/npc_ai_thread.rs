use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct NpcAiThread {
    ai_logs: Arc<Mutex<Vec<String>>>,
}

impl NpcAiThread {
    /// 創建新的 NPC AI 執行緒
    /// update_interval_ms: AI 更新間隔（毫秒）
    pub fn new<F>(mut update_fn: F, update_interval_ms: u64) -> Self 
    where
        F: FnMut() -> Vec<String> + Send + 'static,
    {
        let ai_logs = Arc::new(Mutex::new(Vec::new()));
        let ai_logs_clone = Arc::clone(&ai_logs);
        
        let _ = thread::spawn(move || {
            loop {               
                // 執行 NPC AI 更新
                let logs = update_fn();
                
                // 將日誌存儲到共享佇列
                if !logs.is_empty() {
                    let mut ai_logs = ai_logs_clone.lock().unwrap();
                    ai_logs.extend(logs);
                }
                
                // 休眠指定時間
                thread::sleep(Duration::from_millis(update_interval_ms));
            }
        });
        
        NpcAiThread {
            ai_logs,
        }
    }
    
    /// 獲取並清空 AI 日誌
    pub fn get_logs(&self) -> Vec<String> {
        let mut logs = self.ai_logs.lock().unwrap();
        let result = logs.clone();
        logs.clear();
        result
    }    
}