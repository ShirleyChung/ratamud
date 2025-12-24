// 模組聲明
mod input;
mod output;
mod ui;
mod world;
mod observable;
mod person;
mod npc_manager;
mod npc_ai;
mod trade;
mod quest;
mod map;
mod time_updatable;
mod time_thread;
mod npc_ai_thread;
mod item;
mod item_registry;
mod settings;
mod app;
mod event;
mod event_scheduler;
mod event_executor;
mod event_loader;
mod callback;  // 新增 callback 模組
mod command_processor;  // 新增：命令處理器
mod game_engine;        // 新增：遊戲引擎

use std::io;
use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

use input::InputHandler;
use output::OutputManager;
use person::Person;
use world::GameWorld;
use ui::Menu;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化終端原始模式和備用螢幕
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // 初始化 InputHandler, OutputManager, GameWorld, Person
    let input_handler = InputHandler::new();
    let mut output_manager = OutputManager::new();
    
    // 初始化 Menu 狀態
    let menu: Option<Menu> = None;
    
    // 載入遊戲設定
    use settings::GameSettings;
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

    // 初始化 Me 物件 (Player)
    let mut me = Person::new(
        "勇士".to_string(),
        "冒險的勇士，探索未知的世界".to_string(),
    );
    me.add_ability("劍術".to_string());
    me.add_ability("魔法".to_string());
    me.add_ability("探險".to_string());
    me.add_item("木劍".to_string());
    me.add_item("魔法書".to_string());
    me.add_item("治療藥水".to_string());
    me.set_status("精力充沛".to_string());

    // 初始化遊戲世界，並將 me 物件移入
    let mut game_world = GameWorld::new(me.clone());
    
    // 嘗試加載世界元數據和時間
    let _ = game_world.load_metadata();
    let _ = game_world.load_time();  // 載入保存的世界時間
    
    // 設置初始時間顯示
    output_manager.set_current_time(game_world.format_time());
    
    // 載入事件腳本
    let events_dir = format!("{}/events", game_world.world_dir);
    match event_loader::EventLoader::load_from_directory(&mut game_world.event_manager, &events_dir) {
        Ok((count, event_list)) => {
            if count > 0 {
                output_manager.log(format!("✅ 載入了 {count} 個事件"));
                for event_name in event_list {
                    output_manager.log(format!("  📌 {event_name}"));
                }
            }
        }
        Err(e) => {
            output_manager.log(format!("⚠️  載入事件失敗: {e}"));
        }
    }    
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
    
    // 嘗試載入 Me（如果存在）
    let person_dir = format!("{}/persons", game_world.world_dir);
    std::fs::create_dir_all(&person_dir)?;
    
    if let Ok(loaded_me) = Person::load(&person_dir, "me") {
        me = loaded_me;
        output_manager.log("已載入角色: Me".to_string());
    } else {
        // 如果沒有存檔，保存初始化的 Me
        let _ = me.save(&person_dir, "me");
        output_manager.log("已保存新角色: Me".to_string());
    }
    
    // 載入所有 NPC
    output_manager.log("開始載入 NPC...".to_string());
    match game_world.npc_manager.load_all_from_directory(&person_dir, vec!["me"]) {
        Ok(count) => {
            output_manager.log(format!("從文件載入了 {count} 個 NPC"));
            
            // 記錄每個 NPC 的詳細資訊
            for npc in game_world.npc_manager.get_all_npcs() {
                output_manager.log(format!("已載入 NPC: {} 在位置 ({}, {})", npc.name, npc.x, npc.y));
            }
        }
        Err(e) => {
            output_manager.log(format!("⚠️  載入 NPC 失敗: {e}"));
        }
    }
    
    // 載入任務
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

    // 顯示歡迎訊息
    show_welcome_message(&mut output_manager, &game_world);
    show_current_map_info(&mut output_manager, &game_world);

    // 如果小地圖已開啟，初始化其內容
    if output_manager.is_minimap_open() {
        app::update_minimap_display(&mut output_manager, &game_world, &me);
    }

    // 運行主迴圈
    app::run_main_loop(terminal, input_handler, output_manager, game_world, me, menu)?;

    // 清理終端設定並返回到常規模式
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

/// 顯示世界歡迎訊息
fn show_welcome_message(output_manager: &mut output::OutputManager, game_world: &world::GameWorld) {
    output_manager.print(format!("✨ 歡迎來到 {} ✨", game_world.metadata.name));
    output_manager.print(game_world.metadata.description.clone());
    output_manager.print("".to_string());
    output_manager.print("💡 輸入 'help' 查看可用指令".to_string());
    output_manager.print("".to_string());
}

/// 顯示當前地圖資訊
fn show_current_map_info(output_manager: &mut output::OutputManager, game_world: &world::GameWorld) {
    if let Some(current_map) = game_world.get_current_map() {
        output_manager.print(format!("📍 當前區域: {}", current_map.name));
        output_manager.print(current_map.description.clone());
    }
}
