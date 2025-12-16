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
use map::Map;
use map::MapType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化終端原始模式和備用螢幕
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 初始化輸入和輸出管理器
    let mut input_handler = InputHandler::new();
    let mut output_manager = OutputManager::new();
    
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

    // 初始化遊戲世界
    let mut game_world = GameWorld::new();
    
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
    
    // 更新世界元數據，添加4個地圖名稱
    game_world.metadata.maps = vec![
        "初始之地".to_string(),
        "森林".to_string(),
        "洞穴".to_string(),
        "沙漠".to_string(),
        "山脈".to_string(),
    ];
    
    // 建立 maps 資料夾
    std::fs::create_dir_all(game_world.get_maps_dir())?;
    
    // 生成並保存4張地圖
    let maps_config = vec![
        ("初始之地", MapType::Normal),
        ("森林", MapType::Forest),
        ("洞穴", MapType::Cave),
        ("沙漠", MapType::Desert),
        ("山脈", MapType::Mountain),
    ];

    for (map_name, map_type) in maps_config {
        let map_path = format!("{}/{}.json", game_world.get_maps_dir(), map_name);
        
        let map = if std::path::Path::new(&map_path).exists() {
            // 如果檔案存在，則加載（不要重新初始化物品）
            Map::load(&map_path)?
        } else {
            // 否則生成新地圖
            let mut new_map = Map::new_with_type(map_name.to_string(), 100, 100, map_type);
            // 只在新地圖時初始化物品
            new_map.initialize_items();
            // 保存新地圖
            new_map.save(&map_path)?;
            new_map
        };
        output_manager.log(format!("地圖已加載: {}", map.name));
        let (walkable, unwalkable) = map.get_stats();
        output_manager.log(format!("{map_name} - 可行走點: {walkable}, 不可行走點: {unwalkable}"));
        game_world.add_map(map);
    }
    
    // 保存世界元數據
    let _ = game_world.save_metadata();
    
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
    
    // 先載入 persons 目錄下的所有 NPC 文件
    output_manager.log("開始載入 NPC...".to_string());
    let mut loaded_npc_count = 0;
    
    if let Ok(entries) = std::fs::read_dir(&person_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // 跳過 "me" 文件，因為已經單獨載入了
                    if file_stem == "me" {
                        continue;
                    }
                    
                    // 嘗試載入 NPC
                    if let Ok(npc) = Person::load(&person_dir, file_stem) {
                        let npc_name = npc.name.clone();
                        let npc_x = npc.x;
                        let npc_y = npc.y;
                        
                        // 使用文件名作為 ID，名稱作為別名
                        game_world.npc_manager.add_npc(
                            file_stem.to_string(), 
                            npc, 
                            vec![npc_name.to_lowercase()]
                        );
                        
                        loaded_npc_count += 1;
                        output_manager.log(format!("已載入 NPC: {npc_name} 在位置 ({npc_x}, {npc_y})"));
                    }
                }
            }
        }
    }    
    output_manager.log(format!("從文件載入了 {loaded_npc_count} 個 NPC"));    
    output_manager.log(format!("已加載 {} 個地圖", game_world.map_count()));

    // 顯示歡迎訊息
    show_welcome_message(&mut output_manager, &game_world);
    show_current_map_info(&mut output_manager, &game_world);

    // 如果小地圖已開啟，初始化其內容
    if output_manager.is_minimap_open() {
        app::update_minimap_display(&mut output_manager, &game_world, &me);
    }

    // 運行主迴圈
    app::run_main_loop(&mut terminal, &mut input_handler, &mut output_manager, &mut game_world, &mut me)?;

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
