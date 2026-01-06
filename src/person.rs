use crate::time_updatable::{TimeUpdatable, TimeInfo};
use crate::item_registry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use rand::Rng;
use std::sync::OnceLock;

// 全域靜態描述資料
static PERSON_DESCRIPTIONS: OnceLock<PersonDescriptions> = OnceLock::new();

// 描述資料結構
#[derive(Debug, Deserialize)]
pub struct PersonDescriptions {
    pub appearance: HashMap<String, String>,
    pub strength: AttributeRanges,
    pub build: AttributeRanges,
    pub health_status: HealthStatusRanges,
}

#[derive(Debug, Deserialize)]
pub struct AttributeRanges {
    pub ranges: Vec<AttributeRange>,
}

#[derive(Debug, Deserialize)]
pub struct AttributeRange {
    pub min: i32,
    pub max: i32,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthStatusRanges {
    pub ranges: Vec<HealthStatusRange>,
}

#[derive(Debug, Deserialize)]
pub struct HealthStatusRange {
    pub hp_ratio: f32,
    pub description: String,
}

impl PersonDescriptions {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let json_path = "worlds/person_descriptions.json";
        let json_str = fs::read_to_string(json_path)?;
        let descriptions: PersonDescriptions = serde_json::from_str(&json_str)?;
        Ok(descriptions)
    }

    pub fn get_appearance_description(&self, appearance: i32) -> String {
        let clamped = appearance.clamp(1, 100);
        self.appearance
            .get(&clamped.to_string())
            .cloned()
            .unwrap_or_else(|| "普通的".to_string())
    }

    pub fn get_strength_description(&self, strength: i32) -> String {
        for range in &self.strength.ranges {
            if strength >= range.min && strength <= range.max {
                return range.description.clone();
            }
        }
        "普通的".to_string()
    }

    pub fn get_build_description(&self, build: i32) -> String {
        for range in &self.build.ranges {
            if build >= range.min && build <= range.max {
                return range.description.clone();
            }
        }
        "普通".to_string()
    }

    pub fn get_health_status_description(&self, hp: i32, max_hp: i32) -> String {
        if max_hp <= 0 {
            return "未知".to_string();
        }
        
        let hp_ratio = hp as f32 / max_hp as f32;
        
        for range in &self.health_status.ranges {
            if hp_ratio <= range.hp_ratio {
                return range.description.clone();
            }
        }
        
        "健康的".to_string()
    }
}

/// 初始化描述資料（應在程式啟動時調用）
pub fn init_person_descriptions() {
    PERSON_DESCRIPTIONS.get_or_init(|| {
        PersonDescriptions::load().expect("Failed to load person descriptions")
    });
}

/// 獲取全域描述資料
fn get_descriptions() -> &'static PersonDescriptions {
    PERSON_DESCRIPTIONS.get_or_init(|| {
        PersonDescriptions::load().expect("Failed to load person descriptions")
    })
}

// 對話條件
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DialogueCondition {
    pub attribute: String,      // 屬性名稱 (hp, mp, strength, 顏值, 性別 等)
    pub operator: String,       // 運算子 (>, <, =, >=, <=, !=)
    pub value: String,          // 比較值 (可能是數字或字串)
}

