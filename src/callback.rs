use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::Mutex;

/// 輸出回調函數類型
/// 參數: message (C字串)
/// 用於將遊戲輸出傳遞給外部（如 iOS/Android UI）
pub type OutputCallback = extern "C" fn(*const c_char);

/// 全局回調函數存儲
static OUTPUT_CALLBACK: Mutex<Option<OutputCallback>> = Mutex::new(None);

/// 註冊輸出回調
/// 當遊戲有新輸出時，會調用此回調
#[no_mangle]
pub extern "C" fn ratamud_register_output_callback(callback: OutputCallback) {
    let mut cb = OUTPUT_CALLBACK.lock().unwrap();
    *cb = Some(callback);
}

/// 取消註冊輸出回調
#[no_mangle]
pub extern "C" fn ratamud_unregister_output_callback() {
    let mut cb = OUTPUT_CALLBACK.lock().unwrap();
    *cb = None;
}

/// 內部函數：觸發輸出回調
pub(crate) fn trigger_output_callback(message: &str) {
    let cb = OUTPUT_CALLBACK.lock().unwrap();
    if let Some(callback) = *cb {
        if let Ok(c_string) = CString::new(message) {
            callback(c_string.as_ptr());
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

/// 取消註冊狀態回調
#[no_mangle]
pub extern "C" fn ratamud_unregister_state_callback() {
    let mut cb = STATE_CALLBACK.lock().unwrap();
    *cb = None;
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

/// 取消註冊事件回調
#[no_mangle]
pub extern "C" fn ratamud_unregister_event_callback() {
    let mut cb = EVENT_CALLBACK.lock().unwrap();
    *cb = None;
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
