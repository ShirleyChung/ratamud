use crate::observable::Observable;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// Person 類別，實現 Observable trait
#[derive(Clone, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub description: String,
    pub abilities: Vec<String>,
    pub items: Vec<String>,
    pub status: String,
    pub x: usize,                    // X 座標
    pub y: usize,                    // Y 座標
}

impl Person {
    pub fn new(name: String, description: String) -> Self {
        Person {
            name,
            description,
            abilities: Vec::new(),
            items: Vec::new(),
            status: "正常".to_string(),
            x: 50,                    // 初始位置：地圖中央
            y: 50,
        }
    }

    // 添加能力
    pub fn add_ability(&mut self, ability: String) {
        self.abilities.push(ability);
    }

    // 添加物品
    pub fn add_item(&mut self, item: String) {
        self.items.push(item);
    }

    // 設置狀態
    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    // 設置描述
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    // 移動到指定位置
    pub fn move_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    // 獲取位置
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

// 實現 Observable trait
impl Observable for Person {
    fn show_title(&self) -> String {
        self.name.clone()
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

        // 添加物品
        if !self.items.is_empty() {
            list.push("【持有物品】".to_string());
            for item in &self.items {
                list.push(item.clone());
            }
        }

        // 如果沒有能力和物品，顯示空
        if list.is_empty() {
            list.push("無特殊能力".to_string());
            list.push("未持有物品".to_string());
        }

        list
    }
}
