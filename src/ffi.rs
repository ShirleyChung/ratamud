#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

use crate::core_output::{self, OutputZone};
use crate::world::GameWorld;

/// 全局遊戲世界實例（FFI 和其他非 UI 模式共用）
static GAME_WORLD: Lazy<Mutex<Option<GameWorld>>> = Lazy::new(|| Mutex::new(None));

/// host 目前開啟的面板（由 `ratamud_set_active_panel` 設定）。
/// 當世界因 NPC 行動或事件而改變時，引擎會自動重新渲染並推回這個面板。
static ACTIVE_PANEL: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

/// 面板的視窗大小（由 `ratamud_set_map_view_size` 設定，0 = 用預設值）。
/// 自動刷新地圖時會沿用 host 指定的大小。
static MAP_VIEW_SIZE: Lazy<Mutex<(usize, usize)>> = Lazy::new(|| Mutex::new((0, 0)));

/// tick 計數器（用來節流 NPC AI，避免每秒都動）
static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

/// 上一次推進戰鬥回合的時間（對應終端版主迴圈的每 3 秒一回合）
static LAST_COMBAT_ROUND: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));

/// 上一次把地圖寫回磁碟的時間
static LAST_MAP_SAVE: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));

/// 地圖節流存檔間隔。一張 100x100 的地圖 JSON 約 2.4MB，五張接近 10MB；
/// 以前每個指令都全部重寫一次，這正是 iOS 上 event loop 會卡死的主因。
const MAP_SAVE_INTERVAL: Duration = Duration::from_secs(30);

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
    let (view_w, view_h) = map_view_size();
    match panel {
        "MAP" => Some(pr::render_map_view(world, view_w, view_h)),
        "MAP_JSON" => Some(pr::render_map_json(world, view_w, view_h)),
        "MINIMAP" => Some(pr::render_minimap(world)),
        "INVENTORY" => Some(pr::render_inventory(world)),
        "STATUS" => Some(pr::render_status(world)),
        // 交易面板會自行偵測當前格的商人
        "TRADE" => Some(pr::render_trade(world, "")),
        _ => None,
    }
}

/// host 指定的地圖視窗大小（0 表示用引擎預設值）
fn map_view_size() -> (usize, usize) {
    MAP_VIEW_SIZE.lock().map(|size| *size).unwrap_or((0, 0))
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
    use crate::message::Message;
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

        for msg in messages {
            changed = true;

            match &msg {
                // 戰鬥動作必須真的結算傷害與冷卻，否則 NPC 的攻擊等於沒發生
                // （終端版在主迴圈裡處理，FFI 以前只把文字印出來就算了）
                Message::CombatAction { attacker_id, skill_name, target_id, damage, .. } => {
                    if let Some(target) = game_world.npc_manager.get_npc_mut(target_id) {
                        target.check_hp(-damage);
                        let (hp, max_hp) = (target.hp, target.max_hp);
                        core_output::trigger_output(
                            OutputZone::Main,
                            &format!("{} 剩餘 HP: {}/{}", msg.to_display_text(), hp, max_hp),
                        );
                    }
                    if let Some(attacker) = game_world.npc_manager.get_npc_mut(attacker_id) {
                        let _ = attacker.practice_skill(skill_name, true);
                    }
                    crate::combat::check_combat_end(game_world);
                }
                // 移動之類的訊息屬於日誌，不洗主畫面；NPC 對話等才輸出
                _ if msg.is_log() => {}
                _ => core_output::trigger_output(OutputZone::Main, &msg.to_display_text()),
            }
        }
    }

    // 依玩家位置產生「往這邊走來／離開了」與見面語
    if crate::command_executor::report_proximity(game_world, false) {
        changed = true;
    }

    changed
}

