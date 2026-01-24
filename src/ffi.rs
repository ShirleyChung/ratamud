#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr};
use std::os::raw::{c_char, c_int};

use std::ffi::CString;
use std::sync::Mutex;

/// 輸出回調函數類型 (新版：帶類型標記)
/// 參數: msg_type (類型標記: MAIN/LOG/STATUS/SIDE), content (內容)
/// 用於將遊戲輸出傳遞給外部（如 iOS/Android UI 或文件）
pub type OutputCallback = extern "C" fn(*const c_char, *const c_char);

/// 全局回調函數存儲
static OUTPUT_CALLBACK: Mutex<Option<OutputCallback>> = Mutex::new(None);

/// 註冊輸出回調
/// 當遊戲有新輸出時，會調用此回調
/// 
/// 回調函數簽名: fn(msg_type: *const c_char, content: *const c_char)
/// msg_type 可能的值: "MAIN", "LOG", "STATUS", "SIDE"
#[no_mangle]
pub extern "C" fn ratamud_register_output_callback(callback: OutputCallback) {
    let mut cb = OUTPUT_CALLBACK.lock().unwrap();
    *cb = Some(callback);
}

/// 內部函數：觸發輸出回調（帶類型標記）
#[allow(dead_code)]
pub(crate) fn trigger_output_callback(msg_type: &str, content: &str) {
    let cb = OUTPUT_CALLBACK.lock().unwrap();
    if let Some(callback) = *cb {
        if let (Ok(type_c), Ok(content_c)) = (CString::new(msg_type), CString::new(content)) {
            callback(type_c.as_ptr(), content_c.as_ptr());
        }
    }
}

/// 狀態變化回調類型
/// 參數: state_json (JSON格式的遊戲狀態)
pub type StateCallback = extern "C" fn(*const c_char);

/// 全局狀態回調存儲
static STATE_CALLBACK: Mutex<Option<StateCallback>> = Mutex::new(None);

/// 註冊狀態變化回調
#[no_mangle]
pub extern "C" fn ratamud_register_state_callback(callback: StateCallback) {
    let mut cb = STATE_CALLBACK.lock().unwrap();
    *cb = Some(callback);
}

/// 內部函數：觸發狀態回調
#[allow(dead_code)]
pub(crate) fn trigger_state_callback(state_json: &str) {
    let cb = STATE_CALLBACK.lock().unwrap();
    if let Some(callback) = *cb {
        if let Ok(c_string) = CString::new(state_json) {
            callback(c_string.as_ptr());
        }
    }
}

/// 事件回調類型
/// 參數: event_type, event_data (JSON)
pub type EventCallback = extern "C" fn(*const c_char, *const c_char);

/// 全局事件回調存儲
static EVENT_CALLBACK: Mutex<Option<EventCallback>> = Mutex::new(None);

/// 註冊事件回調
#[no_mangle]
pub extern "C" fn ratamud_register_event_callback(callback: EventCallback) {
    let mut cb = EVENT_CALLBACK.lock().unwrap();
    *cb = Some(callback);
}

/// 內部函數：觸發事件回調
#[allow(dead_code)]
pub(crate) fn trigger_event_callback(event_type: &str, event_data: &str) {
    let cb = EVENT_CALLBACK.lock().unwrap();
    if let Some(callback) = *cb {
        if let (Ok(type_c), Ok(data_c)) = (CString::new(event_type), CString::new(event_data)) {
            callback(type_c.as_ptr(), data_c.as_ptr());
        }
    }
}

/// 處理命令
#[no_mangle]
pub extern "C" fn ratamud_input_command(command: *const c_char) -> c_int {
    if command.is_null() {
        return -1;
    }
    
    let c_str = unsafe { CStr::from_ptr(command) };
    let _cmd = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    // You can process `cmd` here as needed
    0
}

#[no_mangle]
pub extern "C" fn ratamud_start_game() {
    // Start the game logic here
}

/// 測試輸出回調功能
/// 會生成各種類型的測試輸出
#[no_mangle]
pub extern "C" fn ratamud_test_output_callback() {
    use crate::output::OutputManager;
    
    let mut output = OutputManager::new();
    
    // 測試各種類型的輸出
    output.print("歡迎來到 RataMUD！".to_string());
    output.print("你站在一個廣場中央。".to_string());
    output.log("遊戲初始化完成".to_string());
    output.log("載入地圖: town_square".to_string());
    output.set_status("保存成功".to_string());
    output.set_side_content("NPC: 商人\n等級: 10\n生命: 100/100".to_string());
    output.print("一隻野豬向你衝來！".to_string());
}

