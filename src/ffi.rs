#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

use crate::person::Person;
use crate::world::GameWorld;
use crate::game_engine::GameEngine;

/// 創建遊戲引擎（無頭模式）
#[no_mangle]
pub extern "C" fn ratamud_create_engine(player_name: *const c_char) -> *mut GameEngine {
    if player_name.is_null() {
        return ptr::null_mut();
    }
    
    let c_name = unsafe { CStr::from_ptr(player_name) };
    match c_name.to_str() {
        Ok(name) => {
            let engine = GameEngine::new(name, "冒險者");
            Box::into_raw(Box::new(engine))
        }
        _ => ptr::null_mut(),
    }
}

/// 釋放遊戲引擎
#[no_mangle]
pub extern "C" fn ratamud_free_engine(engine: *mut GameEngine) {
    if !engine.is_null() {
        unsafe {
            let _ = Box::from_raw(engine);
        }
    }
}

/// 處理命令（返回 1 繼續，0 退出，-1 錯誤）
#[no_mangle]
pub extern "C" fn ratamud_engine_process_command(
    engine: *mut GameEngine,
    command: *const c_char
) -> c_int {
    if engine.is_null() || command.is_null() {
        return -1;
    }
    
    let c_str = unsafe { CStr::from_ptr(command) };
    let cmd = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let engine_ref = unsafe { &mut *engine };
    let (should_continue, _result) = engine_ref.process_command(cmd);
    
    if should_continue {
        1
    } else {
        0
    }
}

