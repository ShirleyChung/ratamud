#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::core_output::{self, OutputZone};
use crate::world::GameWorld;

/// 全局遊戲世界實例（FFI 和其他非 UI 模式共用）
static GAME_WORLD: Lazy<Mutex<Option<GameWorld>>> = Lazy::new(|| Mutex::new(None));

/// host 目前開啟的面板（由 `ratamud_set_active_panel` 設定）。
/// 當世界因 NPC 行動或事件而改變時，引擎會自動重新渲染並推回這個面板。
static ACTIVE_PANEL: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

/// tick 計數器（用來節流 NPC AI，避免每秒都動）
static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

/// 輸出回調函數類型 (C FFI)
/// 參數: msg_type (類型標記: MAIN/LOG/STATUS/SIDE), content (內容)
pub type OutputCallback = extern "C" fn(*const c_char, *const c_char);
pub type StateCallback = extern "C" fn(*const c_char);
pub type EventCallback = extern "C" fn(*const c_char, *const c_char);
/// 面板回調：(panel_type, ascii_content)
/// panel_type 可能的值: "MAP", "MINIMAP", "INVENTORY", "STATUS", "TRADE"
pub type PanelCallback = extern "C" fn(*const c_char, *const c_char);

static STATE_CALLBACK: Lazy<Mutex<Option<StateCallback>>> = Lazy::new(|| Mutex::new(None));
static EVENT_CALLBACK: Lazy<Mutex<Option<EventCallback>>> = Lazy::new(|| Mutex::new(None));
static PANEL_CALLBACK: Lazy<Mutex<Option<PanelCallback>>> = Lazy::new(|| Mutex::new(None));

/// 註冊輸出回調（C FFI）
/// 當遊戲有新輸出時，會調用此回調
/// 
/// 回調函數簽名: fn(msg_type: *const c_char, content: *const c_char)
/// msg_type 可能的值: "MAIN", "LOG", "STATUS", "SIDE"
#[no_mangle]
pub extern "C" fn ratamud_register_output_callback(callback: OutputCallback) {
    core_output::register_output_callback(move |zone, content| {
        if let (Ok(type_c), Ok(content_c)) = (
            CString::new(zone.as_str()),
            CString::new(content)
        ) {
            callback(type_c.as_ptr(), content_c.as_ptr());
        }
    });
}

/// 清除輸出回調
#[no_mangle]
pub extern "C" fn ratamud_clear_output_callback() {
    core_output::clear_output_callback();
}

#[no_mangle]
pub extern "C" fn ratamud_register_state_callback(callback: StateCallback) {
    if let Ok(mut cb) = STATE_CALLBACK.lock() {
        *cb = Some(callback);
    }
}

#[no_mangle]
pub extern "C" fn ratamud_register_event_callback(callback: EventCallback) {
    if let Ok(mut cb) = EVENT_CALLBACK.lock() {
        *cb = Some(callback);
    }
}

/// 註冊面板回調（C FFI）
/// 當 host 請求某個面板（地圖/背包/交易…）時，引擎會以 ASCII 字串透過此回調推回。
#[no_mangle]
pub extern "C" fn ratamud_register_panel_callback(callback: PanelCallback) {
    if let Ok(mut cb) = PANEL_CALLBACK.lock() {
        *cb = Some(callback);
    }
}

/// 清除面板回調
#[no_mangle]
pub extern "C" fn ratamud_clear_panel_callback() {
    if let Ok(mut cb) = PANEL_CALLBACK.lock() {
        *cb = None;
    }
    if let Ok(mut active) = ACTIVE_PANEL.lock() {
        *active = None;
    }
}

/// 告訴引擎 host 目前開啟的是哪個面板（空字串 = 沒有開）。
/// 之後當世界因 NPC 行動或事件而改變時，引擎會自動重新渲染並透過 panel callback 推回該面板。
#[no_mangle]
pub extern "C" fn ratamud_set_active_panel(panel: *const c_char) {
    let name = cstr_to_string(panel);
    if let Ok(mut active) = ACTIVE_PANEL.lock() {
        *active = if name.is_empty() { None } else { Some(name) };
    }
}