impl DialogueCondition {
    /// 評估條件是否滿足
    pub fn evaluate(&self, person: &Person) -> bool {
        // 取得屬性值
        let attr_value = match self.attribute.as_str() {
            "hp" => person.hp.to_string(),
            "mp" => person.mp.to_string(),
            "max_hp" => person.max_hp.to_string(),
            "max_mp" => person.max_mp.to_string(),
            "strength" | "力量" => person.strength.to_string(),
            "knowledge" | "知識" => person.knowledge.to_string(),
            "sociality" | "交誼" => person.sociality.to_string(),
            "relationship" | "好感度" => person.relationship.to_string(),
            "talk_eagerness" | "積極度" => person.talk_eagerness.to_string(),
            "age" | "年齡" => person.age.to_string(),
            "gender" | "性別" => person.gender.clone(),
            "appearance" | "顏值" => person.appearance.to_string(),
            "items_count" | "物品數量" => person.items.values().sum::<u32>().to_string(),
            attr if attr.starts_with("item:") => {
                // 檢查持有特定物品數量 (例如: item:麵包)
                let item_name = &attr[5..];
                person.items.get(item_name).unwrap_or(&0).to_string()
            },
            _ => return false,
        };
        
        // 執行比較
        match self.operator.as_str() {
            ">" => {
                if let (Ok(a), Ok(b)) = (attr_value.parse::<i32>(), self.value.parse::<i32>()) {
                    a > b
                } else {
                    false
                }
            },
            "<" => {
                if let (Ok(a), Ok(b)) = (attr_value.parse::<i32>(), self.value.parse::<i32>()) {
                    a < b
                } else {
                    false
                }
            },
            ">=" => {
                if let (Ok(a), Ok(b)) = (attr_value.parse::<i32>(), self.value.parse::<i32>()) {
                    a >= b
                } else {
                    false
                }
            },
            "<=" => {
                if let (Ok(a), Ok(b)) = (attr_value.parse::<i32>(), self.value.parse::<i32>()) {
                    a <= b
                } else {
                    false
                }
            },
            "=" | "==" => attr_value == self.value,
            "!=" => attr_value != self.value,
            _ => false,
        }
    }
}

// 對話選項（包含句子和條件）
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DialogueOption {
    pub text: String,                       // 對話文字
    pub conditions: Vec<DialogueCondition>, // 觸發條件列表（AND 關係）
    pub weight: f32,                        // 基礎權重（預設1.0）
}

impl DialogueOption {
    pub fn new(text: String) -> Self {
        DialogueOption {
            text,
            conditions: Vec::new(),
            weight: 1.0,
        }
    }
    
    pub fn with_conditions(text: String, conditions: Vec<DialogueCondition>) -> Self {
        DialogueOption {
            text,
            conditions,
            weight: 1.0,
        }
    }
    
    /// 檢查所有條件是否滿足
    pub fn check_conditions(&self, person: &Person) -> bool {
        self.conditions.iter().all(|cond| cond.evaluate(person))
    }
    
    /// 計算實際權重（條件滿足時返回權重，否則返回0）
    pub fn get_effective_weight(&self, person: &Person) -> f32 {
        if self.check_conditions(person) {
            self.weight
        } else {
            0.0
        }
    }
}

fn default_gender() -> String {
    "未知".to_string()
}

fn default_appearance() -> i32 {
    50  // 預設顏值 50
}

fn default_build() -> i32 {
    50  // 預設體格 50
}

// Person 類別，實現 Observable trait
#[derive(Clone, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub description: String,
    pub abilities: Vec<String>,
    pub items: HashMap<String, u32>,  // 物品名稱 -> 數量（向後兼容）
    #[serde(default)]
    pub item_instances: HashMap<String, Vec<crate::item::ItemInstance>>,  // 物品名稱 -> 實例列表
    pub status: String,
    pub x: usize,                    // X 座標
    pub y: usize,                    // Y 座標
    #[serde(default = "default_map")]
    pub map: String,                 // 所在地圖名稱
    pub hp: i32,                     // 體力/健康程度
    pub mp: i32,                     // 精神力/意志力
    pub max_hp: i32,                 // 最大 HP 值
    pub max_mp: i32,                 // 最大 MP 值
    pub strength: i32,               // 力量
    pub knowledge: i32,              // 知識
    pub sociality: i32,              // 交誼
    pub age: u64,                    // 年齡（以秒計算）
    pub last_hunger_hour: u8,        // 上次扣 HP 的小時數
    pub is_sleeping: bool,           // 是否正在睡覺
    pub last_mp_restore_minute: u8,  // 上次恢復 MP 的分鐘數
    #[serde(default)]
    pub is_interacting: bool,        // 是否正在互動中（交易、對話等）
    #[serde(default)]
    pub dialogues: HashMap<String, Vec<DialogueOption>>,  // 話題 -> 對話選項列表
    #[serde(default = "default_talk_eagerness")]
    pub talk_eagerness: u8,          // 說話積極度 (0-100)
    #[serde(default)]
    pub relationship: i32,           // 好感度 (-100 到 100)
    #[serde(default)]
    pub dialogue_state: String,      // 當前對話狀態 (例如: "初見", "熟識", "好友")
    #[serde(default)]
    pub met_player: bool,            // 是否見過玩家
    #[serde(default)]
    pub interaction_count: u32,      // 互動次數
    #[serde(default = "default_gender")]
    pub gender: String,              // 性別
    #[serde(default = "default_appearance")]
    pub appearance: i32,             // 顏值 (0-100)
    #[serde(default = "default_build")]
    pub build: i32,                  // 體格 (0-100)
    #[serde(default)]
    pub party_leader: Option<String>, // 組隊領隊名稱（None表示未組隊，Some("me")表示與玩家組隊）
    #[serde(default)]
    pub combat_skills: HashMap<String, CombatSkill>, // 戰鬥技能 (技能名 -> 技能資料)
    #[serde(default)]
    pub skill_dialogues: HashMap<String, String>, // 技能台詞 (技能名 -> 台詞)
    #[serde(default)]
    pub combat_exp: i32,             // 戰鬥經驗值
}

