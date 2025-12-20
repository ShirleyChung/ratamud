use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 任務狀態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
pub enum QuestStatus {
    #[default]
    NotStarted,   // 未開始
    InProgress,   // 進行中
    Completed,    // 已完成
    Failed,       // 失敗
}


/// 任務條件類型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum QuestCondition {
    /// 與 NPC 對話
    #[serde(rename = "talk_to_npc")]
    TalkToNpc {
        npc_id: String,
        #[serde(default)]
        completed: bool,
    },
    
    /// 擁有指定物品
    #[serde(rename = "has_item")]
    HasItem {
        item: String,
        count: u32,
        #[serde(default)]
        completed: bool,
    },
    
    /// 擊殺敵人
    #[serde(rename = "kill_enemy")]
    KillEnemy {
        enemy: String,
        count: u32,
        #[serde(default)]
        current: u32,
    },
    
    /// 到達地點
    #[serde(rename = "reach_location")]
    ReachLocation {
        map: String,
        x: usize,
        y: usize,
        #[serde(default)]
        completed: bool,
    },
    
    /// 玩家屬性要求
    #[serde(rename = "player_stat")]
    PlayerStat {
        stat: String,      // "hp", "mp", "strength", "knowledge", "sociality"
        min_value: i32,
        #[serde(default)]
        completed: bool,
    },
    
    /// NPC 好感度要求
    #[serde(rename = "npc_relationship")]
    NpcRelationship {
        npc_id: String,
        min_value: i32,
        #[serde(default)]
        completed: bool,
    },
}

impl QuestCondition {
    /// 檢查條件是否完成
    pub fn is_completed(&self) -> bool {
        match self {
            QuestCondition::TalkToNpc { completed, .. } => *completed,
            QuestCondition::HasItem { completed, .. } => *completed,
            QuestCondition::KillEnemy { count, current, .. } => current >= count,
            QuestCondition::ReachLocation { completed, .. } => *completed,
            QuestCondition::PlayerStat { completed, .. } => *completed,
            QuestCondition::NpcRelationship { completed, .. } => *completed,
        }
    }
    
    /// 獲取條件描述
    pub fn description(&self) -> String {
        match self {
            QuestCondition::TalkToNpc { npc_id, completed } => {
                let status = if *completed { "✓" } else { "○" };
                format!("{status} 與 {npc_id} 對話")
            }
            QuestCondition::HasItem { item, count, completed } => {
                let status = if *completed { "✓" } else { "○" };
                format!("{status} 擁有 {item} x{count}")
            }
            QuestCondition::KillEnemy { enemy, count, current } => {
                format!("擊殺 {enemy} ({current}/{count})")
            }
            QuestCondition::ReachLocation { map, x, y, completed } => {
                let status = if *completed { "✓" } else { "○" };
                format!("{status} 到達 {map} ({x}, {y})")
            }
            QuestCondition::PlayerStat { stat, min_value, completed } => {
                let status = if *completed { "✓" } else { "○" };
                format!("{status} {stat} 達到 {min_value}")
            }
            QuestCondition::NpcRelationship { npc_id, min_value, completed } => {
                let status = if *completed { "✓" } else { "○" };
                format!("{status} {npc_id} 好感度達到 {min_value}")
            }
        }
    }
}

/// 任務獎勵類型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum QuestReward {
    /// 物品獎勵
    #[serde(rename = "item")]
    Item {
        item: String,
        count: u32,
    },
    
    /// 經驗值獎勵（預留）
    #[serde(rename = "experience")]
    Experience {
        amount: u32,
    },
    
    /// 好感度獎勵
    #[serde(rename = "relationship")]
    Relationship {
        npc_id: String,
        change: i32,
    },
    
    /// 解鎖對話
    #[serde(rename = "unlock_dialogue")]
    UnlockDialogue {
        npc_id: String,
        scene: String,
        text: String,
    },
    
    /// 屬性提升
    #[serde(rename = "stat_boost")]
    StatBoost {
        stat: String,
        amount: i32,
    },
}

impl QuestReward {
    /// 獲取獎勵描述
    pub fn description(&self) -> String {
        match self {
            QuestReward::Item { item, count } => {
                format!("獲得 {item} x{count}")
            }
            QuestReward::Experience { amount } => {
                format!("獲得 {amount} 經驗值")
            }
            QuestReward::Relationship { npc_id, change } => {
                if *change > 0 {
                    format!("{npc_id} 好感度 +{change}")
                } else {
                    format!("{npc_id} 好感度 {change}")
                }
            }
            QuestReward::UnlockDialogue { npc_id, scene, .. } => {
                format!("解鎖 {npc_id} 的 {scene} 對話")
            }
            QuestReward::StatBoost { stat, amount } => {
                format!("{stat} +{amount}")
            }
        }
    }
}

