use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 事件檢查執行緒
pub struct EventCheckThread {
    handle: Option<thread::JoinHandle<()>>,
    should_stop: Arc<Mutex<bool>>,
    event_logs: Arc<Mutex<Vec<String>>>,
    triggered_events: Arc<Mutex<Vec<String>>>,
}

impl EventCheckThread {
    /// 創建新的事件檢查執行緒
    /// check_fn: 事件檢查函數，返回 (日誌列表, 觸發的事件ID列表)
    /// check_interval_ms: 檢查間隔（毫秒）
    pub fn new<F>(mut check_fn: F, check_interval_ms: u64) -> Self
    where
        F: FnMut() -> (Vec<String>, Vec<String>) + Send + 'static,
    {
        let should_stop = Arc::new(Mutex::new(false));
        let should_stop_clone = Arc::clone(&should_stop);
        let event_logs = Arc::new(Mutex::new(Vec::new()));
        let event_logs_clone = Arc::clone(&event_logs);
        let triggered_events = Arc::new(Mutex::new(Vec::new()));
        let triggered_events_clone = Arc::clone(&triggered_events);
        
        let handle = thread::spawn(move || {
            loop {
                // 檢查是否應該停止
                if *should_stop_clone.lock().unwrap() {
                    break;
                }
                
                // 執行事件檢查
                let (logs, events) = check_fn();
                
                // 將日誌存儲到共享佇列
                if !logs.is_empty() {
                    let mut event_logs = event_logs_clone.lock().unwrap();
                    event_logs.extend(logs);
                }
                
                // 將觸發的事件存儲到共享佇列
                if !events.is_empty() {
                    let mut triggered = triggered_events_clone.lock().unwrap();
                    triggered.extend(events);
                }
                
                // 休眠指定時間
                thread::sleep(Duration::from_millis(check_interval_ms));
            }
        });
        
        EventCheckThread {
            handle: Some(handle),
            should_stop,
            event_logs,
            triggered_events,
        }
    }
    
    /// 獲取並清空事件日誌
    pub fn get_logs(&self) -> Vec<String> {
        let mut logs = self.event_logs.lock().unwrap();
        let result = logs.clone();
        logs.clear();
        result
    }
    
    /// 獲取並清空觸發的事件ID
    pub fn get_triggered_events(&self) -> Vec<String> {
        let mut events = self.triggered_events.lock().unwrap();
        let result = events.clone();
        events.clear();
        result
    }
    
    /// 停止事件檢查執行緒
    pub fn stop(&mut self) {
        *self.should_stop.lock().unwrap() = true;
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for EventCheckThread {
    fn drop(&mut self) {
        self.stop();
    }
}