/// 戰鬥技能
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CombatSkill {
    pub name: String,        // 技能名稱
    pub proficiency: i32,    // 熟練度
    pub damage: i32,         // 傷害值
    pub cooldown: i32,       // 冷卻時間（回合數）
    pub current_cooldown: i32, // 當前冷卻剩餘時間
}

fn default_talk_eagerness() -> u8 {
    100  // 預設積極度為 100
}

fn default_map() -> String {
    "beginMap".to_string()
}

impl Person {
    pub fn new(name: String, description: String) -> Self {
        let mut person = Person {
            name: name.clone(),
            description,
            abilities: Vec::new(),
            items: HashMap::new(),
            item_instances: HashMap::new(),
            status: "正常".to_string(),
            x: 50,                    // 初始位置：地圖中央
            y: 50,
            map: "beginMap".to_string(),  // 預設在 beginMap
            hp: 100_000,
            mp: 100_000,
            max_hp: 100_000,
            max_mp: 100_000,
            strength: 100,
            knowledge: 100,
            sociality: 100,
            age: 0,
            last_hunger_hour: 0,
            is_sleeping: false,
            last_mp_restore_minute: 0,
            is_interacting: false,    // 初始化為 false
            dialogues: HashMap::new(),
            talk_eagerness: 100,
            relationship: 0,
            dialogue_state: "初見".to_string(),
            met_player: false,
            interaction_count: 0,
            gender: "".to_string(),
            appearance: 50,
            build: 50,
            party_leader: None,
            combat_skills: HashMap::new(),
            skill_dialogues: HashMap::new(),
            combat_exp: 0,
        };
        
        // 初始化基本戰鬥技能
        person.init_combat_skills();
        
        // 設置預設的"被叫住"對話
        person.set_dialogue("被叫住".to_string(), format!("{name} 有什麼事嗎？"));
        
        // 更新描述
        person.update_description();
        
        person
    }
    
    /// 初始化戰鬥技能
    fn init_combat_skills(&mut self) {
        // 初始化 punch 技能
        self.combat_skills.insert("punch".to_string(), CombatSkill {
            name: "punch".to_string(),
            proficiency: 0,
            damage: 2,
            cooldown: 2,
            current_cooldown: 0,
        });
        
        // 初始化 kick 技能
        self.combat_skills.insert("kick".to_string(), CombatSkill {
            name: "kick".to_string(),
            proficiency: 0,
            damage: 3,
            cooldown: 3,
            current_cooldown: 0,
        });
        
        // 設置預設技能台詞
        if self.name == "me" {
            self.skill_dialogues.insert("punch".to_string(), "吃我一拳".to_string());
            self.skill_dialogues.insert("kick".to_string(), "看我的飛踢".to_string());
        } else {
            self.skill_dialogues.insert("punch".to_string(), "你搞什麼".to_string());
            self.skill_dialogues.insert("kick".to_string(), "別惹我".to_string());
        }
    }
    