/// 任務結構
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub description: String,
    
    /// 前置任務（必須完成的任務 ID 列表）
    #[serde(default)]
    pub prerequisites: Vec<String>,
    
    /// 任務條件
    pub conditions: Vec<QuestCondition>,
    
    /// 任務獎勵
    pub rewards: Vec<QuestReward>,
    
    /// 任務狀態
    #[serde(default)]
    pub status: QuestStatus,
    
    /// 任務給予者（NPC ID）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub giver: Option<String>,
    
    /// 是否可重複
    #[serde(default)]
    pub repeatable: bool,
}

impl Quest {
    /// 檢查是否可以開始（前置任務已完成）
    pub fn can_start(&self, completed_quests: &[String]) -> bool {
        self.prerequisites.iter()
            .all(|prereq| completed_quests.contains(prereq))
    }
    
    /// 檢查所有條件是否完成
    pub fn check_conditions(&self) -> bool {
        self.conditions.iter().all(|c| c.is_completed())
    }
    
    /// 顯示任務詳情
    pub fn show_detail(&self) -> String {
        let mut info = String::new();
        
        // 標題
        info.push_str(&format!("┌─ {} ─────────────────\n", self.name));
        info.push_str(&format!("│ ID: {}\n", self.id));
        
        // 描述
        info.push_str(&format!("│ {}\n", self.description));
        
        // 狀態
        let status_str = match self.status {
            QuestStatus::NotStarted => "未開始",
            QuestStatus::InProgress => "進行中",
            QuestStatus::Completed => "已完成",
            QuestStatus::Failed => "失敗",
        };
        info.push_str(&format!("│ 狀態: {status_str}\n"));
        
        // 前置任務
        if !self.prerequisites.is_empty() {
            info.push_str("├─────────────────────────\n");
            info.push_str("│ 前置任務:\n");
            for prereq in &self.prerequisites {
                info.push_str(&format!("│  • {prereq}\n"));
            }
        }
        
        // 任務條件
        if !self.conditions.is_empty() {
            info.push_str("├─────────────────────────\n");
            info.push_str("│ 任務目標:\n");
            for condition in &self.conditions {
                info.push_str(&format!("│  {}\n", condition.description()));
            }
        }
        
        // 任務獎勵
        if !self.rewards.is_empty() {
            info.push_str("├─────────────────────────\n");
            info.push_str("│ 任務獎勵:\n");
            for reward in &self.rewards {
                info.push_str(&format!("│  • {}\n", reward.description()));
            }
        }
        
        info.push_str("└─────────────────────────\n");
        
        info
    }
}

/// 任務管理器
#[derive(Clone, Serialize, Deserialize)]
pub struct QuestManager {
    /// 所有任務（任務 ID -> 任務）
    pub quests: HashMap<String, Quest>,
    
    /// 已完成的任務 ID 列表
    pub completed_quests: Vec<String>,
}

impl Default for QuestManager {
    fn default() -> Self {
        Self::new()
    }
}

impl QuestManager {
    pub fn new() -> Self {
        QuestManager {
            quests: HashMap::new(),
            completed_quests: Vec::new(),
        }
    }
    
    /// 添加任務
    pub fn add_quest(&mut self, quest: Quest) {
        self.quests.insert(quest.id.clone(), quest);
    }
    
    /// 獲取任務
    pub fn get_quest(&self, quest_id: &str) -> Option<&Quest> {
        self.quests.get(quest_id)
    }
    
    /// 獲取可變任務
    #[allow(dead_code)]
    pub fn get_quest_mut(&mut self, quest_id: &str) -> Option<&mut Quest> {
        self.quests.get_mut(quest_id)
    }
    
