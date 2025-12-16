use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct NpcAiThread {
    handle: Option<thread::JoinHandle<()>>,
    should_stop: Arc<Mutex<bool>>,
    ai_logs: Arc<Mutex<Vec<String>>>,
}

impl NpcAiThread {
    /// 創建新的 NPC AI 執行緒
    /// update_interval_ms: AI 更新間隔（毫秒）
    pub fn new<F>(mut update_fn: F, update_interval_ms: u64) -> Self 
    where
        F: FnMut() -> Vec<String> + Send + 'static,
    {
        let should_stop = Arc::new(Mutex::new(false));
        let should_stop_clone = Arc::clone(&should_stop);
        let ai_logs = Arc::new(Mutex::new(Vec::new()));
        let ai_logs_clone = Arc::clone(&ai_logs);
        
        let handle = thread::spawn(move || {
            loop {
                // 檢查是否應該停止
                if *should_stop_clone.lock().unwrap() {
                    break;
                }
                
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
            handle: Some(handle),
            should_stop,
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
    
    /// 停止 AI 執行緒
    pub fn stop(&mut self) {
        *self.should_stop.lock().unwrap() = true;
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for NpcAiThread {
    fn drop(&mut self) {
        self.stop();
    }
}