    /// 使用戰鬥技能（練習模式，不指定目標）
    pub fn practice_skill(&mut self, skill_name: &str, in_combat: bool) -> Option<String> {
        if let Some(skill) = self.combat_skills.get_mut(skill_name) {
            // 檢查冷卻
            if skill.current_cooldown > 0 {
                return Some(format!("你還沒準備好{}", skill_name));
            }
            
            // 增加熟練度
            let proficiency_gain = if in_combat { 2 } else { 1 };
            skill.proficiency += proficiency_gain;
            
            // 設置冷卻
            skill.current_cooldown = skill.cooldown;
            
            Some(format!("你練習了 {} (熟練度 +{})", skill_name, proficiency_gain))
        } else {
            None
        }
    }
    
    /// 檢查是否可以發動戰鬥
    pub fn can_start_combat(&self) -> bool {
        self.hp >= self.max_hp / 2
    }
    
    /// 減少技能冷卻
    pub fn reduce_skill_cooldowns(&mut self) {
        for skill in self.combat_skills.values_mut() {
            if skill.current_cooldown > 0 {
                skill.current_cooldown -= 1;
            }
        }
    }
    
    /// 獲取技能冷卻狀態
    pub fn get_skill_cooldown(&self, skill_name: &str) -> i32 {
        self.combat_skills
            .get(skill_name)
            .map(|s| s.current_cooldown)
            .unwrap_or(0)
    }

    /// 根據屬性自動生成描述
    pub fn update_description(&mut self) {
        let desc = get_descriptions();
        
        // 外貌描述
        let appearance_desc = desc.get_appearance_description(self.appearance);
        
        // 體格描述
        let build_desc = desc.get_build_description(self.build);
        
        // 性別描述
        let gender_desc = if self.gender == "男" || self.gender == "male" || self.gender == "男性" {
            "男子"
        } else if self.gender == "女" || self.gender == "female" || self.gender == "女性" {
            "女子"
        } else {
            "人"
        };
        
        // 力量描述
        let strength_desc = desc.get_strength_description(self.strength);
        
        // 健康狀態描述
        let health_desc = desc.get_health_status_description(self.hp, self.max_hp);
        
        // 組合描述
        if self.gender.is_empty() {
            self.description = format!("一位{appearance_desc}，{strength_desc}人，看起來{health_desc}。");
        } else {
            self.description = format!("一位{appearance_desc}，{build_desc}體格，{strength_desc}{gender_desc}，看起來{health_desc}。");
        }
    }

    /// 設置外貌並更新描述
    pub fn set_appearance(&mut self, appearance: i32) {
        self.appearance = appearance.clamp(0, 100);
        self.update_description();
    }

    /// 設置體格並更新描述
    pub fn set_build(&mut self, build: i32) {
        self.build = build.clamp(0, 100);
        self.update_description();
    }

    /// 設置性別並更新描述
    pub fn set_gender(&mut self, gender: String) {
        self.gender = gender;
        self.update_description();
    }

    /// 設置力量並更新描述
    pub fn set_strength(&mut self, strength: i32) {
        self.strength = strength;
        self.update_description();
    }

    /// 設置 HP 並更新描述
    pub fn set_hp(&mut self, hp: i32) {
        self.hp = hp;
        self.update_description();
    }

    /// 設置台詞（新版：支援多個選項）
    pub fn add_dialogue_option(&mut self, topic: String, option: DialogueOption) {
        self.dialogues.entry(topic).or_default().push(option);
    }

    /// 設置台詞（簡單版：無條件）
    pub fn set_dialogue(&mut self, topic: String, text: String) {
        let option = DialogueOption::new(text);
        self.dialogues.entry(topic).or_default().push(option);
    }

    /// 設置說話積極度 (0-100)
    pub fn set_talk_eagerness(&mut self, eagerness: u8) {
        self.talk_eagerness = eagerness.min(100);
        self.update_description();
    }
    