/// 把面板內容推回 host
fn trigger_panel(panel: &str, content: &str) {
    let callback = match PANEL_CALLBACK.lock() {
        Ok(cb) => *cb,
        Err(_) => None,
    };

    if let Some(callback) = callback {
        if let (Ok(panel_c), Ok(content_c)) = (CString::new(panel), CString::new(content)) {
            callback(panel_c.as_ptr(), content_c.as_ptr());
        }
    }
}

/// 以閉包渲染當前世界並推回指定面板（共用鎖定樣板）
/// 回傳 0=成功, -1=失敗（無世界/鎖定失敗）
fn render_and_push<F>(panel: &str, render: F) -> c_int
where
    F: FnOnce(&GameWorld) -> String,
{
    let world_guard = match GAME_WORLD.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let game_world = match world_guard.as_ref() {
        Some(world) => world,
        None => return -1,
    };
    let content = render(game_world);
    drop(world_guard);
    trigger_panel(panel, &content);
    0
}

/// 依面板名稱渲染當前世界內容（給自動刷新用）
fn render_panel_by_name(world: &GameWorld, panel: &str) -> Option<String> {
    use crate::panel_render as pr;
    match panel {
        "MAP" => Some(pr::render_map(world)),
        "MINIMAP" => Some(pr::render_minimap(world)),
        "INVENTORY" => Some(pr::render_inventory(world)),
        "STATUS" => Some(pr::render_status(world)),
        // 交易面板會自行偵測當前格的商人
        "TRADE" => Some(pr::render_trade(world, "")),
        _ => None,
    }
}

/// 取得 host 目前開啟的面板名稱
fn current_active_panel() -> Option<String> {
    ACTIVE_PANEL.lock().ok().and_then(|guard| guard.clone())
}

/// 若有開啟的面板，渲染其最新內容並回傳 (panel, content)。
/// 渲染需在持有世界鎖時進行，推回（trigger_panel）則應在釋放鎖後執行。
fn active_panel_content(world: &GameWorld) -> Option<(String, String)> {
    let panel = current_active_panel()?;
    let content = render_panel_by_name(world, &panel)?;
    Some((panel, content))
}

/// 驅動所有 NPC 的 AI 一回合（FFI/iOS 沒有終端版的 AI 執行緒，必須由 tick 推動）。
/// 會套用移動／撿物等意圖，並把 NPC 對話與靠近/離開通知輸出到主畫面。
/// 回傳 true 表示世界有實際變動（需要刷新面板）。
fn drive_npc_ai(game_world: &mut GameWorld) -> bool {
    use crate::game_event::GameEvent;
    use crate::npc_action::NpcAction;
    use crate::npc_ai::NpcAiController;

    let ai = NpcAiController::new();
    let views = game_world.build_npc_views();
    let mut changed = false;

    for (npc_id, view) in views {
        let Some(action) = ai.decide_action(&view) else { continue };
        if matches!(action, NpcAction::Idle) {
            continue;
        }

        let messages = game_world.apply_event(GameEvent::NpcActions {
            npc_id,
            actions: vec![action],
        });

        for msg in &messages {
            // 移動之類的訊息屬於日誌，不洗主畫面；NPC 對話等才輸出
            if !msg.is_log() {
                core_output::trigger_output(OutputZone::Main, &msg.to_display_text());
            }
            changed = true;
        }
    }

    // 依玩家位置產生「往這邊走來／離開了」等靠近通知
    if let Some(player) = game_world
        .npc_manager
        .get_npc(&game_world.current_controlled_id)
        .cloned()
    {
        let id = game_world.current_controlled_id.clone();
        let notifications =
            game_world
                .npc_manager
                .update_proximity(&id, player.x, player.y, &player.map, false);
        for (_npc_id, message, _should_greet) in notifications {
            core_output::trigger_output(OutputZone::Main, &message);
            changed = true;
        }
    }

    changed
}

fn trigger_state_callback(game_world: &GameWorld) {
    let callback = match STATE_CALLBACK.lock() {
        Ok(cb) => *cb,
        Err(_) => None,
    };

    if let Some(callback) = callback {
        let current_player = game_world.npc_manager.get_npc(&game_world.current_controlled_id);
        let state_json = serde_json::json!({
            "world": game_world.metadata.name,
            "current_map": game_world.current_map_name,
            "time": game_world.format_time(),
            "player": current_player.map(|player| serde_json::json!({
                "id": game_world.current_controlled_id,
                "name": player.name,
                "x": player.x,
                "y": player.y,
                "hp": player.hp,
                "max_hp": player.max_hp
            }))
        });

        if let Ok(state_c) = CString::new(state_json.to_string()) {
            callback(state_c.as_ptr());
        }
    }
}