    /// 開始任務
    pub fn start_quest(&mut self, quest_id: &str) -> Result<String, String> {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            // 檢查前置任務
            if !quest.can_start(&self.completed_quests) {
                return Err(format!("未完成前置任務: {:?}", quest.prerequisites));
            }
            
            // 檢查狀態
            if quest.status == QuestStatus::InProgress {
                return Err("任務已經在進行中".to_string());
            }
            
            if quest.status == QuestStatus::Completed && !quest.repeatable {
                return Err("任務已完成且不可重複".to_string());
            }
            
            quest.status = QuestStatus::InProgress;
            Ok(format!("開始任務: {}", quest.name))
        } else {
            Err(format!("找不到任務: {quest_id}"))
        }
    }
    
    /// 完成任務
    pub fn complete_quest(&mut self, quest_id: &str) -> Result<Vec<QuestReward>, String> {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if quest.status != QuestStatus::InProgress {
                return Err("任務未在進行中".to_string());
            }
            
            if !quest.check_conditions() {
                return Err("任務條件未全部完成".to_string());
            }
            
            quest.status = QuestStatus::Completed;
            
            // 添加到已完成列表
            if !self.completed_quests.contains(&quest.id) {
                self.completed_quests.push(quest.id.clone());
            }
            
            Ok(quest.rewards.clone())
        } else {
            Err(format!("找不到任務: {quest_id}"))
        }
    }
    
    /// 放棄任務
    pub fn abandon_quest(&mut self, quest_id: &str) -> Result<String, String> {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if quest.status != QuestStatus::InProgress {
                return Err("只能放棄進行中的任務".to_string());
            }
            
            quest.status = QuestStatus::NotStarted;
            Ok(format!("已放棄任務: {}", quest.name))
        } else {
            Err(format!("找不到任務: {quest_id}"))
        }
    }
    
    /// 獲取所有進行中的任務
    pub fn get_active_quests(&self) -> Vec<&Quest> {
        self.quests.values()
            .filter(|q| q.status == QuestStatus::InProgress)
            .collect()
    }
    
    /// 獲取所有可接取的任務
    pub fn get_available_quests(&self) -> Vec<&Quest> {
        self.quests.values()
            .filter(|q| {
                (q.status == QuestStatus::NotStarted || (q.status == QuestStatus::Completed && q.repeatable))
                    && q.can_start(&self.completed_quests)
            })
            .collect()
    }
    
    /// 獲取所有已完成的任務
    pub fn get_completed_quests(&self) -> Vec<&Quest> {
        self.quests.values()
            .filter(|q| q.status == QuestStatus::Completed)
            .collect()
    }
    
    /// 從目錄載入所有任務
    pub fn load_from_directory(&mut self, quest_dir: &str) -> Result<usize, Box<dyn std::error::Error>> {
        use std::fs;
        
        fs::create_dir_all(quest_dir)?;
        let mut loaded_count = 0;
        
        if let Ok(entries) = fs::read_dir(quest_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let content = fs::read_to_string(&path)?;
                    let quest: Quest = serde_json::from_str(&content)?;
                    self.add_quest(quest);
                    loaded_count += 1;
                }
            }
        }
        
        Ok(loaded_count)
    }
    
    /// 保存所有任務
    pub fn save_to_directory(&self, quest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        
        fs::create_dir_all(quest_dir)?;
        
        for quest in self.quests.values() {
            let filename = format!("{}/{}.json", quest_dir, quest.id);
            let content = serde_json::to_string_pretty(quest)?;
            fs::write(filename, content)?;
        }
        
        Ok(())
    }
    
    /// 檢查任務條件是否已滿足（由外部系統調用）
    #[allow(dead_code)]
    pub fn check_quest_progress(&mut self, quest_id: &str) -> bool {
        if let Some(quest) = self.quests.get(quest_id) {
            quest.status == QuestStatus::InProgress && quest.check_conditions()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_creation() {
        let quest = Quest {
            id: "test_quest".to_string(),
            name: "測試任務".to_string(),
            description: "這是一個測試任務".to_string(),
            prerequisites: vec![],
            conditions: vec![
                QuestCondition::TalkToNpc {
                    npc_id: "商人".to_string(),
                    completed: false,
                },
            ],
            rewards: vec![
                QuestReward::Item {
                    item: "金幣".to_string(),
                    count: 100,
                },
            ],
            status: QuestStatus::NotStarted,
            giver: Some("村長".to_string()),
            repeatable: false,
        };
        
        assert_eq!(quest.status, QuestStatus::NotStarted);
        assert!(!quest.check_conditions());
    }
    
    #[test]
    fn test_quest_manager() {
        let mut manager = QuestManager::new();
        
        let quest = Quest {
            id: "quest1".to_string(),
            name: "任務1".to_string(),
            description: "測試".to_string(),
            prerequisites: vec![],
            conditions: vec![],
            rewards: vec![],
            status: QuestStatus::NotStarted,
            giver: None,
            repeatable: false,
        };
        
        manager.add_quest(quest);
        
        assert!(manager.start_quest("quest1").is_ok());
        assert_eq!(manager.get_quest("quest1").unwrap().status, QuestStatus::InProgress);
    }
    
    #[test]
    fn test_quest_prerequisites() {
        let mut manager = QuestManager::new();
        
        let quest1 = Quest {
            id: "quest1".to_string(),
            name: "任務1".to_string(),
            description: "前置任務".to_string(),
            prerequisites: vec![],
            conditions: vec![],
            rewards: vec![],
            status: QuestStatus::Completed,
            giver: None,
            repeatable: false,
        };
        
        let quest2 = Quest {
            id: "quest2".to_string(),
            name: "任務2".to_string(),
            description: "需要完成任務1".to_string(),
            prerequisites: vec!["quest1".to_string()],
            conditions: vec![],
            rewards: vec![],
            status: QuestStatus::NotStarted,
            giver: None,
            repeatable: false,
        };
        
        manager.add_quest(quest1);
        manager.add_quest(quest2);
        manager.completed_quests.push("quest1".to_string());
        
        assert!(manager.start_quest("quest2").is_ok());
    }
}