    /// 顯示 Person 的詳細資料
    pub fn show_detail(&self) -> String {
        let mut info = String::new();
        
        // 標題
        info.push_str(&format!(" {} \n", self.name));
        
        // 基本資訊 - 緊湊格式
        info.push_str(&format!("│ {}\n", self.description));
        info.push_str(&format!("│ 位置: ({}, {}) @ {}\n", self.x, self.y, self.map));
        info.push_str(&format!("│ 狀態: {}\n", self.status));
        
        // 屬性 - 兩列排版
        info.push_str(&format!("│ HP: {:>3}/{:<3}  力量: {}\n", 
            self.hp, self.max_hp, self.strength));
        info.push_str(&format!("│ MP: {:>3}/{:<3}  知識: {}\n", 
            self.mp, self.max_mp, self.knowledge));
        info.push_str(&format!("│ 年齡: {}秒   交誼: {}\n", 
            self.age, self.sociality));
        info.push_str(&format!("│ 性別: {}    顏值: {}\n", 
            self.gender, self.appearance));
        
        // 關係信息
        if self.met_player || self.relationship != 0 || self.interaction_count > 0 {
            info.push_str("├─────────────────────────\n");
            info.push_str(&format!("│ 關係: {}\n", self.get_relationship_description()));
            if self.met_player {
                info.push_str(&format!("│ 互動次數: {}\n", self.interaction_count));
            }
        }
        
        // 持有物品
        if !self.items.is_empty() {
            info.push_str("├─────────────────────────\n");
            info.push_str("│ 持有物品:\n");
            for (item_name, quantity) in &self.items {
                info.push_str(&format!("│  • {item_name} x{quantity}\n"));
            }
        }
        
        // 能力
        if !self.abilities.is_empty() {
            info.push_str("├─────────────────────────\n");
            info.push_str("│ 能力:\n");
            for ability in &self.abilities {
                info.push_str(&format!("│  • {ability}\n"));
            }
        }
        
        // 對話設置
        if !self.dialogues.is_empty() {
            info.push_str("├─────────────────────────\n");
            info.push_str(&format!("│ 對話 (積極度: {}%)\n", self.talk_eagerness));
            for (topic, options) in &self.dialogues {
                info.push_str(&format!("│  [{topic}] {} 個選項\n", options.len()));
                for (i, opt) in options.iter().enumerate() {
                    let cond_str = if opt.conditions.is_empty() {
                        "無條件".to_string()
                    } else {
                        format!("{} 個條件", opt.conditions.len())
                    };
                    let max_len = 30;
                    let text = if opt.text.chars().count() > max_len {
                        let substr: String = opt.text.chars().take(max_len).collect();
                        format!("{substr}...")
                    } else {
                        opt.text.clone()
                    };
                    info.push_str(&format!("│    {}. {} ({})\n", i + 1, text, cond_str));
                }
            }
        } else if self.talk_eagerness > 0 {
            info.push_str("├─────────────────────────\n");
            info.push_str(&format!("│ 說話積極度: {}%\n", self.talk_eagerness));
        }
        
        info.push_str("└─────────────────────────\n");
        
        info
    }

    /// 根據權重選擇對話（新版）
    /// target_person: 用來評估條件的 Person（通常是玩家）
    pub fn get_weighted_dialogue(&self, topic: &str, target_person: &Person) -> Option<String> {
        let options = self.dialogues.get(topic)?;
        if options.is_empty() {
            return None;
        }
        
        // 計算所有選項的有效權重（根據 target_person 的屬性）
        let weights: Vec<f32> = options.iter()
            .map(|opt| opt.get_effective_weight(target_person))
            .collect();
        
        let total_weight: f32 = weights.iter().sum();
        
        // 如果沒有任何選項滿足條件，返回 None
        if total_weight <= 0.0 {
            return None;
        }
        
        // 加權隨機選擇
        let mut rng = rand::thread_rng();
        let mut roll = rng.gen::<f32>() * total_weight;
        
        for (i, &weight) in weights.iter().enumerate() {
            roll -= weight;
            if roll <= 0.0 {
                return Some(options[i].text.clone());
            }
        }
        
        // 備用：返回最後一個有效選項
        options.iter()
            .enumerate()
            .rev()
            .find(|(i, _)| weights[*i] > 0.0)
            .map(|(_, opt)| opt.text.clone())
    }

