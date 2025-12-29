use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::world::WorldTime;

#[derive(Clone)]
pub struct TimeThread {
    time: Arc<Mutex<WorldTime>>,
}

impl TimeThread {
    pub fn new(initial_time: WorldTime, game_speed: f32) -> Self {
        let time = Arc::new(Mutex::new(initial_time));
        let time_clone = Arc::clone(&time);
        
        let _ = thread::spawn(move || {
            loop {               
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
        }
    }
    
    pub fn get_time(&self) -> WorldTime {
        self.time.lock().unwrap().clone()
    }
    
    pub fn set_time(&self, new_time: WorldTime) {
        let mut time = self.time.lock().unwrap();
        *time = new_time;
    }
}