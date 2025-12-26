use std::collections::HashMap;
use once_cell::sync::Lazy;

// 物品效果類型
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum ItemEffect {
    RestoreHp(i32),      // 恢復 HP
    RestoreMp(i32),      // 恢復 MP
    IncreaseMaxHp(i32),  // 增加最大 HP
    IncreaseMaxMp(i32),  // 增加最大 MP
    IncreaseStrength(i32), // 增加力量
    IncreaseKnowledge(i32), // 增加知識
    IncreaseSociality(i32), // 增加交誼
    ChangeSex(String), // 改變性別
    IncreaseAppearance(i32), // 改善外貌
    DecreaseAppearance(i32), // 降低外貌
}

/// 物品效果註冊表
static ITEM_EFFECTS: Lazy<HashMap<&'static str, Vec<ItemEffect>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // 食物效果
    m.insert("蘋果", vec![ItemEffect::RestoreHp(300)]);
    m.insert("麵包", vec![ItemEffect::RestoreHp(500)]);
    m.insert("乾肉", vec![ItemEffect::RestoreHp(800)]);
    m.insert("漿果", vec![ItemEffect::RestoreHp(200)]);
    
    // 藥水效果
    m.insert("治療藥水", vec![ItemEffect::RestoreHp(1000)]);
    m.insert("魔力藥水", vec![ItemEffect::RestoreMp(1000)]);
    m.insert("potion", vec![ItemEffect::RestoreHp(1000)]);
    m.insert("mana", vec![ItemEffect::RestoreMp(1000)]);

    // 咒泉鄉效果
    m.insert("娘溺泉", vec![ItemEffect::ChangeSex("女".to_string())]);
    m.insert("男溺泉", vec![ItemEffect::ChangeSex("男".to_string())]);
    m.insert("雞溺泉", vec![ItemEffect::ChangeSex("雞".to_string())]);
    m.insert("牛溺泉", vec![ItemEffect::ChangeSex("牛".to_string())]);
    m.insert("豬溺泉", vec![ItemEffect::ChangeSex("豬".to_string())]);
    
    // 魔法書效果
    m.insert("美容果", vec![ItemEffect::IncreaseAppearance(5)]);
    m.insert("變醜果", vec![ItemEffect::DecreaseAppearance(5)]);

    m
});

/// 獲取物品的效果
pub fn get_item_effects(item_name: &str) -> Option<&Vec<ItemEffect>> {
    ITEM_EFFECTS.get(item_name)
}

/// 檢查物品是否可使用
pub fn is_usable(item_name: &str) -> bool {
    ITEM_EFFECTS.contains_key(item_name)
}

/// 物品名稱映射表（英文 -> 中文）
static ITEM_NAME_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // 雜物
    m.insert("cloth", "舊布料");
    m.insert("stone", "石子");
    m.insert("bark", "樹皮");
    m.insert("feather", "羽毛");
    
    // 食物
    m.insert("apple", "蘋果");
    m.insert("bread", "麵包");
    m.insert("jerky", "乾肉");
    m.insert("berry", "漿果");
    
    // 武器
    m.insert("sword", "木劍");
    m.insert("iron_sword", "鐵劍");
    m.insert("bow", "弓");
    m.insert("dagger", "匕首");
    
    // 裝備
    m.insert("leather", "皮衣");
    m.insert("helmet", "頭盔");
    m.insert("shield", "盾牌");
    
    // 消耗品
    m.insert("potion", "治療藥水");
    m.insert("mana", "魔力藥水");
    m.insert("poison", "毒藥");
    
    // 工具
    m.insert("torch", "火把");
    m.insert("rope", "繩索");
    m.insert("pickaxe", "鎬");
    m.insert("key", "鑰匙");
    
    // 其他
    m.insert("book", "魔法書");
    m.insert("magic_book", "魔法書");
    m.insert("gold", "金幣");
    m.insert("coin", "金幣");
    
    m
});

/// 食物的 HP 回復值映射表
static FOOD_HP_MAP: Lazy<HashMap<&'static str, i32>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    m.insert("蘋果", 300);
    m.insert("麵包", 500);
    m.insert("乾肉", 800);
    m.insert("漿果", 200);
    
    m
});

/// 檢查物品是否為食物
pub fn is_food(item_name: &str) -> bool {
    FOOD_HP_MAP.contains_key(item_name)
}

/// 獲取食物的 HP 回復值
pub fn get_food_hp(item_name: &str) -> Option<i32> {
    FOOD_HP_MAP.get(item_name).copied()
}

/// 將輸入的名稱（可能是英文或中文）轉換為統一的中文名稱
pub fn resolve_item_name(input: &str) -> String {
    // 先轉小寫
    let input_lower = input.to_lowercase();
    
    // 檢查是否是英文名稱
    if let Some(&chinese_name) = ITEM_NAME_MAP.get(input_lower.as_str()) {
        return chinese_name.to_string();
    }
    
    // 如果不是英文名稱，檢查是否已經是中文名稱
    for (&_eng, &chi) in ITEM_NAME_MAP.iter() {
        if chi == input {
            return input.to_string();
        }
    }
    
    // 都不是，返回原始輸入
    input.to_string()
}

/// 獲取物品的顯示名稱（中文+英文）
pub fn get_item_display_name(chinese_name: &str) -> String {
    // 找到對應的英文名稱
    for (&eng, &chi) in ITEM_NAME_MAP.iter() {
        if chi == chinese_name {
            return format!("{chinese_name} ({eng})");
        }
    }
    chinese_name.to_string()
}