    /// 嘗試說話（根據積極度和權重）
    /// target_person: 用來評估條件的 Person（通常是玩家）
    pub fn try_talk(&self, topic: &str, target_person: &Person) -> Option<String> {
        // 根據積極度決定是否說話
        let mut rng = rand::thread_rng();
        let roll: u8 = rng.gen_range(0..100);                
        if roll < self.talk_eagerness {
            self.get_weighted_dialogue(topic, target_person)
        } else {
            None
        }
    }
    
    /// 根據好感度和狀態動態選擇對話（僅測試使用）
    #[cfg(test)]
    pub fn get_context_dialogue(&self, scene: &str) -> Option<String> {
        self.get_weighted_dialogue(scene, self)
    }
    
    /// 改變好感度
    pub fn change_relationship(&mut self, delta: i32) {
        self.relationship = (self.relationship + delta).clamp(-100, 100);
        self.update_dialogue_state();
    }
    
    /// 更新對話狀態
    fn update_dialogue_state(&mut self) {
        self.dialogue_state = match self.relationship {
            r if r >= 70 => "摯友".to_string(),
            r if r >= 30 => "好友".to_string(),
            r if r >= 0 => "普通".to_string(),
            r if r >= -30 => "冷淡".to_string(),
            _ => "敵對".to_string(),
        };
    }
    
    /// 標記為已見過玩家（僅測試使用）
    #[cfg(test)]
    pub fn mark_met_player(&mut self) {
        if !self.met_player {
            self.met_player = true;
            self.change_relationship(5);
        }
    }
    
    /// 獲取關係等級描述
    pub fn get_relationship_description(&self) -> String {
        match self.relationship {
            r if r >= 70 => format!("摯友 ({r})"),
            r if r >= 30 => format!("好友 ({r})"),
            r if r >= 0 => format!("普通 ({r})"),
            r if r >= -30 => format!("冷淡 ({r})"),
            r => format!("敵對 ({r})"),
        }
    }

        /// 消耗 MP，並檢查是否進入睡眠狀態
    pub fn check_hp(&mut self, amount: i32) {
        self.hp += amount;
        if self.hp < 0 {
            self.hp = 0;
        }
        if self.hp > self.max_hp {
            self.hp = self.max_hp;
        }
        if self.hp <= self.max_hp / 10 && self.hp > self.max_hp {
            self.status = "覺得有點累了".to_string();
        } else if self.hp <= self.max_hp / 4 {
            self.status = "感到疲憊".to_string();
        } else if self.hp <= 50 {
            self.status = "精疲力盡".to_string();
        } else {
            self.status = "正常".to_string();
        }
        self.update_description();
    }

    /// 消耗 MP，並檢查是否進入睡眠狀態
    pub fn check_mp(&mut self, amount: i32) {
        self.mp += amount;
        if self.mp < 0 {
            self.mp = 0;
        }
        if self.mp <= 50 {
            self.is_sleeping = true; // MP 耗盡後進入睡眠狀態
        }
    }

    // 添加能力
    pub fn add_ability(&mut self, ability: String) {
        self.abilities.push(ability);
    }

    // 添加物品（支援數量）
    pub fn add_item(&mut self, item: String) {
        self.add_item_with_quantity(item, 1);
    }
    
    // 添加指定數量的物品
    pub fn add_items(&mut self, item: String, quantity: u32) {
        self.add_item_with_quantity(item, quantity);
    }
    
