use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

use crate::person::Person;
use crate::world::GameWorld;

/// 創建新玩家
/// 返回不透明指針，失敗則返回 null
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
/// 返回不透明指針，失敗則返回 null
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
/// 返回 0 表示成功，非 0 表示失敗
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

/// 獲取玩家位置
#[no_mangle]
pub extern "C" fn ratamud_get_player_position(player: *const Person, x: *mut c_int, y: *mut c_int) -> c_int {
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
    player_ref.x = x as usize;
    player_ref.y = y as usize;
    0
}

/// 獲取當前地圖名稱
#[no_mangle]
pub extern "C" fn ratamud_get_current_map(world: *const GameWorld) -> *mut c_char {
    if world.is_null() {
        return ptr::null_mut();
    }
    
    let world_ref = unsafe { &*world };
    match CString::new(world_ref.current_map_name.clone()) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// 獲取玩家資訊（JSON 格式）
#[no_mangle]
pub extern "C" fn ratamud_get_player_info(player: *const Person) -> *mut c_char {
    if player.is_null() {
        return ptr::null_mut();
    }
    
    let player_ref = unsafe { &*player };
    let info = serde_json::json!({
        "name": player_ref.name,
        "position": (player_ref.x, player_ref.y),
        "map": player_ref.map,
        "hp": player_ref.hp,
        "mp": player_ref.mp,
        "max_hp": player_ref.max_hp,
        "max_mp": player_ref.max_mp,
        "status": player_ref.status,
    });
    
    if let Ok(json_str) = serde_json::to_string(&info) {
        if let Ok(c_string) = CString::new(json_str) {
            return c_string.into_raw();
        }
    }
    ptr::null_mut()
}

/// 釋放由 ratamud_* 函數分配的字串
#[no_mangle]
pub extern "C" fn ratamud_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

/// 獲取玩家名稱
#[no_mangle]
pub extern "C" fn ratamud_get_player_name(player: *const Person) -> *mut c_char {
    if player.is_null() {
        return ptr::null_mut();
    }
    
    let player_ref = unsafe { &*player };
    match CString::new(player_ref.name.clone()) {
        Ok(c_string) => c_string.into_raw(),
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

/// 獲取版本資訊
#[no_mangle]
pub extern "C" fn ratamud_version() -> *const c_char {
    "RataMUD v0.1.0\0".as_ptr() as *const c_char
}
