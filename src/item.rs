use serde::{Serialize, Deserialize};
use rand::Rng;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// 全局物品流水號生成器
static ITEM_ID_GENERATOR: Lazy<Mutex<u64>> = Lazy::new(|| {
    Mutex::new(1)
});

/// 獲取下一個物品流水號
pub fn next_item_id() -> u64 {
    let mut id = ITEM_ID_GENERATOR.lock().unwrap();
    let current = *id;
    *id += 1;
    current
}

/// 設置當前流水號（用於載入存檔）
#[allow(dead_code)]
pub fn set_item_id_counter(value: u64) {
    let mut id = ITEM_ID_GENERATOR.lock().unwrap();
    *id = value;
}

/// 獲取當前流水號（用於保存存檔）
#[allow(dead_code)]
pub fn get_item_id_counter() -> u64 {
    *ITEM_ID_GENERATOR.lock().unwrap()
}

// 物品類型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum ItemType {
    Miscellaneous,  // 雜物
    Food,           // 食物
    Weapon,         // 武器
    Armor,          // 裝備
    Consumable,     // 消耗品
    Tool,           // 工具
    Magic,          // 魔法
    Fruit,          // 果實
}

impl ItemType {
    #[allow(dead_code)]
    pub fn describe(&self) -> &str {
        match self {
            ItemType::Miscellaneous => "雜物",
            ItemType::Food => "食物",
            ItemType::Weapon => "武器",
            ItemType::Armor => "裝備",
            ItemType::Consumable => "消耗品",
            ItemType::Tool => "工具",
            ItemType::Magic => "魔法",
            ItemType::Fruit => "果實",
        }
    }
}

// 物品定義（模板）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    #[serde(default)]
    pub english_name: Option<String>,  // 英文名稱或簡稱（可選，向後兼容）
    pub item_type: ItemType,
    pub description: String,
    pub value: u32,  // 物品價值
    #[serde(default)]
    pub age: u64,    // 物品存在時間（以秒計算）
    pub stories: Vec<String>,       // 物品的故事
}

// 物品實例（每個實例都有唯一的流水號）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemInstance {
    pub id: u64,          // 唯一流水號
    pub name: String,     // 物品名稱
    #[serde(default)]
    pub age: u64,         // 物品存在時間（以秒計算）
    #[serde(default)]
    pub stories: Vec<String>,  // 物品的故事
}

impl ItemInstance {
    /// 創建新物品實例
    pub fn new(name: String) -> Self {
        ItemInstance {
            id: next_item_id(),
            name,
            age: 0,
            stories: Vec::new(),
        }
    }
    
    /// 從已有流水號創建實例（用於載入存檔）
    #[allow(dead_code)]
    pub fn with_id(id: u64, name: String) -> Self {
        ItemInstance {
            id,
            name,
            age: 0,
            stories: Vec::new(),
        }
    }
}

impl Item {
    #[allow(dead_code)]
    pub fn new(name: String, english_name: String, item_type: ItemType, description: String, value: u32) -> Self {
        Item {
            name,
            english_name: Some(english_name),
            item_type,
            description,
            value,
            age: 0,
            stories: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn display(&self) -> String {
        if let Some(ref eng) = self.english_name {
            format!("{} ({}) [{}]", self.name, eng, self.item_type.describe())
        } else {
            format!("{} [{}]", self.name, self.item_type.describe())
        }
    }

    // 生成隨機物品
    #[allow(dead_code)]
    pub fn generate_random() -> Self {
        let mut rng = rand::thread_rng();
        
        let items = vec![
            // 雜物 (name, english_name, type, description, value)
            ("舊布料", "cloth", ItemType::Miscellaneous, "一塊破舊的布料", 5),
            ("石子", "stone", ItemType::Miscellaneous, "光滑的小石子", 2),
            ("樹皮", "bark", ItemType::Miscellaneous, "剝落的樹皮", 3),
            ("羽毛", "feather", ItemType::Miscellaneous, "柔軟的羽毛", 4),
            
            // 食物
            ("蘋果", "apple", ItemType::Food, "新鮮的紅蘋果", 10),
            ("麵包", "bread", ItemType::Food, "烤得金黃的麵包", 15),
            ("乾肉", "jerky", ItemType::Food, "風乾的肉乾", 20),
            ("漿果", "berry", ItemType::Food, "野生的紫色漿果", 8),
            
            // 武器
            ("木劍", "sword", ItemType::Weapon, "簡陋的木製劍", 30),
            ("鐵劍", "iron_sword", ItemType::Weapon, "鋒利的鐵劍", 100),
            ("弓", "bow", ItemType::Weapon, "木製的弓", 50),
            ("匕首", "dagger", ItemType::Weapon, "精緻的小匕首", 40),
            
            // 裝備
            ("皮衣", "leather", ItemType::Armor, "耐用的皮衣", 60),
            ("頭盔", "helmet", ItemType::Armor, "堅固的鐵頭盔", 80),
            ("盾牌", "shield", ItemType::Armor, "厚實的木盾", 70),
            
            // 消耗品
            ("治療藥水", "potion", ItemType::Consumable, "恢復體力的魔法藥水", 50),
            ("魔力藥水", "mana", ItemType::Consumable, "補充魔力的藍色藥水", 45),
            ("毒藥", "poison", ItemType::Consumable, "致命的紫色液體", 120),
            
            // 工具
            ("火把", "torch", ItemType::Tool, "點燃的木製火把", 25),
            ("繩索", "rope", ItemType::Tool, "粗糙的麻繩", 15),
            ("鎬", "pickaxe", ItemType::Tool, "採礦用的工具", 35),
            ("鑰匙", "key", ItemType::Tool, "古舊的金屬鑰匙", 40),

            // 魔法
            ("娘溺泉", "girl_spring", ItemType::Magic, "咒泉鄉的泉水，傳說從前有一個年輕女孩溺在此泉水裡,從此掉到此泉水裡的人都會變成女性", 150),
            ("男溺泉", "boy_spring", ItemType::Magic, "咒泉鄉的泉水，傳說從前有一個年輕男孩溺在此泉水裡,從此掉到此泉水裡的人都會變成男性", 130),
            ("牛溺泉", "ox_spring", ItemType::Magic, "咒泉鄉的泉水，傳說從前有一頭牛為了喝水不小心掉到此泉水裡,從此掉到此泉水裡的人都會變成牛", 140),
            ("豬溺泉", "pig_spring", ItemType::Magic, "咒泉鄉的泉水，傳說從前有一頭豬在這泉水裡洗澡洗到暈倒,從此掉到此泉水裡的人都會變成豬", 140),
            // 果實
            ("美容果", "beauty_fruit", ItemType::Fruit, "傳說吃了會變漂亮的神奇果實", 200),
            ("變醜果", "ugly_fruit", ItemType::Fruit, "傳說吃了會變醜的神奇果實", 180),
        ];
        
        let idx = rng.gen_range(0..items.len());
        let (name, english_name, item_type, description, value) = items[idx];
        Item::new(name.to_string(), english_name.to_string(), item_type, description.to_string(), value)
    }
}

// 實現 TimeUpdatable trait
use crate::time_updatable::{TimeUpdatable, TimeInfo};

impl TimeUpdatable for Item {
    fn on_time_update(&mut self, _current_time: &TimeInfo) {
        // 每秒增加年齡
        self.age += 1;
    }
}

impl TimeUpdatable for ItemInstance {
    fn on_time_update(&mut self, _current_time: &TimeInfo) {
        // 每秒增加年齡
        self.age += 1;
    }
}
