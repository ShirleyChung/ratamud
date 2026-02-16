#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::core_output;
use crate::world::GameWorld;

/// 全局遊戲世界實例（FFI 和其他非 UI 模式共用）
static GAME_WORLD: Lazy<Mutex<Option<GameWorld>>> = Lazy::new(|| Mutex::new(None));

/// 輸出回調函數類型 (C FFI)
/// 參數: msg_type (類型標記: MAIN/LOG/STATUS/SIDE), content (內容)
pub type OutputCallback = extern "C" fn(*const c_char, *const c_char);

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
    
    if should_continue {
        1 // 繼續
    } else {
        0 // 退出
    }
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

