use crate::event::{EventManager, GameEvent, TriggerType};
use crate::world::GameWorld;
use crate::person::Person;
use rand::Rng;

/// Crontab 解析器
pub struct CronParser;

impl CronParser {
    /// 解析 crontab 格式的時間表達式
    /// 格式: "分 時 日 月 星期"
    /// 支持: * (任意), */N (每N), N (具體值), N-M (範圍)
    pub fn matches(schedule: &str, minute: u8, hour: u8, _day: u32) -> bool {
        let parts: Vec<&str> = schedule.split_whitespace().collect();
        if parts.len() < 2 {
            return false;
        }
        
        let minute_match = Self::matches_field(parts[0], minute as u32, 0, 59);
        let hour_match = Self::matches_field(parts[1], hour as u32, 0, 23);
        
        // 簡化版：暫時不考慮日月星期
        minute_match && hour_match
    }
    
    fn matches_field(pattern: &str, value: u32, _min: u32, _max: u32) -> bool {
        if pattern == "*" {
            return true;
        }
        
        // 處理 */N 格式（每N）
        if let Some(stripped) = pattern.strip_prefix("*/") {
            if let Ok(interval) = stripped.parse::<u32>() {
                return value.is_multiple_of(interval);
            }
        }
        
        // 處理範圍 N-M
        if pattern.contains('-') {
            let range: Vec<&str> = pattern.split('-').collect();
            if range.len() == 2 {
                if let (Ok(start), Ok(end)) = (range[0].parse::<u32>(), range[1].parse::<u32>()) {
                    return value >= start && value <= end;
                }
            }
        }
        
        // 處理具體值
        if let Ok(target) = pattern.parse::<u32>() {
            return value == target;
        }
        
        false
    }
}

/// 事件調度器
pub struct EventScheduler {
    pub last_check_time: (u32, u8, u8),  // (day, hour, minute)
}

impl EventScheduler {
    pub fn new() -> Self {
        EventScheduler {
            last_check_time: (0, 0, 0),
        }
    }
    
    /// 檢查並觸發時間相關事件
    #[allow(dead_code)]
    pub fn check_and_trigger(
        &mut self,
        event_manager: &mut EventManager,
        game_world: &GameWorld,
        player: &Person,
    ) -> Vec<String> {
        let mut triggered_events = Vec::new();
        
        let current_day = game_world.time.day;
        let current_hour = game_world.time.hour;
        let current_minute = game_world.time.minute;
        
        // 檢查是否進入新的分鐘
        if (current_day, current_hour, current_minute) == self.last_check_time {
            return triggered_events;
        }
        
        self.last_check_time = (current_day, current_hour, current_minute);
        
        // 先收集所有事件的克隆，避免借用衝突
        let events: Vec<GameEvent> = event_manager.list_events()
            .iter()
            .map(|e| (*e).clone())
            .collect();
        
        // 遍歷所有事件
        for event in events {
            let event_id = event.id.clone();
            
            // 檢查運行時狀態
            if let Some(runtime_state) = event_manager.get_runtime_state(&event_id) {
                if !event.can_trigger(runtime_state) {
                    continue;
                }
            }
            
            // 檢查觸發條件
            if self.check_trigger(&event, game_world) {
                // 檢查條件（人事時地物）
                if self.check_conditions(&event, game_world, player) {
                    triggered_events.push(event_id.clone());
                    event_manager.trigger_event(&event_id);
                }
            }
        }
        
        triggered_events
    }
    
