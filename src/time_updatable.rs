/// TimeUpdatable trait 定義時間更新事件
/// 實現此 trait 的物件能接收並處理時間更新事件
pub trait TimeUpdatable {
    /// 當世界時間更新時被調用
    fn on_time_update(&mut self, current_time: &TimeInfo);
}

/// 時間信息結構體，包含當前遊戲時間
#[derive(Debug, Clone, Copy)]
pub struct TimeInfo {
    pub hour: u8,      // 0-23
    pub minute: u8,    // 0-59
    pub second: u8,    // 0-59
    pub day: u32,      // 遊戲日期
}

impl TimeInfo {
    pub fn new(hour: u8, minute: u8, day: u32) -> Self {
        TimeInfo { hour, minute, second: 0, day }
    }

    pub fn new_with_seconds(hour: u8, minute: u8, second: u8, day: u32) -> Self {
        TimeInfo { hour, minute, second, day }
    }

    pub fn format_time(&self) -> String {
        format!("Day {} {:02}:{:02}:{:02}", self.day, self.hour, self.minute, self.second)
    }
}
