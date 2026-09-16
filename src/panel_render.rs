//! 純文字(ASCII)面板渲染器
//!
//! 這些函式把遊戲世界渲染成純文字 `String`，供 FFI 推送到 host（iOS SwiftUI）。
//! 刻意**不依賴 ratatui/crossterm**（那些只在 `terminal-ui` feature 下存在，iOS 不會編入），
//! 只使用核心的 `GameWorld` / `Map` / `Person` / `TradeSystem` 等永遠可用的資料。

use std::collections::HashMap;

use crate::map::Map;
use crate::world::GameWorld;
use crate::npc_manager::NpcManager;
use crate::trade::TradeSystem;

/// 大地圖預設視窗大小。整張地圖是 100x100，直接吐 10,000 個字元給 host 既難讀
/// 又慢，所以跟終端版一樣改用「以玩家為中心的視窗」。
pub const DEFAULT_MAP_VIEW_WIDTH: usize = 41;
pub const DEFAULT_MAP_VIEW_HEIGHT: usize = 21;

/// 目前地圖上「座標 -> 顯示字元」的索引。
///
/// 以前每一格都呼叫一次 `get_npcs_at_in_map_excluding`，那是對全部 NPC 做線性
/// 掃描：一張 100x100 的地圖等於掃 10,000 次。改成先建一次索引再查表。
fn npc_char_index(world: &GameWorld) -> HashMap<(usize, usize), char> {
    let mut index = HashMap::new();
    for npc_id in world.npc_manager.get_all_npc_ids() {
        if npc_id == world.current_controlled_id {
            continue;
        }
        let Some(npc) = world.npc_manager.get_npc(&npc_id) else { continue };
        if npc.map != world.current_map_name {
            continue;
        }
        // 同一格有多個 NPC 時，商人優先顯示（跟終端版的小地圖一致）
        let ch = NpcManager::get_display_char(&npc.name);
        index
            .entry((npc.x, npc.y))
            .and_modify(|existing| {
                if ch == 'M' {
                    *existing = ch;
                }
            })
            .or_insert(ch);
    }
    index
}