/// 獲取輸出（返回所有輸出並清空緩衝區）
#[no_mangle]
pub extern "C" fn ratamud_engine_get_output(engine: *mut GameEngine) -> *mut c_char {
    if engine.is_null() {
        return ptr::null_mut();
    }
    
    let engine_ref = unsafe { &mut *engine };
    let output = engine_ref.get_output();
    let combined = output.join("\n");
    
    match CString::new(combined) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// 獲取遊戲狀態（JSON）
#[no_mangle]
pub extern "C" fn ratamud_engine_get_state(engine: *mut GameEngine) -> *mut c_char {
    if engine.is_null() {
        return ptr::null_mut();
    }
    
    let engine_ref = unsafe { &*engine };
    let state_json = engine_ref.get_state_json();
    
    match CString::new(state_json) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// 更新遊戲邏輯
#[no_mangle]
pub extern "C" fn ratamud_engine_update(engine: *mut GameEngine, delta_ms: u32) {
    if engine.is_null() {
        return;
    }
    
    let engine_ref = unsafe { &mut *engine };
    engine_ref.update(delta_ms);
}

// ===== 以下是原有的 Player/World API =====

/// 創建新玩家
#[no_mangle]
pub extern "C" fn ratamud_create_player(name: *const c_char, description: *const c_char) -> *mut Person {
    if name.is_null() || description.is_null() {
        return ptr::null_mut();
    }
    
    let c_name = unsafe { CStr::from_ptr(name) };
    let c_desc = unsafe { CStr::from_ptr(description) };
    
    match (c_name.to_str(), c_desc.to_str()) {
        (Ok(n), Ok(d)) => {
            let player = Person::new(n.to_string(), d.to_string());
            Box::into_raw(Box::new(player))
        }
        _ => ptr::null_mut(),
    }
}

/// 創建遊戲世界
#[no_mangle]
pub extern "C" fn ratamud_create_world(player: *mut Person) -> *mut GameWorld {
    if player.is_null() {
        return ptr::null_mut();
    }
    
    let player_ref = unsafe { &*player };
    let world = GameWorld::new(player_ref.clone());
    Box::into_raw(Box::new(world))
}

/// 釋放玩家
#[no_mangle]
pub extern "C" fn ratamud_free_player(player: *mut Person) {
    if !player.is_null() {
        unsafe {
            let _ = Box::from_raw(player);
        }
    }
}

/// 釋放遊戲世界
#[no_mangle]
pub extern "C" fn ratamud_free_world(world: *mut GameWorld) {
    if !world.is_null() {
        unsafe {
            let _ = Box::from_raw(world);
        }
    }
}

/// 載入地圖
#[no_mangle]
pub extern "C" fn ratamud_load_map(world: *mut GameWorld, map_name: *const c_char) -> c_int {
    if world.is_null() || map_name.is_null() {
        return -1;
    }
    
    let c_str = unsafe { CStr::from_ptr(map_name) };
    let name = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let world_ref = unsafe { &mut *world };
    match world_ref.load_map(name) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// 獲取版本資訊
#[no_mangle]
pub extern "C" fn ratamud_version() -> *const c_char {
    c"RataMUD v0.1.0".as_ptr()
}

/// 獲取玩家名稱
#[no_mangle]
pub extern "C" fn ratamud_get_player_name(player: *const Person) -> *mut c_char {
    if player.is_null() {
        return ptr::null_mut();
    }
    
    let player_ref = unsafe { &*player };
    match CString::new(player_ref.name.clone()) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// 獲取玩家資訊（JSON）- 需要 Person 指針
#[no_mangle]
pub extern "C" fn ratamud_player_get_info(player: *const Person) -> *mut c_char {
    if player.is_null() {
        return ptr::null_mut();
    }
    
    let player_ref = unsafe { &*player };
    let info = serde_json::json!({
        "name": player_ref.name,
        "hp": player_ref.hp,
        "max_hp": player_ref.max_hp,
        "mp": player_ref.mp,
        "max_mp": player_ref.max_mp,
        "position": [player_ref.x, player_ref.y],
        "map": "初始之地",
        "status": player_ref.status,
    });
    
    match CString::new(info.to_string()) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// 獲取玩家 HP
#[no_mangle]
pub extern "C" fn ratamud_get_player_hp(player: *const Person) -> c_int {
    if player.is_null() {
        return -1;
    }
    
    let player_ref = unsafe { &*player };
    player_ref.hp
}

/// 設置玩家 HP
#[no_mangle]
pub extern "C" fn ratamud_set_player_hp(player: *mut Person, hp: c_int) -> c_int {
    if player.is_null() {
        return -1;
    }
    
    let player_ref = unsafe { &mut *player };
    player_ref.hp = hp;
    0
}

/// 獲取玩家位置 - 需要 Person 指針
#[no_mangle]
pub extern "C" fn ratamud_player_get_position(player: *const Person, x: *mut c_int, y: *mut c_int) -> c_int {
    if player.is_null() || x.is_null() || y.is_null() {
        return -1;
    }
    
    let player_ref = unsafe { &*player };
    unsafe {
        *x = player_ref.x as c_int;
        *y = player_ref.y as c_int;
    }
    0
}

/// 設置玩家位置
#[no_mangle]
pub extern "C" fn ratamud_set_player_position(player: *mut Person, x: c_int, y: c_int) -> c_int {
    if player.is_null() {
        return -1;
    }
    
    let player_ref = unsafe { &mut *player };
    player_ref.x = x.max(0) as usize;
    player_ref.y = y.max(0) as usize;
    0
}

/// 獲取當前地圖名稱 - 需要 GameWorld 指針
#[no_mangle]
pub extern "C" fn ratamud_world_get_current_map(world: *const GameWorld) -> *mut c_char {
    if world.is_null() {
        return ptr::null_mut();
    }
    
    let world_ref = unsafe { &*world };
    let map_name = world_ref.current_map_name.clone();
    
    match CString::new(map_name) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// 釋放字串
#[no_mangle]
pub extern "C" fn ratamud_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

// ===== 簡化的全局 API（用於簡單的 C 集成）=====

use std::sync::Mutex;
use once_cell::sync::Lazy;

static GLOBAL_ENGINE: Lazy<Mutex<Option<Box<GameEngine>>>> = Lazy::new(|| Mutex::new(None));

/// 初始化遊戲（使用全局引擎）
#[no_mangle]
pub extern "C" fn ratamud_init() -> c_int {
    let mut engine_guard = match GLOBAL_ENGINE.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    
    if engine_guard.is_some() {
        return -1; // 已經初始化
    }
    
    let engine = GameEngine::new("玩家", "冒險者");
    *engine_guard = Some(Box::new(engine));
    0
}

/// 清理遊戲資源
#[no_mangle]
pub extern "C" fn ratamud_cleanup() {
    let mut engine_guard = match GLOBAL_ENGINE.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    
    *engine_guard = None;
}

/// 處理命令
#[no_mangle]
pub extern "C" fn ratamud_process_command(command: *const c_char) -> c_int {
    if command.is_null() {
        return -1;
    }
    
    let c_str = unsafe { CStr::from_ptr(command) };
    let cmd = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let mut engine_guard = match GLOBAL_ENGINE.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    
    let engine = match engine_guard.as_mut() {
        Some(e) => e,
        None => return -1,
    };
    
    let (should_continue, _result) = engine.process_command(cmd);
    
    if should_continue {
        1
    } else {
        0
    }
}

/// 獲取輸出
#[no_mangle]
pub extern "C" fn ratamud_get_output() -> *mut c_char {
    let mut engine_guard = match GLOBAL_ENGINE.lock() {
        Ok(guard) => guard,
        Err(_) => return ptr::null_mut(),
    };
    
    let engine = match engine_guard.as_mut() {
        Some(e) => e,
        None => return ptr::null_mut(),
    };
    
    let output = engine.get_output();
    let combined = output.join("\n");
    
    match CString::new(combined) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// 更新遊戲（全局引擎）
#[no_mangle]
pub extern "C" fn ratamud_update(delta_ms: c_int) -> c_int {
    if delta_ms < 0 {
        return -1;
    }
    
    let mut engine_guard = match GLOBAL_ENGINE.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    
    let engine = match engine_guard.as_mut() {
        Some(e) => e,
        None => return -1,
    };
    
    engine.update(delta_ms as u32);
    0
}

/// 獲取玩家資訊（使用全局引擎）
#[no_mangle]
pub extern "C" fn ratamud_get_player_info() -> *mut c_char {
    let engine_guard = match GLOBAL_ENGINE.lock() {
        Ok(guard) => guard,
        Err(_) => return ptr::null_mut(),
    };
    
    let engine = match engine_guard.as_ref() {
        Some(e) => e,
        None => return ptr::null_mut(),
    };
    
    let state_json = engine.get_state_json();
    
    match CString::new(state_json) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// 獲取玩家位置（使用全局引擎）
#[no_mangle]
pub extern "C" fn ratamud_get_player_position(x: *mut c_int, y: *mut c_int) -> c_int {
    if x.is_null() || y.is_null() {
        return -1;
    }
    
    let engine_guard = match GLOBAL_ENGINE.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };
    
    let engine = match engine_guard.as_ref() {
        Some(e) => e,
        None => return -1,
    };
    
    let player = &engine.player;
    unsafe {
        *x = player.x as c_int;
        *y = player.y as c_int;
    }
    0
}

/// 獲取當前地圖（使用全局引擎）
#[no_mangle]
pub extern "C" fn ratamud_get_current_map() -> *mut c_char {
    let engine_guard = match GLOBAL_ENGINE.lock() {
        Ok(guard) => guard,
        Err(_) => return ptr::null_mut(),
    };
    
    let engine = match engine_guard.as_ref() {
        Some(e) => e,
        None => return ptr::null_mut(),
    };
    
    let map_name = engine.world.current_map_name.clone();
    
    match CString::new(map_name) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}