fn trigger_event_callback(event_type: &str, event_data: &str) {
    let callback = match EVENT_CALLBACK.lock() {
        Ok(cb) => *cb,
        Err(_) => None,
    };

    if let Some(callback) = callback {
        if let (Ok(type_c), Ok(data_c)) = (CString::new(event_type), CString::new(event_data)) {
            callback(type_c.as_ptr(), data_c.as_ptr());
        }
    }
}

fn check_and_execute_events(game_world: &mut GameWorld) -> bool {
    game_world.update_time();

    let current_time = (
        game_world.time.day,
        game_world.time.hour,
        game_world.time.minute,
    );

    if current_time == game_world.event_scheduler.last_check_time {
        return false;
    }

    game_world.event_scheduler.last_check_time = current_time;

    let events: Vec<crate::event::GameEvent> = game_world
        .event_manager
        .list_events()
        .into_iter()
        .cloned()
        .collect();

    let Some(player) = game_world.npc_manager.get_npc(&game_world.current_controlled_id).cloned() else {
        core_output::trigger_output(OutputZone::Status, "找不到當前控制的角色，無法檢查事件");
        return false;
    };

    let mut triggered_event_ids = Vec::new();
    for event in events {
        if let Some(runtime_state) = game_world.event_manager.get_runtime_state(&event.id) {
            if !event.can_trigger(runtime_state) {
                continue;
            }
        }

        if game_world.event_scheduler.check_trigger(&event, game_world)
            && game_world
                .event_scheduler
                .check_conditions(&event, game_world, &player)
        {
            triggered_event_ids.push(event.id);
        }
    }

    let did_trigger = !triggered_event_ids.is_empty();
    for event_id in triggered_event_ids {
        game_world.event_manager.trigger_event(&event_id);
        if let Some(event) = game_world.event_manager.get_event(&event_id).cloned() {
            trigger_event_callback("triggered", &event.id);
            core_output::trigger_output(OutputZone::Log, &format!("🎭 事件: {}", event.name));

            let mut output = core_output::CoreOutputManager::new();
            if let Err(e) = crate::event_executor::EventExecutor::execute_event(
                &event,
                game_world,
                &mut output,
            ) {
                core_output::trigger_output(OutputZone::Log, &format!("⚠️  事件執行錯誤: {e}"));
            }
        }
    }

    did_trigger
}

fn save_game_world(game_world: &GameWorld) -> Result<(), Box<dyn std::error::Error>> {
    game_world.save_metadata()?;
    game_world.save_time()?;
    game_world.save_item_counter()?;

    let person_dir = format!("{}/persons", game_world.world_dir);
    game_world.npc_manager.save_all(&person_dir)?;

    for map in game_world.maps.values() {
        game_world.save_map(map)?;
    }

    Ok(())
}