/// 取得單一格的顯示字元。
/// 優先序：玩家(@) > NPC(顯示字元) > 物品(i) > 可走(.) > 牆(#) > 界外(?)
fn cell_char(
    map: &Map,
    npcs: &HashMap<(usize, usize), char>,
    x: usize,
    y: usize,
    player_x: usize,
    player_y: usize,
) -> char {
    if x == player_x && y == player_y {
        return '@';
    }

    if let Some(ch) = npcs.get(&(x, y)) {
        return *ch;
    }

    match map.get_point(x, y) {
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

/// 小地圖：以玩家為中心的視窗（21 寬 x 11 高）
pub fn render_minimap(world: &GameWorld) -> String {
    render_centered(world, "⊕ 小地圖", 21, 11)
}

/// 大地圖：以玩家為中心的預設視窗
pub fn render_map(world: &GameWorld) -> String {
    render_map_view(world, DEFAULT_MAP_VIEW_WIDTH, DEFAULT_MAP_VIEW_HEIGHT)
}

/// 大地圖：以玩家為中心、host 指定大小的視窗。
/// 寬/高傳 0 以下時使用預設值；超過地圖尺寸時自動夾到地圖大小。
pub fn render_map_view(world: &GameWorld, width: usize, height: usize) -> String {
    let width = if width == 0 { DEFAULT_MAP_VIEW_WIDTH } else { width };
    let height = if height == 0 { DEFAULT_MAP_VIEW_HEIGHT } else { height };
    render_centered(world, "▣ 地圖", width, height)
}

/// 整張地圖（除錯用；100x100 會產生上萬個字元，一般 host 不該直接顯示）
#[allow(dead_code)]
pub fn render_map_full(world: &GameWorld) -> String {
    let Some(player) = world.npc_manager.get_npc(&world.current_controlled_id) else {
        return "（找不到玩家角色）".to_string();
    };
    let Some(map) = world.get_current_map() else {
        return "（當前沒有地圖）".to_string();
    };
    let npcs = npc_char_index(world);

    let mut out = String::with_capacity(map.width * map.height + map.height + 128);
    out.push_str(&format!(
        "▣ 地圖 — {} [{}x{}] 你在 ({}, {})\n",
        map.name, map.width, map.height, player.x, player.y
    ));

    for y in 0..map.height {
        for x in 0..map.width {
            out.push(cell_char(map, &npcs, x, y, player.x, player.y));
        }
        out.push('\n');
    }

    out.push_str(legend());
    out
}

/// 以玩家為中心、指定大小的網格
fn render_centered(world: &GameWorld, title: &str, width: usize, height: usize) -> String {
    let Some(player) = world.npc_manager.get_npc(&world.current_controlled_id) else {
        return "（找不到玩家角色）".to_string();
    };
    let Some(map) = world.get_current_map() else {
        return "（當前沒有地圖）".to_string();
    };
    let npcs = npc_char_index(world);

    // 視窗不超過地圖大小，並夾在地圖邊界內，這樣玩家走到角落時畫面不會整片是 '?'
    let view_w = width.min(map.width).max(1);
    let view_h = height.min(map.height).max(1);
    let start_x = player.x.saturating_sub(view_w / 2).min(map.width - view_w);
    let start_y = player.y.saturating_sub(view_h / 2).min(map.height - view_h);

    let mut out = String::with_capacity(view_w * view_h + view_h + 128);
    out.push_str(&format!(
        "{} — {} [{}x{}] 你在 ({}, {})\n",
        title, map.name, map.width, map.height, player.x, player.y
    ));

    for y in start_y..start_y + view_h {
        for x in start_x..start_x + view_w {
            out.push(cell_char(map, &npcs, x, y, player.x, player.y));
        }
        out.push('\n');
    }

    out.push_str(legend());
    out
}

/// 地圖視窗的結構化 JSON（給想自己畫格子、而不是顯示 ASCII 的 host）。
///
/// 格式：
/// ```json
/// {"map":"beginMap","width":100,"height":100,
///  "view":{"x":30,"y":20,"width":41,"height":21},
///  "player":{"x":50,"y":40},
///  "rows":["..#..","....."],
///  "npcs":[{"id":"merchant","name":"商人","x":51,"y":40,"char":"M"}],
///  "legend":"..."}
/// ```
pub fn render_map_json(world: &GameWorld, width: usize, height: usize) -> String {
    let width = if width == 0 { DEFAULT_MAP_VIEW_WIDTH } else { width };
    let height = if height == 0 { DEFAULT_MAP_VIEW_HEIGHT } else { height };

    let Some(player) = world.npc_manager.get_npc(&world.current_controlled_id) else {
        return serde_json::json!({ "error": "找不到玩家角色" }).to_string();
    };
    let Some(map) = world.get_current_map() else {
        return serde_json::json!({ "error": "當前沒有地圖" }).to_string();
    };
    let npc_index = npc_char_index(world);

    let view_w = width.min(map.width).max(1);
    let view_h = height.min(map.height).max(1);
    let start_x = player.x.saturating_sub(view_w / 2).min(map.width - view_w);
    let start_y = player.y.saturating_sub(view_h / 2).min(map.height - view_h);

    let rows: Vec<String> = (start_y..start_y + view_h)
        .map(|y| {
            (start_x..start_x + view_w)
                .map(|x| cell_char(map, &npc_index, x, y, player.x, player.y))
                .collect()
        })
        .collect();

    // 只回傳視窗內的 NPC，host 才能標名字
    let npcs: Vec<serde_json::Value> = world
        .npc_manager
        .get_all_npc_ids()
        .into_iter()
        .filter_map(|id| world.npc_manager.get_npc(&id).map(|npc| (id, npc)))
        .filter(|(id, npc)| {
            *id != world.current_controlled_id
                && npc.map == world.current_map_name
                && (start_x..start_x + view_w).contains(&npc.x)
                && (start_y..start_y + view_h).contains(&npc.y)
        })
        .map(|(id, npc)| {
            serde_json::json!({
                "id": id,
                "name": npc.name,
                "x": npc.x,
                "y": npc.y,
                "char": NpcManager::get_display_char(&npc.name).to_string(),
            })
        })
        .collect();

    serde_json::json!({
        "map": map.name,
        "description": map.description,
        "width": map.width,
        "height": map.height,
        "view": { "x": start_x, "y": start_y, "width": view_w, "height": view_h },
        "player": { "x": player.x, "y": player.y },
        "rows": rows,
        "npcs": npcs,
        "legend": legend(),
    })
    .to_string()
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
///
/// 回傳的是 NPC **ID**，不是顯示名稱——世界裡有多個 NPC 同名（`merchant` 和
/// `商人` 的 name 都是「商人」），拿名稱當 key 會買賣到隔壁那個人身上。
pub fn resolve_trade_npc(world: &GameWorld, npc_id: &str) -> Option<String> {
    if !npc_id.is_empty() {
        return world.npc_manager.resolve_id(npc_id);
    }

    let player = world.npc_manager.get_npc(&world.current_controlled_id)?;
    let here: Vec<(String, &crate::person::Person)> = world
        .npc_manager
        .get_npcs_with_ids_at_in_map(&player.map, player.x, player.y)
        .into_iter()
        .filter(|(id, _)| id != &world.current_controlled_id)
        .collect();

    // 優先商人，否則取第一個
    here.iter()
        .find(|(_, npc)| npc.name == "商人")
        .or_else(|| here.first())
        .map(|(id, _)| id.clone())
}

/// 交易面板：列出商人商品（買價）與玩家可售物品（賣價）
pub fn render_trade(world: &GameWorld, npc_id: &str) -> String {
    let Some(resolved_id) = resolve_trade_npc(world, npc_id) else {
        return "（這裡沒有可以交易的對象。走到商人所在的格子再試試。）".to_string();
    };

    let Some(npc) = world.npc_manager.get_npc(&resolved_id) else {
        return format!("（找不到交易對象：{resolved_id}）");
    };
    let npc_name = npc.name.clone();
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