fn trigger_state_callback(game_world: &GameWorld) {
    let callback = match STATE_CALLBACK.lock() {
        Ok(cb) => *cb,
        Err(_) => None,
    };

    if let Some(callback) = callback {
        use crate::world::{CombatState, InteractionState};

        let current_player = game_world.npc_manager.get_npc(&game_world.current_controlled_id);

        // host 需要這些才畫得出狀態列，也才知道現在是不是在戰鬥/交易中
        let combat = match &game_world.combat_state {
            CombatState::None => serde_json::json!({ "in_combat": false }),
            CombatState::InCombat { participants, round } => serde_json::json!({
                "in_combat": true,
                "round": round,
                "participants": participants,
            }),
        };

        let interaction = match &game_world.interaction_state {
            InteractionState::None => serde_json::Value::Null,
            InteractionState::Trading { npc_name } => {
                serde_json::json!({ "kind": "trading", "npc": npc_name })
            }
            InteractionState::Buying { npc_name } => {
                serde_json::json!({ "kind": "buying", "npc": npc_name })
            }
            InteractionState::Selling { npc_name } => {
                serde_json::json!({ "kind": "selling", "npc": npc_name })
            }
        };

        let state_json = serde_json::json!({
            "world": game_world.metadata.name,
            "current_map": game_world.current_map_name,
            "time": game_world.format_time(),
            "day": game_world.time.day,
            "hour": game_world.time.hour,
            "minute": game_world.time.minute,
            "player": current_player.map(|player| serde_json::json!({
                "id": game_world.current_controlled_id,
                "name": player.name,
                "x": player.x,
                "y": player.y,
                "map": player.map,
                "hp": player.hp,
                "max_hp": player.max_hp,
                "mp": player.mp,
                "max_mp": player.max_mp,
                "gold": player.items.get("金幣").copied().unwrap_or(0),
                "status": player.status,
                "is_sleeping": player.is_sleeping,
                "combat_exp": player.combat_exp,
            })),
            "combat": combat,
            "interaction": interaction,
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

/// 輕量存檔：世界中繼資料、時間、物品流水號、所有角色。
///
/// 這些檔案加起來只有幾十 KB，隨時寫都不會卡。地圖**不在**這裡，因為它動輒
/// 10MB；地圖改由 [`save_maps_throttled`] 或 [`ratamud_save`] 處理。
fn save_game_light(game_world: &GameWorld) -> Result<(), Box<dyn std::error::Error>> {
    game_world.save_metadata()?;
    game_world.save_time()?;
    game_world.save_item_counter()?;

    let person_dir = format!("{}/persons", game_world.world_dir);
    game_world.npc_manager.save_all(&person_dir)?;

    Ok(())
}

/// 節流寫入有變動的地圖：有 dirty 地圖、且距離上次寫入超過
/// [`MAP_SAVE_INTERVAL`] 才真的寫。
fn save_maps_throttled(game_world: &mut GameWorld) -> Result<(), Box<dyn std::error::Error>> {
    if !game_world.has_dirty_maps() {
        return Ok(());
    }

    let now = Instant::now();
    {
        let Ok(mut last) = LAST_MAP_SAVE.lock() else { return Ok(()) };
        match *last {
            Some(prev) if now.duration_since(prev) < MAP_SAVE_INTERVAL => return Ok(()),
            _ => *last = Some(now),
        }
    }

    game_world.save_dirty_maps()?;
    Ok(())
}

/// 完整存檔：輕量存檔 + 所有有變動的地圖，不受節流限制。
/// host 應該在進背景／關閉前呼叫 `ratamud_save()`。
fn save_game_full(game_world: &mut GameWorld) -> Result<(), Box<dyn std::error::Error>> {
    save_game_light(game_world)?;
    game_world.save_dirty_maps()?;
    if let Ok(mut last) = LAST_MAP_SAVE.lock() {
        *last = Some(Instant::now());
    }
    Ok(())
}

// ============= 資料目錄（iOS 沙盒必要）=============

/// 設定資料根目錄。所有 `worlds/...` 都會相對於它。
///
/// iOS 的行程工作目錄是唯讀的，也不是 app bundle 所在位置，所以引擎預設的
/// 相對路徑 `worlds/beginWorld` 一定會失敗——地圖載不進來、狀態也存不起來。
/// host 必須在 `ratamud_init_game()` **之前**把可寫目錄（通常是
/// `FileManager.default.urls(for: .documentDirectory, ...)`）傳進來。
///
/// 回傳 0=成功, -1=路徑無效或無法建立。
#[no_mangle]
pub extern "C" fn ratamud_set_data_dir(path: *const c_char) -> c_int {
    let dir = cstr_to_string(path);
    if dir.is_empty() {
        return -1;
    }

    if let Err(e) = std::fs::create_dir_all(&dir) {
        core_output::trigger_output(
            OutputZone::Status,
            &format!("無法建立資料目錄 {dir}: {e}"),
        );
        return -1;
    }

    crate::paths::set_data_root(&dir);
    core_output::trigger_output(OutputZone::Log, &format!("資料目錄: {dir}"));
    0
}

/// 取得目前的資料根目錄（唯讀，供除錯用）。回傳的字串由呼叫端以
/// `ratamud_free_string()` 釋放；失敗時回傳 NULL。
#[no_mangle]
pub extern "C" fn ratamud_get_data_dir() -> *mut c_char {
    let root = crate::paths::data_root().to_string_lossy().into_owned();
    match CString::new(root) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// 釋放本模組回傳的字串。
///
/// # Safety
/// `ptr` 必須是本模組（例如 `ratamud_get_data_dir`）回傳、且尚未釋放的指標。
#[no_mangle]
pub unsafe extern "C" fn ratamud_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(unsafe { CString::from_raw(ptr) });
    }
}

/// 把 app bundle 內的唯讀世界資料安裝到可寫的資料目錄。
///
/// `bundle_dir` 應該是包含 `worlds/` 的那一層（iOS 上通常是
/// `Bundle.main.resourcePath`）。**不會覆蓋**已存在的檔案，所以可以每次啟動
/// 都呼叫，玩家的存檔不會被初始資料蓋掉。
///
/// 回傳實際複製的檔案數量，失敗回傳 -1。
#[no_mangle]
pub extern "C" fn ratamud_seed_data_dir(bundle_dir: *const c_char) -> c_int {
    let bundle = cstr_to_string(bundle_dir);
    if bundle.is_empty() {
        return -1;
    }

    let src = Path::new(&bundle).join("worlds");
    if !src.is_dir() {
        core_output::trigger_output(
            OutputZone::Status,
            &format!("找不到 bundle 內的世界資料: {}", src.display()),
        );
        return -1;
    }

    let dst = crate::paths::data_root().join("worlds");
    match crate::paths::copy_tree_if_missing(&src, &dst) {
        Ok(count) => {
            if count > 0 {
                core_output::trigger_output(
                    OutputZone::Log,
                    &format!("已安裝 {count} 個世界資料檔到 {}", dst.display()),
                );
            }
            count as c_int
        }
        Err(e) => {
            core_output::trigger_output(OutputZone::Status, &format!("安裝世界資料失敗: {e}"));
            -1
        }
    }
}

/// 一次做完 iOS 需要的三件事：設定可寫目錄 → 從 bundle 安裝資料 → 初始化世界。
///
/// `bundle_dir` 可傳 NULL 或空字串表示不需要安裝（資料已經在可寫目錄裡）。
/// 回傳 0=成功, -1=失敗。
#[no_mangle]
pub extern "C" fn ratamud_init_game_with_dir(
    data_dir: *const c_char,
    bundle_dir: *const c_char,
) -> c_int {
    if ratamud_set_data_dir(data_dir) != 0 {
        return -1;
    }

    if !bundle_dir.is_null() && !cstr_to_string(bundle_dir).is_empty() {
        // 安裝失敗不一定是致命的（資料可能已經在了），讓 init 自己去判斷
        let _ = ratamud_seed_data_dir(bundle_dir);
    }

    ratamud_init_game()
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

    // 退出時做完整存檔；其餘只寫小檔案，地圖走節流。
    // （以前每個指令都把五張 100x100 的地圖重寫一次 ≈ 10MB，
    //   在 iOS 上等於每個動作都卡住好幾秒。）
    let save_result = if should_continue {
        save_game_light(game_world).and_then(|_| save_maps_throttled(game_world))
    } else {
        save_game_full(game_world)
    };
    if let Err(e) = save_result {
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

/// 立刻把目前狀態完整寫回磁碟（含有變動的地圖）。
///
/// iOS host 應該在 `scenePhase` 變成 `.background`／`.inactive` 時呼叫，
/// 因為 app 隨時可能被系統終止。回傳 0=成功, -1=失敗。
#[no_mangle]
pub extern "C" fn ratamud_save() -> c_int {
    let Ok(mut world_guard) = GAME_WORLD.lock() else { return -1 };
    let Some(game_world) = world_guard.as_mut() else { return -1 };

    match save_game_full(game_world) {
        Ok(()) => 0,
        Err(e) => {
            core_output::trigger_output(OutputZone::Status, &format!("存檔失敗: {e}"));
            -1
        }
    }
}

/// 推進遊戲世界一次。
///
/// 這是 iOS 版的「主迴圈」，對應終端版 `app::run_main_loop` 每一幀做的事：
/// 推進時間、更新角色狀態、檢查世界事件、驅動 NPC AI、推進戰鬥回合，最後刷新
/// host 開啟的面板。host 大約每秒呼叫一次即可。
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

    // --- 1. 時間與世界事件（check_and_execute_events 內部會先 update_time）---
    let did_trigger_event = check_and_execute_events(game_world);

    // --- 2. 更新玩家自己的時間狀態（飢餓扣 HP、睡眠回 MP…）---
    // 終端版每一幀都做這件事，FFI 以前完全漏掉，所以 iOS 上玩家狀態永遠不變。
    {
        use crate::time_updatable::TimeUpdatable;
        let time_info = game_world.get_time_info();
        let id = game_world.current_controlled_id.clone();
        if let Some(me) = game_world.npc_manager.get_npc_mut(&id) {
            me.on_time_update(&time_info);
        }
    }

    // --- 3. 驅動 NPC AI（每 2 個 tick 一次，讓 NPC 會走動/反應而不會太吵）---
    let tick = TICK_COUNT.fetch_add(1, Ordering::Relaxed);
    let npc_changed = if tick.is_multiple_of(2) {
        drive_npc_ai(game_world)
    } else {
        false
    };

    // --- 4. 自動戰鬥回合（對應終端版的每 3 秒一回合）---
    let combat_changed = drive_combat_round(game_world);

    trigger_state_callback(game_world);

    let world_changed = did_trigger_event || npc_changed || combat_changed;

    // 世界有變動時，渲染 host 目前開啟的面板（持鎖時渲染，釋放後再推回）
    let panel_push = if world_changed {
        active_panel_content(game_world)
    } else {
        None
    };

    // 世界有變動才寫小檔案；地圖永遠走節流，避免每秒大量磁碟 I/O
    let save_result = if world_changed {
        save_game_light(game_world).and_then(|_| save_maps_throttled(game_world))
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

/// 在戰鬥中時，每隔 `COMBAT_ROUND_INTERVAL_SECS` 推進一個回合。
/// 回傳 true 表示這次 tick 真的推進了一回合。
fn drive_combat_round(game_world: &mut GameWorld) -> bool {
    use crate::world::CombatState;

    if matches!(game_world.combat_state, CombatState::None) {
        // 不在戰鬥中就重置計時器，下次開打才會從頭計時
        if let Ok(mut last) = LAST_COMBAT_ROUND.lock() {
            *last = None;
        }
        return false;
    }

    let interval = Duration::from_secs(crate::combat::COMBAT_ROUND_INTERVAL_SECS);
    let now = Instant::now();
    {
        let Ok(mut last) = LAST_COMBAT_ROUND.lock() else { return false };
        match *last {
            Some(prev) if now.duration_since(prev) < interval => return false,
            // 開戰後的第一回合已經由玩家那一擊帶過，這裡只起算計時器，
            // 下一個自動回合要等滿一個間隔才來
            None => {
                *last = Some(now);
                return false;
            }
            _ => *last = Some(now),
        }
    }

    crate::combat::execute_combat_round(game_world)
}

// ============= 面板請求（map / minimap / inventory / status / trade）=============

/// 設定大地圖視窗大小（以玩家為中心的格數）。傳 0 表示使用引擎預設值。
///
/// 整張地圖是 100x100，一次吐一萬個字元給 host 既慢又沒辦法顯示；改由 host
/// 依畫面寬高指定要看多大一塊。
#[no_mangle]
pub extern "C" fn ratamud_set_map_view_size(width: c_int, height: c_int) {
    let width = if width > 0 { width as usize } else { 0 };
    let height = if height > 0 { height as usize } else { 0 };
    if let Ok(mut size) = MAP_VIEW_SIZE.lock() {
        *size = (width, height);
    }
}

/// 請求大地圖（以玩家為中心的視窗）；內容透過 panel callback 以 "MAP" 推回
#[no_mangle]
pub extern "C" fn ratamud_request_map() -> c_int {
    let (w, h) = map_view_size();
    render_and_push("MAP", move |world| crate::panel_render::render_map_view(world, w, h))
}

/// 請求結構化的地圖資料（JSON）；以 "MAP_JSON" 推回。
/// 想自己畫格子而不是顯示 ASCII 的 host 用這個。
#[no_mangle]
pub extern "C" fn ratamud_request_map_json() -> c_int {
    let (w, h) = map_view_size();
    render_and_push("MAP_JSON", move |world| crate::panel_render::render_map_json(world, w, h))
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

/// 交易執行共用邏輯（買/賣）。
/// 規則本身在 `command_interact::execute_trade`（與終端版共用），這裡只負責
/// FFI 的字串轉換、鎖管理與面板推送。
fn execute_trade(npc: *const c_char, item: *const c_char, qty: c_int, is_buy: bool) -> c_int {
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

    let success =
        crate::command_interact::execute_trade(game_world, &npc_name, &item_name, quantity, is_buy);

    // 重新渲染面板（在持鎖狀態下產生字串）
    let trade_content = crate::panel_render::render_trade(game_world, &npc_name);
    let inv_content = crate::panel_render::render_inventory(game_world);
    trigger_state_callback(game_world);
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