/// 初始化遊戲世界（無 UI 模式）
/// 返回 0=成功, -1=失敗
#[no_mangle]
pub extern "C" fn ratamud_init_game() -> c_int {
    use crate::core_output::OutputZone;
    use crate::event_loader;
    
    // 初始化 Person 描述資料
    crate::person::init_person_descriptions();
    
    // 創建遊戲世界
    let mut game_world = GameWorld::new();
    
    // 載入世界元數據和時間
    let _ = game_world.load_metadata();
    let _ = game_world.load_time();
    let _ = game_world.load_item_counter();
    
    // 輸出當前時間
    core_output::trigger_output(OutputZone::Status, &game_world.format_time());
    
    // 載入地圖
    match game_world.initialize_maps() {
        Ok((map_count, logs)) => {
            for log in logs {
                core_output::trigger_output(OutputZone::Log, &log);
            }
            core_output::trigger_output(OutputZone::Log, &format!("已加載 {} 個地圖", map_count));
        }
        Err(e) => {
            core_output::trigger_output(OutputZone::Log, &format!("⚠️  載入地圖失敗: {}", e));
        }
    }
    
    // 初始化 NPC Manager
    let person_dir = format!("{}/persons", game_world.world_dir);
    let me = match game_world.npc_manager.initialize(&person_dir) {
        Ok((count, me)) => {
            core_output::trigger_output(OutputZone::Log, &format!("已載入 {} 個角色", count));
            for npc in game_world.npc_manager.get_all_npcs() {
                core_output::trigger_output(OutputZone::Log, 
                    &format!("  - {} 在位置 ({}, {})", npc.name, npc.x, npc.y));
            }
            me
        }
        Err(e) => {
            core_output::trigger_output(OutputZone::Status, &format!("❌ 初始化角色系統失敗: {}", e));
            return -1;
        }
    };
    
    // 設定 original_player
    game_world.original_player = Some(me.clone());
    
    // 載入任務
    let quest_dir = format!("{}/quests", game_world.world_dir);
    if let Ok(quest_count) = game_world.quest_manager.load_from_directory(&quest_dir) {
        core_output::trigger_output(OutputZone::Log, &format!("已載入 {} 個任務", quest_count));
    }
    
    // 載入事件腳本
    let events_dir = format!("{}/events", game_world.world_dir);
    if let Ok((count, _event_list)) = event_loader::EventLoader::load_from_directory(&mut game_world.event_manager, &events_dir) {
        if count > 0 {
            core_output::trigger_output(OutputZone::Log, &game_world.event_manager.show_total_loaded_events());
        }
    }
    
    // 顯示歡迎訊息
    core_output::trigger_output(OutputZone::Main, &format!("✨ 歡迎來到 {} ✨", game_world.metadata.name));
    core_output::trigger_output(OutputZone::Main, &game_world.metadata.description);
    core_output::trigger_output(OutputZone::Main, "💡 輸入 'help' 查看可用指令");
    
    // 顯示當前位置資訊
    if let Some(map) = game_world.get_current_map() {
        core_output::trigger_output(OutputZone::Main, &format!("📍 當前區域: {}", map.name));
        core_output::trigger_output(OutputZone::Main, &map.description);
    }
    
    // 儲存到全局狀態
    trigger_state_callback(&game_world);
    if let Ok(mut world) = GAME_WORLD.lock() {
        *world = Some(game_world);
        0 // 成功
    } else {
        -1 // 鎖定失敗
    }
}

/// 處理命令（無 UI 模式）
#[no_mangle]
pub extern "C" fn ratamud_input_command(command: *const c_char) -> c_int {
    if command.is_null() {
        return -1;
    }
    
    let c_str = unsafe { CStr::from_ptr(command) };
    let cmd = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    // 從全局狀態獲取遊戲世界
    let mut world_guard = match GAME_WORLD.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    
    let game_world = match world_guard.as_mut() {
        Some(world) => world,
        None => {
            use crate::core_output::OutputZone;
            core_output::trigger_output(OutputZone::Status, "遊戲尚未初始化，請先調用 ratamud_init_game()");
            return -1;
        }
    };
    
    // 執行命令
    let should_continue = game_world.execute_command(cmd);

    check_and_execute_events(game_world);
    trigger_state_callback(game_world);

    if let Err(e) = save_game_world(game_world) {
        core_output::trigger_output(OutputZone::Status, &format!("存檔失敗: {e}"));
    }

    // 指令可能改變了世界（移動、撿物…），自動刷新 host 目前開啟的面板
    let panel_push = active_panel_content(game_world);
    drop(world_guard);
    if let Some((panel, content)) = panel_push {
        trigger_panel(&panel, &content);
    }

    if should_continue {
        1 // 繼續
    } else {
        0 // 退出
    }
}

#[no_mangle]
pub extern "C" fn ratamud_tick() -> c_int {
    let mut world_guard = match GAME_WORLD.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };

    let game_world = match world_guard.as_mut() {
        Some(world) => world,
        None => return -1,
    };

    let did_trigger_event = check_and_execute_events(game_world);

    // 驅動 NPC AI（每 2 個 tick 一次，讓 NPC 會走動/反應而不會太吵）
    let tick = TICK_COUNT.fetch_add(1, Ordering::Relaxed);
    let npc_changed = if tick % 2 == 0 {
        drive_npc_ai(game_world)
    } else {
        false
    };

    trigger_state_callback(game_world);

    let world_changed = did_trigger_event || npc_changed;

    // 世界有變動時，渲染 host 目前開啟的面板（持鎖時渲染，釋放後再推回）
    let panel_push = if world_changed {
        active_panel_content(game_world)
    } else {
        None
    };

    // NPC 走動不寫檔（避免每秒大量磁碟 I/O）；只有事件觸發才整體存檔
    let save_result = if did_trigger_event {
        save_game_world(game_world)
    } else {
        game_world.save_time()
    };

    drop(world_guard);

    if let Some((panel, content)) = panel_push {
        trigger_panel(&panel, &content);
    }

    if let Err(e) = save_result {
        core_output::trigger_output(OutputZone::Status, &format!("存檔失敗: {e}"));
        return -1;
    }

    0
}

