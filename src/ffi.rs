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

/// 啟動遊戲主程式
/// 可從 main() 或 FFI 外部呼叫
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
    let me = match game_world.npc_manager.initialize(&person_dir) {
        Ok((count, me)) => {
            output_manager.log(format!("已載入 {count} 個角色"));
            for npc in game_world.npc_manager.get_all_npcs() {
                output_manager.log(format!("  - {} 在位置 ({}, {})", npc.name, npc.x, npc.y));
            }
            me
        }
        Err(e) => {
            eprintln!("初始化角色系統失敗: {e}");
            return -1;
        }
    };
    
    // 設定 game_world.original_player
    game_world.original_player = Some(me.clone());
    
    // 載入任務
    load_quest_internal(&mut game_world, &mut output_manager);

    // 載入事件腳本
    load_event_internal(&mut game_world, &mut output_manager);

    // 顯示歡迎訊息
    show_welcome_message_internal(&mut output_manager, &game_world);
    show_current_map_info_internal(&mut output_manager, &game_world);

    // 如果小地圖已開啟，初始化其內容
    if output_manager.is_minimap_open() {
        app::update_minimap_display(&mut output_manager, &game_world, &me);
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
    if app::run_main_loop(terminal, input_handler, output_manager, game_world, me, rx).is_err() {
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

