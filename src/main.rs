// 模組聲明
mod input;
mod output;
mod ui;
mod world;
mod observable;
mod person;
mod map;
mod time_updatable;
mod item;
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
    output_manager.print(format!("載入設定: show_minimap = {}", game_settings.show_minimap));
    if game_settings.show_minimap {
        output_manager.show_minimap();
        output_manager.print("小地圖已開啟".to_string());
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
        Ok(count) => {
            if count > 0 {
                output_manager.print(format!("✅ 載入了 {} 個事件", count));
            }
        }
        Err(e) => {
            output_manager.print(format!("⚠️  載入事件失敗: {}", e));
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
        
        let mut map = if std::path::Path::new(&map_path).exists() {
            // 如果檔案存在，則加載
            Map::load(&map_path)?
        } else {
            // 否則生成新地圖
            let new_map = Map::new_with_type(map_name.to_string(), 100, 100, map_type);
            // 保存新地圖
            new_map.save(&map_path)?;
            new_map
        };
        map.initialize_items();
        output_manager.print(format!("地圖已加載: {}", map.name));
        let (walkable, unwalkable) = map.get_stats();
        output_manager.print(format!("{} - 可行走點: {}, 不可行走點: {}", map_name, walkable, unwalkable));
        game_world.add_map(map);
    }
    
    // 保存世界元數據
    let _ = game_world.save_metadata();
    
    // 顯示當前時間
    output_manager.print(format!("⏰ {}", game_world.format_time()));
    
    // 嘗試載入 Me（如果存在）
    let person_dir = format!("{}/persons", game_world.world_dir);
    std::fs::create_dir_all(&person_dir)?;
    
    if let Ok(loaded_me) = Person::load(&person_dir, "me") {
        me = loaded_me;
        output_manager.print("已載入角色: Me".to_string());
    } else {
        // 如果沒有存檔，保存初始化的 Me
        let _ = me.save(&person_dir, "me");
        output_manager.print("已保存新角色: Me".to_string());
    }
    
    // 生成或加載 NPC
    let npc_types = vec![
        ("merchant", "商人", "精明的商人，販售各種物品"),
        ("traveler", "路人", "友善的旅者，經過森林"),
        ("doctor", "醫生", "熟練的醫生，治療傷口"),
        ("worker", "工人", "努力的工人，從事建築工作"),
        ("farmer", "農夫", "勤勞的農夫，種植農作物"),
    ];
    
    if let Some(forest_map) = game_world.get_current_map() {
        let walkable_points = forest_map.get_walkable_points();
        
        for (i, (npc_id, name, desc)) in npc_types.iter().enumerate() {
            let npc = if let Ok(loaded_npc) = Person::load(&person_dir, npc_id) {
                loaded_npc
            } else {
                let mut new_npc = Person::new(name.to_string(), desc.to_string());
                if i < walkable_points.len() {
                    let (x, y) = walkable_points[i];
                    new_npc.move_to(x, y);
                }
                let _ = new_npc.save(&person_dir, npc_id);
                new_npc
            };
            output_manager.print(format!("已載入 NPC: {} 在位置 ({}, {})", name, npc.x, npc.y));
        }
    }
    
    output_manager.print(format!("已加載 {} 個地圖", game_world.map_count()));

    // 如果小地圖已開啟，初始化其內容
    if output_manager.is_minimap_open() {
        if let Some(current_map) = game_world.get_current_map() {
            let mut minimap_data = vec![format!("【位置: ({}, {})】", me.x, me.y)];
            
            // 上方
            if me.y > 0 {
                if let Some(point) = current_map.get_point(me.x, me.y - 1) {
                    let walkable = if point.walkable { '✓' } else { '✗' };
                    minimap_data.push(format!("↑ {} {}", point.description, walkable));
                }
            } else {
                minimap_data.push("↑ (邊界)".to_string());
            }
            
            // 下方
            if me.y + 1 < current_map.height {
                if let Some(point) = current_map.get_point(me.x, me.y + 1) {
                    let walkable = if point.walkable { '✓' } else { '✗' };
                    minimap_data.push(format!("↓ {} {}", point.description, walkable));
                }
            } else {
                minimap_data.push("↓ (邊界)".to_string());
            }
            
            // 左方
            if me.x > 0 {
                if let Some(point) = current_map.get_point(me.x - 1, me.y) {
                    let walkable = if point.walkable { '✓' } else { '✗' };
                    minimap_data.push(format!("← {} {}", point.description, walkable));
                }
            } else {
                minimap_data.push("← (邊界)".to_string());
            }
            
            // 右方
            if me.x + 1 < current_map.width {
                if let Some(point) = current_map.get_point(me.x + 1, me.y) {
                    let walkable = if point.walkable { '✓' } else { '✗' };
                    minimap_data.push(format!("→ {} {}", point.description, walkable));
                }
            } else {
                minimap_data.push("→ (邊界)".to_string());
            }
            
            output_manager.update_minimap(minimap_data);
        }
    }

    // 運行主迴圈
    app::run_main_loop(&mut terminal, &mut input_handler, &mut output_manager, &mut game_world, &mut me)?;

    // 清理終端設定並返回到常規模式
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
