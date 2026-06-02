//! 純文字(ASCII)面板渲染器
//!
//! 這些函式把遊戲世界渲染成純文字 `String`，供 FFI 推送到 host（iOS SwiftUI）。
//! 刻意**不依賴 ratatui/crossterm**（那些只在 `terminal-ui` feature 下存在，iOS 不會編入），
//! 只使用核心的 `GameWorld` / `Map` / `Person` / `TradeSystem` 等永遠可用的資料。

use crate::world::GameWorld;
use crate::npc_manager::NpcManager;
use crate::trade::TradeSystem;

/// 取得玩家所在格上某座標的顯示字元。
/// 優先序：玩家(@) > NPC(顯示字元) > 物品(i) > 可走(.) > 牆(#) > 界外(?)
fn cell_char(world: &GameWorld, x: usize, y: usize, player_x: usize, player_y: usize) -> char {
    if x == player_x && y == player_y {
        return '@';
    }

    // 該格上的 NPC（排除玩家本身）
    let npcs = world.npc_manager.get_npcs_at_in_map_excluding(
        &world.current_map_name,
        x,
        y,
        &world.current_controlled_id,
    );
    if let Some(npc) = npcs.first() {
        return NpcManager::get_display_char(&npc.name);
    }

    match world.get_current_map().and_then(|m| m.get_point(x, y)) {
        Some(point) => {
            if !point.objects.is_empty() {
                'i'
            } else if point.walkable {
                '.'
            } else {
                '#'
            }
        }
        None => '?',
    }
}

/// 圖例字串（所有面板共用）
fn legend() -> &'static str {
    "圖例: @你 M商人 N居民 i物品 .可走 #牆 ?界外"
}

/// 小地圖：以玩家為中心的視窗（預設 21 寬 x 11 高）
pub fn render_minimap(world: &GameWorld) -> String {
    render_centered(world, 21, 11)
}

/// 以玩家為中心、指定大小的網格
fn render_centered(world: &GameWorld, width: i64, height: i64) -> String {
    let Some(player) = world.npc_manager.get_npc(&world.current_controlled_id) else {
        return "（找不到玩家角色）".to_string();
    };
    let (px, py) = (player.x as i64, player.y as i64);

    let mut out = String::new();
    out.push_str(&format!("⊕ 小地圖 — {} ({}, {})\n", world.current_map_name, px, py));

    let half_w = width / 2;
    let half_h = height / 2;

    for dy in -half_h..=half_h {
        for dx in -half_w..=half_w {
            let cx = px + dx;
            let cy = py + dy;
            if cx < 0 || cy < 0 {
                out.push('?');
            } else {
                out.push(cell_char(world, cx as usize, cy as usize, player.x, player.y));
            }
        }
        out.push('\n');
    }

    out.push_str(legend());
    out
}

/// 大地圖：整張當前地圖
pub fn render_map(world: &GameWorld) -> String {
    let Some(player) = world.npc_manager.get_npc(&world.current_controlled_id) else {
        return "（找不到玩家角色）".to_string();
    };
    let Some(map) = world.get_current_map() else {
        return "（當前沒有地圖）".to_string();
    };

    let mut out = String::new();
    out.push_str(&format!(
        "▣ 地圖 — {} [{}x{}] 你在 ({}, {})\n",
        map.name, map.width, map.height, player.x, player.y
    ));

    for y in 0..map.height {
        for x in 0..map.width {
            out.push(cell_char(world, x, y, player.x, player.y));
        }
        out.push('\n');
    }

    out.push_str(legend());
    out
}