    /// 檢查觸發器是否滿足
    pub fn check_trigger(&self, event: &GameEvent, game_world: &GameWorld) -> bool {
        match &event.trigger {
            TriggerType::Time { schedule, random_chance, day_range, time_range } => {
                // 檢查 crontab 時間表達式
                if !CronParser::matches(
                    schedule,
                    game_world.time.minute,
                    game_world.time.hour,
                    game_world.time.day
                ) {
                    return false;
                }
                
                // 檢查天數範圍
                if let Some([start_day, end_day]) = day_range {
                    if game_world.time.day < *start_day || game_world.time.day > *end_day {
                        return false;
                    }
                }
                
                // 檢查時間範圍
                if let Some([start_time, end_time]) = time_range {
                    let current_time = format!("{:02}:{:02}:{:02}", 
                        game_world.time.hour, 
                        game_world.time.minute,
                        game_world.time.second);
                    
                    if current_time < *start_time || current_time > *end_time {
                        return false;
                    }
                }
                
                // 檢查隨機機率
                if let Some(chance) = random_chance {
                    let mut rng = rand::thread_rng();
                    if rng.gen::<f32>() > *chance {
                        return false;
                    }
                }
                
                true
            }
            TriggerType::Random { chance, .. } => {
                let mut rng = rand::thread_rng();
                rng.gen::<f32>() <= *chance
            }
            TriggerType::Location { positions } => {
                // 需要在移動時檢查，這裡返回 false
                positions.is_empty()  // 暫時不在此處檢查
            }
            TriggerType::Condition { .. } => {
                // 條件觸發需要額外的條件檢查器
                false
            }
            TriggerType::Manual => false,
        }
    }
    
    /// 檢查事件條件（人事時地物）
    pub fn check_conditions(&self, event: &GameEvent, game_world: &GameWorld, player: &Person) -> bool {
        // 檢查地點條件（Where）
        if let Some(map_name) = &event.r#where.map {
            if game_world.current_map_name != *map_name {
                return false;
            }
        }
        
        if let Some(positions) = &event.r#where.positions {
            let player_pos = (player.x, player.y);
            let mut found = false;
            for pos in positions {
                if pos[0] == player_pos.0 && pos[1] == player_pos.1 {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        
        if let Some(area) = &event.r#where.area {
            let in_x_range = player.x >= area.x[0] && player.x <= area.x[1];
            let in_y_range = player.y >= area.y[0] && player.y <= area.y[1];
            if !in_x_range || !in_y_range {
                return false;
            }
        }
        
        // 檢查物品條件（What）
        if let Some(required_items) = &event.what.required_items {
            for item in required_items {
                if player.get_item_count(item) == 0 {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// 檢查位置觸發
    #[allow(dead_code)]
    pub fn check_location_trigger(
        &self,
        event_manager: &EventManager,
        player_x: usize,
        player_y: usize,
    ) -> Vec<String> {
        let mut triggered_events = Vec::new();
        
        for event in event_manager.list_events() {
            if let TriggerType::Location { positions } = &event.trigger {
                for pos in positions {
                    if pos[0] == player_x && pos[1] == player_y {
                        if let Some(runtime_state) = event_manager.get_runtime_state(&event.id) {
                            if event.can_trigger(runtime_state) {
                                triggered_events.push(event.id.clone());
                            }
                        }
                    }
                }
            }
        }
        
        triggered_events
    }
}

impl Default for EventScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cron_parser() {
        // 每10分鐘
        assert!(CronParser::matches("*/10 * * * *", 0, 12, 1));
        assert!(CronParser::matches("*/10 * * * *", 10, 12, 1));
        assert!(CronParser::matches("*/10 * * * *", 20, 12, 1));
        assert!(!CronParser::matches("*/10 * * * *", 5, 12, 1));
        
        // 每小時整點
        assert!(CronParser::matches("0 * * * *", 0, 12, 1));
        assert!(!CronParser::matches("0 * * * *", 30, 12, 1));
        
        // 9-17點的整點
        assert!(CronParser::matches("0 9-17 * * *", 0, 9, 1));
        assert!(CronParser::matches("0 9-17 * * *", 0, 12, 1));
        assert!(CronParser::matches("0 9-17 * * *", 0, 17, 1));
        assert!(!CronParser::matches("0 9-17 * * *", 0, 8, 1));
        assert!(!CronParser::matches("0 9-17 * * *", 0, 18, 1));
    }
}
