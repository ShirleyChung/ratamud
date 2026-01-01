use crate::person::Person;
use crate::world::GameWorld;
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// 物品價格系統（全局單例）
static ITEM_PRICES: Lazy<Mutex<HashMap<String, u32>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// 交易結果
pub enum TradeResult {
    Success(String),    // 成功，附帶訊息
    Failed(String),     // 失敗，附帶原因
}

/// 交易系統
pub struct TradeSystem;

impl TradeSystem {
    /// 玩家向 NPC 購買物品
    /// price: 購買價格（金幣數量）
    /// 玩家從 NPC 購買物品
    /// world: 遊戲世界
    /// me: 當前玩家
    /// npc_id: NPC ID
    /// item_name: 物品名稱
    /// quantity: 數量
    /// price: 購買價格
    pub fn buy_from_npc(
        world: &mut GameWorld,
        me: &mut Person,
        npc_id: &str,
        item_name: &str,
        quantity: u32,
        price: u32,
    ) -> TradeResult {
        let npc_option = world.npc_manager.get_npc_mut(npc_id);

        if npc_option.is_none() {
            return TradeResult::Failed("找不到指定的商人".to_string());
        }
        let npc = npc_option.unwrap();

        // 檢查 NPC 是否有足夠的物品
        let npc_has = npc.items.get(item_name).copied().unwrap_or(0);
        if npc_has < quantity {
            return TradeResult::Failed(format!(
                "{} 沒有足夠的 {}（只有 {}）",
                npc.name, item_name, npc_has
            ));
        }
        
        // 檢查玩家是否有足夠的金幣
        let player_gold = me.items.get("金幣").copied().unwrap_or(0);
        if player_gold < price {
            return TradeResult::Failed(format!(
                "你沒有足夠的金幣（需要 {price}，只有 {player_gold}）"
            ));
        }
        
        // 執行交易
        // 1. 玩家扣除金幣
        let player_gold_entry = me.items.entry("金幣".to_string()).or_insert(0);
        *player_gold_entry -= price;
        if *player_gold_entry == 0 {
            me.items.remove("金幣");
        }
        
        // 2. NPC 獲得金幣
        *npc.items.entry("金幣".to_string()).or_insert(0) += price;
        
        // 3. NPC 扣除物品
        let npc_item_entry = npc.items.get_mut(item_name).unwrap();
        *npc_item_entry -= quantity;
        if *npc_item_entry == 0 {
            npc.items.remove(item_name);
        }
        
        // 4. 玩家獲得物品
        *me.items.entry(item_name.to_string()).or_insert(0) += quantity;
        
        TradeResult::Success(format!(
            "你花費 {} 金幣從 {} 購買了 {} x{}",
            price, npc.name, item_name, quantity
        ))
    }
    
    /// 玩家向 NPC 出售物品
    /// world: 遊戲世界
    /// me: 當前玩家
    /// npc_id: NPC ID
    /// item_name: 物品名稱
    /// quantity: 數量
    /// price: 出售價格（金幣數量）
    pub fn sell_to_npc(
        world: &mut GameWorld,
        me: &mut Person,
        npc_id: &str,
        item_name: &str,
        quantity: u32,
        price: u32,
    ) -> TradeResult {
        let npc_option = world.npc_manager.get_npc_mut(npc_id);

        if npc_option.is_none() {
            return TradeResult::Failed("找不到指定的商人".to_string());
        }
        let npc = npc_option.unwrap();

        // 檢查玩家是否有足夠的物品
        let player_has = me.items.get(item_name).copied().unwrap_or(0);
        if player_has < quantity {
            return TradeResult::Failed(format!(
                "你沒有足夠的 {item_name}（只有 {player_has}）"
            ));
        }
        
        // 檢查 NPC 是否有足夠的金幣
        let npc_gold = npc.items.get("金幣").copied().unwrap_or(0);
        if npc_gold < price {
            return TradeResult::Failed(format!(
                "{} 沒有足夠的金幣購買（需要 {}，只有 {}）",
                npc.name, price, npc_gold
            ));
        }
        
        // 執行交易
        // 1. NPC 扣除金幣
        let npc_gold_entry = npc.items.get_mut("金幣").unwrap();
        *npc_gold_entry -= price;
        if *npc_gold_entry == 0 {
            npc.items.remove("金幣");
        }
        
        // 2. 玩家獲得金幣
        *me.items.entry("金幣".to_string()).or_insert(0) += price;
        
        // 3. 玩家扣除物品
        let player_item_entry = me.items.get_mut(item_name).unwrap();
        *player_item_entry -= quantity;
        if *player_item_entry == 0 {
            me.items.remove(item_name);
        }
        
        // 4. NPC 獲得物品
        *npc.items.entry(item_name.to_string()).or_insert(0) += quantity;
        
        TradeResult::Success(format!(
            "你以 {} 金幣向 {} 出售了 {} x{}",
            price, npc.name, item_name, quantity
        ))
    }
    
    /// 設置物品價格
    pub fn set_item_price(item_name: &str, price: u32) {
        let mut prices = ITEM_PRICES.lock().unwrap();
        prices.insert(item_name.to_string(), price);
    }
    
    /// 獲取物品的基礎價格（用於計算買賣價）
    /// 優先從價格系統獲取，否則返回預設值 10
    pub fn get_item_base_price(item_name: &str) -> u32 {
        let prices = ITEM_PRICES.lock().unwrap();
        
        // 先檢查價格系統
        if let Some(&price) = prices.get(item_name) {
            return price;
        }
        
        // 檢查別名（英文名稱）
        let item_lower = item_name.to_lowercase();
        if let Some(&price) = prices.get(&item_lower) {
            return price;
        }
        
        // 預設價格為 10
        10
    }
    
    /// 計算購買價格（NPC 賣給玩家）
    /// 通常是基礎價格的 1.5 倍
    pub fn calculate_buy_price(item_name: &str, quantity: u32) -> u32 {
        let base = Self::get_item_base_price(item_name);
        (base as f32 * 1.5) as u32 * quantity
    }
    
    /// 計算出售價格（玩家賣給 NPC）
    /// 通常是基礎價格的 0.7 倍
    pub fn calculate_sell_price(item_name: &str, quantity: u32) -> u32 {
        let base = Self::get_item_base_price(item_name);
        (base as f32 * 0.7) as u32 * quantity
    }
    
    /// 顯示 NPC 的商品列表
    pub fn get_npc_goods(npc: &Person) -> Vec<(String, u32, u32)> {
        // 返回 (物品名稱, 數量, 購買價格)
        let mut goods = Vec::new();
        
        for (item_name, quantity) in &npc.items {
            if item_name != "金幣" && *quantity > 0 {
                let price = Self::calculate_buy_price(item_name, 1);
                goods.push((item_name.clone(), *quantity, price));
            }
        }
        
        goods.sort_by(|a, b| a.0.cmp(&b.0));
        goods
    }
    
    /// 獲取玩家持有的物品列表（用於出售）
    pub fn get_player_items(player: &Person) -> Vec<(String, u32, u32)> {
        // 返回 (物品名稱, 數量, 出售價格)
        let mut items = Vec::new();
        
        for (item_name, quantity) in &player.items {
            // 排除金幣，只顯示可以出售的物品
            if item_name != "金幣" && *quantity > 0 {
                let price = Self::calculate_sell_price(item_name, 1);
                items.push((item_name.clone(), *quantity, price));
            }
        }
        
        items.sort_by(|a, b| a.0.cmp(&b.0));
        items
    }
}
