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
    pub items: HashMap<String, u32>,  // 物品名稱 -> 數量
    pub status: String,
    pub x: usize,                    // X 座標
    pub y: usize,                    // Y 座標
    pub hp: i32,                     // 體力/健康程度
    pub mp: i32,                     // 精神力/意志力
    pub strength: i32,               // 力量
    pub knowledge: i32,              // 知識
    pub sociality: i32,              // 交誼
    pub age: u64,                    // 年齡（以秒計算）
    pub last_hunger_hour: u8,        // 上次扣 HP 的小時數
    pub is_sleeping: bool,           // 是否正在睡覺
    pub last_mp_restore_minute: u8,  // 上次恢復 MP 的分鐘數
    pub max_mp: i32,                 // 最大 MP 值
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
            hp: 100000,
            mp: 100000,
            strength: 100,
            knowledge: 100,
            sociality: 100,
            age: 0,
            last_hunger_hour: 0,
            is_sleeping: false,
            last_mp_restore_minute: 0,
            max_mp: 100000,
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
    #[allow(dead_code)]
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
        self.hp -= 1; // 移動消耗體力
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
        let file_path = format!("{person_dir}/{filename}.json");
        let json = serde_json::to_string_pretty(self)?;
        fs::write(file_path, json)?;
        Ok(())
    }

    // 從文件加載 Person
    pub fn load(person_dir: &str, filename: &str) -> Result<Person, Box<dyn std::error::Error>> {
        let file_path = format!("{person_dir}/{filename}.json");
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

        // 添加睡眠狀態
        if self.is_sleeping {
            list.push("【狀態】".to_string());
            list.push("💤 睡眠中（不會消耗HP，每10分鐘恢復10% MP）".to_string());
        }

        // 添加屬性
        list.push("【屬性】".to_string());
        list.push(format!("HP: {}", self.hp));
        list.push(format!("MP: {} / {}", self.mp, self.max_mp));
        list.push(format!("力量: {}", self.strength));
        list.push(format!("知識: {}", self.knowledge));
        list.push(format!("交誼: {}", self.sociality));
        list.push(format!("存在時間: {}秒 ({}天{}時{}分{}秒)", 
            self.age,
            self.age / 86400,
            (self.age % 86400) / 3600,
            (self.age % 3600) / 60,
            self.age % 60
        ));

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
            list.push(format!("【持有物品】({total_types} 種, {total_count} 個)"));
            for (item, count) in &self.items {
                let display_name = item_registry::get_item_display_name(item);
                list.push(format!("{display_name} x{count}"));
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
        // 每秒增加年齡
        self.age += 1;
        
        // 只有在非睡眠狀態才扣除 HP（飢餓消耗）
        if !self.is_sleeping
            && current_time.hour != self.last_hunger_hour {
                self.hp -= 100;
                self.last_hunger_hour = current_time.hour;
                
                // HP 不能低於 0
                if self.hp < 0 {
                    self.hp = 0;
                }
            }
        
        // 睡眠時每 10 分鐘恢復 10% MP
        if self.is_sleeping {
            // 檢查是否到達 10 的倍數分鐘且與上次不同
            if current_time.minute % 10 == 0 && current_time.minute != self.last_mp_restore_minute {
                let restore_amount = (self.max_mp as f32 * 0.1) as i32;
                self.mp += restore_amount;
                
                // MP 不能超過最大值
                if self.mp > self.max_mp {
                    self.mp = self.max_mp;
                }
                
                self.last_mp_restore_minute = current_time.minute;
            }
        }
        
        // 根據遊戲時間更新人物狀態（非睡眠時）
        if !self.is_sleeping {
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
}
