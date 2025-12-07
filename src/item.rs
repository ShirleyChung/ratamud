use serde::{Serialize, Deserialize};
use rand::Rng;

// 物品類型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum ItemType {
    Miscellaneous,  // 雜物
    Food,           // 食物
    Weapon,         // 武器
    Armor,          // 裝備
    Consumable,     // 消耗品
    Tool,           // 工具
}

impl ItemType {
    pub fn describe(&self) -> &str {
        match self {
            ItemType::Miscellaneous => "雜物",
            ItemType::Food => "食物",
            ItemType::Weapon => "武器",
            ItemType::Armor => "裝備",
            ItemType::Consumable => "消耗品",
            ItemType::Tool => "工具",
        }
    }
}

// 物品類別
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub item_type: ItemType,
    pub description: String,
    pub value: u32,  // 物品價值
}

impl Item {
    pub fn new(name: String, item_type: ItemType, description: String, value: u32) -> Self {
        Item {
            name,
            item_type,
            description,
            value,
        }
    }

    pub fn display(&self) -> String {
        format!("{} ({})", self.name, self.item_type.describe())
    }

    // 生成隨機物品
    pub fn generate_random() -> Self {
        let mut rng = rand::thread_rng();
        
        let items = vec![
            // 雜物
            ("舊布料", ItemType::Miscellaneous, "一塊破舊的布料", 5),
            ("石子", ItemType::Miscellaneous, "光滑的小石子", 2),
            ("樹皮", ItemType::Miscellaneous, "剝落的樹皮", 3),
            ("羽毛", ItemType::Miscellaneous, "柔軟的羽毛", 4),
            
            // 食物
            ("蘋果", ItemType::Food, "新鮮的紅蘋果", 10),
            ("麵包", ItemType::Food, "烤得金黃的麵包", 15),
            ("乾肉", ItemType::Food, "風乾的肉乾", 20),
            ("漿果", ItemType::Food, "野生的紫色漿果", 8),
            
            // 武器
            ("木劍", ItemType::Weapon, "簡陋的木製劍", 30),
            ("鐵劍", ItemType::Weapon, "鋒利的鐵劍", 100),
            ("弓", ItemType::Weapon, "木製的弓", 50),
            ("匕首", ItemType::Weapon, "精緻的小匕首", 40),
            
            // 裝備
            ("皮衣", ItemType::Armor, "耐用的皮衣", 60),
            ("頭盔", ItemType::Armor, "堅固的鐵頭盔", 80),
            ("盾牌", ItemType::Armor, "厚實的木盾", 70),
            
            // 消耗品
            ("治療藥水", ItemType::Consumable, "恢復體力的魔法藥水", 50),
            ("魔力藥水", ItemType::Consumable, "補充魔力的藍色藥水", 45),
            ("毒藥", ItemType::Consumable, "致命的紫色液體", 120),
            
            // 工具
            ("火把", ItemType::Tool, "點燃的木製火把", 25),
            ("繩索", ItemType::Tool, "粗糙的麻繩", 15),
            ("鎬", ItemType::Tool, "採礦用的工具", 35),
            ("鑰匙", ItemType::Tool, "古舊的金屬鑰匙", 40),
        ];
        
        let idx = rng.gen_range(0..items.len());
        let (name, item_type, description, value) = items[idx];
        Item::new(name.to_string(), item_type.clone(), description.to_string(), value)
    }
}