    /// 添加物品（新版，使用物品實例）
    fn add_item_with_quantity(&mut self, item_name: String, quantity: u32) {
        // 同步更新舊格式（向後兼容）
        *self.items.entry(item_name.clone()).or_insert(0) += quantity;
        
        // 創建物品實例
        let instances = self.item_instances.entry(item_name.clone()).or_default();
        for _ in 0..quantity {
            instances.push(crate::item::ItemInstance::new(item_name.clone()));
        }
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

    // 移動到指定位置
    pub fn move_to(&mut self, x: usize, y: usize) {
        self.check_hp(-1); // 移動消耗體力
        self.x = x;
        self.y = y;
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

impl Person {
    /// 移除物品（新版，使用物品實例）
    pub fn remove_item(&mut self, item_name: &str, quantity: u32) -> bool {
        // 檢查是否有足夠數量
        let current = self.items.get(item_name).copied().unwrap_or(0);
        if current < quantity {
            return false;
        }
        
        // 更新舊格式
        let new_count = current - quantity;
        if new_count == 0 {
            self.items.remove(item_name);
        } else {
            self.items.insert(item_name.to_string(), new_count);
        }
        
        // 移除實例（從後面開始移除）
        if let Some(instances) = self.item_instances.get_mut(item_name) {
            for _ in 0..quantity {
                instances.pop();
            }
            if instances.is_empty() {
                self.item_instances.remove(item_name);
            }
        }
        
        true
    }
    
    /// 使用物品
    pub fn use_item(&mut self, item_name: &str) -> Result<String, String> {
        // 解析物品名稱
        let resolved_name = item_registry::resolve_item_name(item_name);
        
        // 檢查是否擁有該物品
        if !self.items.contains_key(&resolved_name) || self.items[&resolved_name] == 0 {
            return Err(format!("你沒有 {resolved_name}"));
        }
        
        // 檢查是否可使用
        if !item_registry::is_usable(&resolved_name) {
            return Err(format!("{resolved_name} 無法使用"));
        }
        
        // 獲取物品效果
        let effects = item_registry::get_item_effects(&resolved_name)
            .ok_or_else(|| format!("{resolved_name} 沒有效果"))?;
        
        // 應用效果
        let mut result_messages = Vec::new();
        for effect in effects {
            match effect {
                item_registry::ItemEffect::RestoreHp(amount) => {
                    let old_hp = self.hp;
                    self.hp = (self.hp + amount).min(self.max_hp);
                    let actual_restored = self.hp - old_hp;
                    result_messages.push(format!("恢復了 {actual_restored} HP"));
                },
                item_registry::ItemEffect::RestoreMp(amount) => {
                    let old_mp = self.mp;
                    self.mp = (self.mp + amount).min(self.max_mp);
                    let actual_restored = self.mp - old_mp;
                    result_messages.push(format!("恢復了 {actual_restored} MP"));
                },
                item_registry::ItemEffect::IncreaseMaxHp(amount) => {
                    self.max_hp += amount;
                    result_messages.push(format!("最大 HP 增加了 {amount}"));
                },
                item_registry::ItemEffect::IncreaseMaxMp(amount) => {
                    self.max_mp += amount;
                    result_messages.push(format!("最大 MP 增加了 {amount}"));
                },
                item_registry::ItemEffect::IncreaseStrength(amount) => {
                    self.strength += amount;
                    result_messages.push(format!("力量增加了 {amount}"));
                },
                item_registry::ItemEffect::IncreaseKnowledge(amount) => {
                    self.knowledge += amount;
                    result_messages.push(format!("知識增加了 {amount}"));
                },
                item_registry::ItemEffect::IncreaseSociality(amount) => {
                    self.sociality += amount;
                    result_messages.push(format!("交誼增加了 {amount}"));
                },
                item_registry::ItemEffect::ChangeSex(sex) => {
                    self.gender = sex.to_string();
                    result_messages.push(format!("性別變更為 {}", self.gender));
                },
                item_registry::ItemEffect::IncreaseAppearance(amount) => {
                    self.appearance += amount;
                    result_messages.push(format!("顏值變更為 {}", self.appearance));
                },
                item_registry::ItemEffect::DecreaseAppearance(amount) => {
                    self.appearance -= amount;
                    if self.appearance < 0 {
                        self.appearance = 0;
                    }
                    result_messages.push(format!("顏值變更為 {}", self.appearance));
                },
            }
        }
        
        // 消耗物品
        self.remove_item(&resolved_name, 1);
        
        // 更新描述（因為屬性可能改變）
        self.update_description();
        
        Ok(format!("使用了 {resolved_name}：{}", result_messages.join("、")))
    }
}

// 實現 TimeUpdatable trait
impl TimeUpdatable for Person {
    fn on_time_update(&mut self, current_time: &TimeInfo) {
        // 如果 MP 已經耗盡，強制進入睡眠狀態
        self.check_mp(0);

        // 每秒增加年齡
        self.age += 1;
        
        // 只有在非睡眠狀態才扣除 HP（飢餓消耗）
        if !self.is_sleeping
            && current_time.hour != self.last_hunger_hour {
                self.check_hp(-100);
                self.last_hunger_hour = current_time.hour;            
            }
        
        // 睡眠恢復MP
        if self.is_sleeping {
            // 有立即效果的恢復
            self.check_mp(1);                
            // MP 不能超過最大值
            if self.mp > self.max_mp {
                self.mp = self.max_mp;
            }
            self.update_description();
        } 
        // 根據遊戲時間更新人物狀態（非睡眠時）
        else {
            self.set_status("".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_system() {
        let mut person = Person::new("測試NPC".to_string(), "測試用NPC".to_string());
        
        // 測試初始狀態
        assert_eq!(person.relationship, 0);
        assert_eq!(person.dialogue_state, "初見");
        assert!(!person.met_player);
        assert_eq!(person.interaction_count, 0);
        
        // 測試好感度改變
        person.change_relationship(50);
        assert_eq!(person.relationship, 50);
        assert_eq!(person.dialogue_state, "好友");
        
        person.change_relationship(30);
        assert_eq!(person.relationship, 80);
        assert_eq!(person.dialogue_state, "摯友");
        
        // 測試超出上限（應該被限制在 100）
        person.change_relationship(50);
        assert_eq!(person.relationship, 100);
        
        // 測試降低到負值
        person.change_relationship(-150);
        assert_eq!(person.relationship, -50); // 100 - 150 = -50
        assert_eq!(person.dialogue_state, "敵對");
        
        // 測試標記已見過玩家
        person.mark_met_player();
        assert!(person.met_player);
        assert_eq!(person.relationship, -45); // -50 + 5
        
        // 第二次調用不應該再加好感度
        person.mark_met_player();
        assert_eq!(person.relationship, -45);
    }
    
    #[test]
    fn test_context_dialogue() {
        let mut person = Person::new("商人".to_string(), "測試商人".to_string());
        
        // 設置不同等級的對話
        person.set_dialogue("對話:敵對".to_string(), "走開！".to_string());
        person.set_dialogue("對話:普通".to_string(), "你好".to_string());
        person.set_dialogue("對話:好友".to_string(), "嘿朋友！".to_string());
        person.set_dialogue("對話".to_string(), "預設對話".to_string());
        
        // 測試敵對狀態
        person.relationship = -50;
        person.change_relationship(0); // 更新狀態
        assert_eq!(person.get_context_dialogue("對話"), Some("走開！".to_string()));
        
        // 測試普通狀態
        person.change_relationship(60); // -50 + 60 = 10
        assert_eq!(person.get_context_dialogue("對話"), Some("你好".to_string()));
        
        // 測試好友狀態
        person.change_relationship(30); // 10 + 30 = 40
        assert_eq!(person.get_context_dialogue("對話"), Some("嘿朋友！".to_string()));
        
        // 測試沒有對應對話時回退到基礎對話
        assert_eq!(person.get_context_dialogue("告別"), None);
    }
    
    #[test]
    fn test_relationship_description() {
        let mut person = Person::new("NPC".to_string(), "測試".to_string());
        
        person.relationship = 80;
        assert!(person.get_relationship_description().contains("摯友"));
        
        person.relationship = 50;
        assert!(person.get_relationship_description().contains("好友"));
        
        person.relationship = 10;
        assert!(person.get_relationship_description().contains("普通"));
        
        person.relationship = -20;
        assert!(person.get_relationship_description().contains("冷淡"));
        
        person.relationship = -50;
        assert!(person.get_relationship_description().contains("敵對"));
    }
}