// ============= 面板請求（map / minimap / inventory / status / trade）=============

/// 請求大地圖；內容透過 panel callback 以 "MAP" 推回
#[no_mangle]
pub extern "C" fn ratamud_request_map() -> c_int {
    render_and_push("MAP", crate::panel_render::render_map)
}

/// 請求小地圖；以 "MINIMAP" 推回
#[no_mangle]
pub extern "C" fn ratamud_request_minimap() -> c_int {
    render_and_push("MINIMAP", crate::panel_render::render_minimap)
}

/// 請求背包；以 "INVENTORY" 推回
#[no_mangle]
pub extern "C" fn ratamud_request_inventory() -> c_int {
    render_and_push("INVENTORY", crate::panel_render::render_inventory)
}

/// 請求角色狀態；以 "STATUS" 推回
#[no_mangle]
pub extern "C" fn ratamud_request_status() -> c_int {
    render_and_push("STATUS", crate::panel_render::render_status)
}

/// 請求交易面板；npc 可為空字串（自動偵測當前格的商人）。內容以 "TRADE" 推回。
#[no_mangle]
pub extern "C" fn ratamud_request_trade(npc: *const c_char) -> c_int {
    let npc_name = cstr_to_string(npc);
    render_and_push("TRADE", move |world| crate::panel_render::render_trade(world, &npc_name))
}

/// 玩家向 NPC 購買物品。成功後刷新 TRADE 與 INVENTORY 面板並存檔。
/// 回傳 1=成功, 0=交易失敗, -1=參數/世界錯誤
#[no_mangle]
pub extern "C" fn ratamud_trade_buy(npc: *const c_char, item: *const c_char, qty: c_int) -> c_int {
    execute_trade(npc, item, qty, true)
}

/// 玩家向 NPC 出售物品。成功後刷新 TRADE 與 INVENTORY 面板並存檔。
#[no_mangle]
pub extern "C" fn ratamud_trade_sell(npc: *const c_char, item: *const c_char, qty: c_int) -> c_int {
    execute_trade(npc, item, qty, false)
}

/// 把 C 字串安全轉成 Rust String（null/非 UTF-8 → 空字串）
fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// 交易執行共用邏輯（買/賣）
fn execute_trade(npc: *const c_char, item: *const c_char, qty: c_int, is_buy: bool) -> c_int {
    use crate::core_output::OutputZone;
    use crate::trade::{TradeResult, TradeSystem};

    let item_name = cstr_to_string(item);
    if item_name.is_empty() || qty <= 0 {
        return -1;
    }
    let quantity = qty as u32;
    let npc_arg = cstr_to_string(npc);

    let mut world_guard = match GAME_WORLD.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    let game_world = match world_guard.as_mut() {
        Some(world) => world,
        None => return -1,
    };

    // 解析交易對象（空字串 → 自動偵測當前格商人）
    let Some(npc_name) = crate::panel_render::resolve_trade_npc(game_world, &npc_arg) else {
        core_output::trigger_output(OutputZone::Status, "這裡沒有可以交易的對象");
        return -1;
    };

    let result = if is_buy {
        let price = TradeSystem::calculate_buy_price(&item_name, quantity);
        TradeSystem::buy_from_npc(game_world, &npc_name, &item_name, quantity, price)
    } else {
        let price = TradeSystem::calculate_sell_price(&item_name, quantity);
        TradeSystem::sell_to_npc(game_world, &npc_name, &item_name, quantity, price)
    };

    let success = match result {
        TradeResult::Success(msg) => {
            core_output::trigger_output(OutputZone::Main, &msg);
            true
        }
        TradeResult::Failed(reason) => {
            core_output::trigger_output(OutputZone::Status, &reason);
            false
        }
    };

    // 重新渲染面板（在持鎖狀態下產生字串）
    let trade_content = crate::panel_render::render_trade(game_world, &npc_name);
    let inv_content = crate::panel_render::render_inventory(game_world);

    if let Err(e) = save_game_world(game_world) {
        core_output::trigger_output(OutputZone::Status, &format!("存檔失敗: {e}"));
    }
    drop(world_guard);

    trigger_panel("TRADE", &trade_content);
    trigger_panel("INVENTORY", &inv_content);

    if success { 1 } else { 0 }
}