/// 背包：玩家持有物品 + 金幣 + 體力/精神
pub fn render_inventory(world: &GameWorld) -> String {
    let Some(player) = world.npc_manager.get_npc(&world.current_controlled_id) else {
        return "（找不到玩家角色）".to_string();
    };

    let mut out = String::new();
    out.push_str("🎒 背包\n");
    out.push_str("─────────────────────────\n");

    let gold = player.items.get("金幣").copied().unwrap_or(0);
    out.push_str(&format!("金幣: {gold}\n"));
    out.push_str(&format!("HP: {}/{}   MP: {}/{}\n", player.hp, player.max_hp, player.mp, player.max_mp));
    out.push_str("─────────────────────────\n");

    let mut items: Vec<(&String, &u32)> = player
        .items
        .iter()
        .filter(|(name, qty)| name.as_str() != "金幣" && **qty > 0)
        .collect();
    items.sort_by(|a, b| a.0.cmp(b.0));

    if items.is_empty() {
        out.push_str("（沒有任何物品）\n");
    } else {
        for (name, qty) in items {
            let display = crate::item_registry::get_item_display_name(name);
            out.push_str(&format!("  • {display} x{qty}\n"));
        }
    }

    out
}

/// 角色狀態：直接沿用 Person::show_detail()
pub fn render_status(world: &GameWorld) -> String {
    match world.npc_manager.get_npc(&world.current_controlled_id) {
        Some(player) => player.show_detail(),
        None => "（找不到玩家角色）".to_string(),
    }
}

/// 解析交易對象：npc_id 為空時，自動挑玩家所在格的 NPC（優先「商人」）。
/// 回傳 NPC 名稱（同時可作為 get_npc 的 key）。
pub fn resolve_trade_npc(world: &GameWorld, npc_id: &str) -> Option<String> {
    if !npc_id.is_empty() {
        return world.npc_manager.get_npc(npc_id).map(|n| n.name.clone());
    }

    let Some(player) = world.npc_manager.get_npc(&world.current_controlled_id) else {
        return None;
    };
    let here = world.npc_manager.get_npcs_at_in_map_excluding(
        &player.map,
        player.x,
        player.y,
        &world.current_controlled_id,
    );
    if here.is_empty() {
        return None;
    }
    // 優先商人，否則取第一個
    here.iter()
        .find(|n| n.name == "商人")
        .or_else(|| here.first())
        .map(|n| n.name.clone())
}

/// 交易面板：列出商人商品（買價）與玩家可售物品（賣價）
pub fn render_trade(world: &GameWorld, npc_id: &str) -> String {
    let Some(npc_name) = resolve_trade_npc(world, npc_id) else {
        return "（這裡沒有可以交易的對象。走到商人所在的格子再試試。）".to_string();
    };

    let Some(npc) = world.npc_manager.get_npc(&npc_name) else {
        return format!("（找不到交易對象：{npc_name}）");
    };
    let Some(player) = world.npc_manager.get_npc(&world.current_controlled_id) else {
        return "（找不到玩家角色）".to_string();
    };

    let player_gold = player.items.get("金幣").copied().unwrap_or(0);

    let mut out = String::new();
    out.push_str(&format!("🤝 與 {npc_name} 交易    你的金幣: {player_gold}\n"));
    out.push_str("═════════════════════════\n");

    out.push_str("【可購買】(名稱 / 庫存 / 買價)\n");
    let goods = TradeSystem::get_npc_goods(npc);
    if goods.is_empty() {
        out.push_str("  （商人沒有可賣的東西）\n");
    } else {
        for (name, qty, price) in goods {
            let display = crate::item_registry::get_item_display_name(&name);
            out.push_str(&format!("  • {display}  x{qty}  {price}金/個\n"));
        }
    }

    out.push_str("─────────────────────────\n");
    out.push_str("【可出售】(名稱 / 持有 / 賣價)\n");
    let sellable = TradeSystem::get_player_items(player);
    if sellable.is_empty() {
        out.push_str("  （你沒有可賣的東西）\n");
    } else {
        for (name, qty, price) in sellable {
            let display = crate::item_registry::get_item_display_name(&name);
            out.push_str(&format!("  • {display}  x{qty}  {price}金/個\n"));
        }
    }

    out
}
