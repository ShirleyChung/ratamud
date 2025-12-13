use crate::observable::Observable;
use crate::time_updatable::{TimeUpdatable, TimeInfo};
use crate::item_registry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::collections::HashMap;

// Person 類別，實現 Observable trait
#[derive(Clone, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub description: String,
    pub abilities: Vec<String>,
    #[serde(default = "default_items")]
    pub items: HashMap<String, u32>,  // 物品名稱 -> 數量
    pub status: String,
    pub x: usize,                    // X 座標
    pub y: usize,                    // Y 座標
}

// 預設物品 HashMap
fn default_items() -> HashMap<String, u32> {
    HashMap::new()
}

impl Person {
    pub fn new(name: String, description: String) -> Self {
        Person {
            name,
            description,
            abilities: Vec::new(),
            items: HashMap::new(),
            status: "正常".to_string(),
            x: 50,                    // 初始位置：地圖中央
            y: 50,
        }
    }

    // 添加能力
    pub fn add_ability(&mut self, ability: String) {
        self.abilities.push(ability);
    }

    // 添加物品（支援數量）
    pub fn add_item(&mut self, item: String) {
        self.add_items(item, 1);
    }
    
    // 添加指定數量的物品
    pub fn add_items(&mut self, item: String, quantity: u32) {
        *self.items.entry(item).or_insert(0) += quantity;
    }

    // 放下物品（預設數量1）
    pub fn drop_item(&mut self, item_name: &str) -> Option<String> {
        self.drop_items(item_name, 1)
    }
    
    // 放下指定數量的物品
    pub fn drop_items(&mut self, item_name: &str, quantity: u32) -> Option<String> {
        if let Some(count) = self.items.get_mut(item_name) {
            if *count >= quantity {
                *count -= quantity;
                if *count == 0 {
                    self.items.remove(item_name);
                }
                return Some(item_name.to_string());
            }
        }
        None
    }
    
    // 獲取物品數量
    pub fn get_item_count(&self, item_name: &str) -> u32 {
        *self.items.get(item_name).unwrap_or(&0)
    }

    // 設置狀態
    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    // 設置描述
    #[allow(dead_code)]
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    // 移動到指定位置
    pub fn move_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    // 獲取位置
    #[allow(dead_code)]
    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    // 保存 Person 到文件
    pub fn save(&self, person_dir: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(person_dir)?;
        let file_path = format!("{}/{}.json", person_dir, filename);
        let json = serde_json::to_string_pretty(self)?;
        fs::write(file_path, json)?;
        Ok(())
    }

    // 從文件加載 Person
    pub fn load(person_dir: &str, filename: &str) -> Result<Person, Box<dyn std::error::Error>> {
        let file_path = format!("{}/{}.json", person_dir, filename);
        if Path::new(&file_path).exists() {
            let json = fs::read_to_string(file_path)?;
            let person = serde_json::from_str(&json)?;
            Ok(person)
        } else {
            Err("Person file not found".into())
        }
    }
}

impl Observable for Person {
    fn show_title(&self) -> String {
        format!("{}【位置: ({}, {})】", self.name, self.x, self.y)
    }

    fn show_description(&self) -> String {
        let mut desc = self.description.clone();
        desc.push_str(&format!("\n狀態: {}", self.status));
        desc
    }

    fn show_list(&self) -> Vec<String> {
        let mut list = Vec::new();

        // 添加能力
        if !self.abilities.is_empty() {
            list.push("【能力】".to_string());
            for ability in &self.abilities {
                list.push(ability.clone());
            }
        }

        // 添加物品（顯示數量和英文名）
        if !self.items.is_empty() {
            let total_types = self.items.len();
            let total_count: u32 = self.items.values().sum();
            list.push(format!("【持有物品】({} 種, {} 個)", total_types, total_count));
            for (item, count) in &self.items {
                let display_name = item_registry::get_item_display_name(item);
                list.push(format!("{} x{}", display_name, count));
            }
        } else {
            list.push("【持有物品】(0 種, 0 個)".to_string());
            list.push("未持有物品".to_string());
        }

        // 如果沒有能力，顯示空能力
        if self.abilities.is_empty() {
            list.push("【能力】".to_string());
            list.push("無特殊能力".to_string());
        }

        list
    }
}

// 實現 TimeUpdatable trait
impl TimeUpdatable for Person {
    fn on_time_update(&mut self, current_time: &TimeInfo) {
        // 根據遊戲時間更新人物狀態
        // 例如：在特定時間改變狀態
        match current_time.hour {
            6..=8 => {
                if !self.status.contains("早晨") && !self.status.contains("起床") {
                    self.set_status("起床中".to_string());
                }
            },
            9..=11 => {
                if !self.status.contains("工作") {
                    self.set_status("工作中".to_string());
                }
            },
            12..=13 => {
                if !self.status.contains("午餐") {
                    self.set_status("午餐時間".to_string());
                }
            },
            14..=17 => {
                if !self.status.contains("工作") {
                    self.set_status("工作中".to_string());
                }
            },
            18..=19 => {
                if !self.status.contains("晚餐") {
                    self.set_status("晚餐時間".to_string());
                }
            },
            20..=22 => {
                if !self.status.contains("放鬆") {
                    self.set_status("放鬆中".to_string());
                }
            },
            _ => {
                if !self.status.contains("睡眠") {
                    self.set_status("睡眠中".to_string());
                }
            }
        }
    }
}