/// 測試輸出回調功能（無 UI 模式）
#[no_mangle]
pub extern "C" fn ratamud_test_output_callback() {
    use crate::core_output::CoreOutputManager;
    
    let mut output = CoreOutputManager::new();
    
    // 測試各種類型的輸出
    output.add_message("歡迎來到 RataMUD！".to_string());
    output.add_message("你站在一個廣場中央。".to_string());
    output.add_log("遊戲初始化完成".to_string());
    output.add_log("載入地圖: town_square".to_string());
    output.set_status("遊戲時間: Day 1 09:00".to_string());
    output.set_side_content("NPC: 商人\n等級: 10\n生命: 100/100".to_string());
    output.add_message("一隻野豬向你衝來！".to_string());
}

#[cfg(not(feature = "terminal-ui"))]
#[no_mangle]
pub extern "C" fn ratamud_start_game() -> c_int {
    ratamud_init_game()
}

// Terminal UI mode functions (only available with terminal-ui feature)
#[cfg(feature = "terminal-ui")]
pub mod terminal_ui_ffi {
    use super::*;

    /// 啟動遊戲主程式（終端 UI 模式）
    #[no_mangle]
    pub extern "C" fn ratamud_start_game() -> c_int {
        use std::io;
        use crossterm::{
            self,
            execute,
            terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        };
        use ratatui::{
            backend::CrosstermBackend,
            Terminal,
        };
        
        use crate::input::InputHandler;
        use crate::output::OutputManager;
        use crate::world::GameWorld;
        use crate::person;
        use crate::settings::GameSettings;
        use crate::app;
        
        // 初始化 Person 描述資料
        person::init_person_descriptions();
        
        // 初始化 InputHandler, OutputManager, GameWorld, Person
        let mut output_manager = OutputManager::new();
            
        // 載入遊戲設定
        let game_settings = GameSettings::load();
        output_manager.log(format!("載入設定: show_minimap = {}, show_log = {}", 
            game_settings.show_minimap, game_settings.show_log));
        
        if game_settings.show_minimap {
            output_manager.show_minimap();
            output_manager.log("小地圖已開啟".to_string());
        }
        
        if !game_settings.show_log {
            output_manager.hide_log();
            output_manager.log("日誌視窗已關閉".to_string());
        } else {
            output_manager.log("日誌視窗已開啟".to_string());
        }

        // 初始化遊戲世界
        let mut game_world = GameWorld::new();
        
        // 嘗試加載世界元數據和時間
        let _ = game_world.load_metadata();
        let _ = game_world.load_time();
        
        // 設置初始時間顯示
        output_manager.set_current_time(game_world.format_time());

        // 載入地圖   
        match game_world.initialize_maps() {
            Ok((map_count, logs)) => {
                for log in logs {
                    output_manager.log(log);
                }
                output_manager.log(format!("已加載 {map_count} 個地圖"));
            }
            Err(e) => {
                output_manager.log(format!("⚠️  載入地圖失敗: {e}"));
            }
        }
        
        // 顯示當前時間
        output_manager.log(format!("⏰ {}", game_world.format_time()));
        
        // 初始化 NPC Manager（載入所有角色並確保 me 存在）
        let person_dir = format!("{}/persons", game_world.world_dir);
        match game_world.npc_manager.initialize(&person_dir) {
            Ok((count, me)) => {
                output_manager.log(format!("已載入 {count} 個角色"));
                for npc in game_world.npc_manager.get_all_npcs() {
                    output_manager.log(format!("  - {} 在位置 ({}, {})", npc.name, npc.x, npc.y));
                }
                // 設定 game_world.original_player
                game_world.original_player = Some(me);
            }
            Err(e) => {
                eprintln!("初始化角色系統失敗: {e}");
                return -1;
            }
        }
        
        // 載入任務
        load_quest_internal(&mut game_world, &mut output_manager);

        // 載入事件腳本
        load_event_internal(&mut game_world, &mut output_manager);

        // 顯示歡迎訊息
        show_welcome_message_internal(&mut output_manager, &game_world);
        show_current_map_info_internal(&mut output_manager, &game_world);

        // 如果小地圖已開啟，初始化其內容
        if output_manager.is_minimap_open() {
            app::update_minimap_display(&mut output_manager, &game_world);
        }

        // 建立crossterm輸入事件執行緒
        let rx = create_key_event_thread_internal();

        // 初始化 InputHandler
        let input_handler = InputHandler::new();
        
        // 初始化終端原始模式和備用螢幕
        if enable_raw_mode().is_err() {
            return -1;
        }
        let mut stdout = io::stdout();
        if execute!(stdout, EnterAlternateScreen).is_err() {
            let _ = disable_raw_mode();
            return -1;
        }
        // 初始化 Terminal UI
        let backend = CrosstermBackend::new(stdout);
        let terminal = match Terminal::new(backend) {
            Ok(t) => t,
            Err(_) => {
                let _ = disable_raw_mode();
                let _ = execute!(io::stdout(), LeaveAlternateScreen);
                return -1;
            }
        };
        // 運行主迴圈 ==>
        if app::run_main_loop(terminal, input_handler, output_manager, game_world, rx).is_err() {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            return -1;
        }
        // <== 運行主迴圈結束(exit/quit)
        // 清理終端設定並返回到常規模式
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);

