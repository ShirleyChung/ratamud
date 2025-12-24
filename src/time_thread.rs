use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::world::WorldTime;

pub struct TimeThread {
    time: Arc<Mutex<WorldTime>>,
    handle: Option<thread::JoinHandle<()>>,
    should_stop: Arc<Mutex<bool>>,
}

impl TimeThread {
    pub fn new(initial_time: WorldTime, game_speed: f32) -> Self {
        let time = Arc::new(Mutex::new(initial_time));
        let time_clone = Arc::clone(&time);
        let should_stop = Arc::new(Mutex::new(false));
        let should_stop_clone = Arc::clone(&should_stop);
        
        let handle = thread::spawn(move || {
            loop {
                // 檢查是否應該停止
                if *should_stop_clone.lock().unwrap() {
                    break;
                }
                
                // 更新時間
                {
                    let mut time = time_clone.lock().unwrap();
                    time.advance(game_speed);
                }
                
                // 每秒更新一次
                thread::sleep(Duration::from_millis(1000));
            }
        });
        
        TimeThread {
            time,
            handle: Some(handle),
            should_stop,
        }
    }
    
    pub fn get_time(&self) -> WorldTime {
        self.time.lock().unwrap().clone()
    }
    
    pub fn set_time(&self, new_time: WorldTime) {
        let mut time = self.time.lock().unwrap();
        *time = new_time;
    }
    
    pub fn stop(&mut self) {
        *self.should_stop.lock().unwrap() = true;
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for TimeThread {
    fn drop(&mut self) {
        self.stop();
    }
}