        0
    }

    // 建立crossterm輸入事件執行緒
    fn create_key_event_thread_internal() -> std::sync::mpsc::Receiver<crossterm::event::KeyEvent> {
        use std::sync::mpsc;
        use std::thread;
        use crossterm::event as evt;
        
        let (tx, rx) = mpsc::channel::<crossterm::event::KeyEvent>();
        thread::spawn(move || {
            loop {
                // `read()` is a blocking call, waiting for an event
                if let Ok(crossterm::event::Event::Key(key_event)) = evt::read() {
                    // Send the key event to the main thread.
                    // If the receiver is dropped, the thread will exit gracefully.
                    if tx.send(key_event).is_err() {
                        break;
                    }
                }
            }
        });
        rx
    }

    /// 顯示世界歡迎訊息
    fn show_welcome_message_internal(output_manager: &mut crate::output::OutputManager, game_world: &crate::world::GameWorld) {
        output_manager.print(format!("✨ 歡迎來到 {} ✨", game_world.metadata.name));
        output_manager.print(game_world.metadata.description.clone());
        output_manager.print("".to_string());
        output_manager.print("💡 輸入 'help' 查看可用指令".to_string());
        output_manager.print("".to_string());
    }

    /// 顯示當前地圖資訊
    fn show_current_map_info_internal(output_manager: &mut crate::output::OutputManager, game_world: &crate::world::GameWorld) {
        if let Some(current_map) = game_world.get_current_map() {
            output_manager.print(format!("📍 當前區域: {}", current_map.name));
            output_manager.print(current_map.description.clone());
        }
    }

    /// 載入事件腳本
    fn load_event_internal(game_world: &mut crate::world::GameWorld, output_manager: &mut crate::output::OutputManager) {
        use crate::event_loader;
        let events_dir = format!("{}/events", game_world.world_dir);
        match event_loader::EventLoader::load_from_directory(&mut game_world.event_manager, &events_dir) {
            Ok((count, event_list)) => {
                if count > 0 {
                    output_manager.log(game_world.event_manager.show_total_loaded_events());
                    for event_name in event_list {
                        output_manager.log(format!("  📌 {event_name}"));
                    }
                }
            }
            Err(e) => {
                output_manager.log(format!("⚠️  載入事件失敗: {e}"));
            }
        } 
    }

    /// 載入任務
    fn load_quest_internal(game_world: &mut crate::world::GameWorld, output_manager: &mut crate::output::OutputManager) {
        output_manager.log("開始載入任務...".to_string());
        let quest_dir = format!("{}/quests", game_world.world_dir);
        match game_world.quest_manager.load_from_directory(&quest_dir) {
            Ok(count) => {
                output_manager.log(format!("從文件載入了 {count} 個任務"));
            }
            Err(e) => {
                output_manager.log(format!("⚠️  載入任務失敗: {e}"));
            }
        }
    }
}

