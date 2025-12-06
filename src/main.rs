// 模組聲明
mod input;
mod output;
mod ui;
mod world;
mod observable;
mod person;
mod map;

use std::io;
use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, KeyCode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Layout, Constraint, Direction, Rect},
    Terminal,
};

use input::{InputHandler, CommandResult};
use output::OutputManager;
use ui::InputDisplay;
use person::Person;
use map::{Map, MapType};
use world::GameWorld;
use observable::WorldInfo;

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
            // 如果檔案存在，則加載
            Map::load(&map_path)?
        } else {
            // 否則生成新地圖
            let new_map = Map::new_with_type(map_name.to_string(), 100, 100, map_type);
            // 保存新地圖
            new_map.save(&map_path)?;
            new_map
        };
        
        output_manager.add_message(format!("地圖已加載: {}", map.name));
        let (walkable, unwalkable) = map.get_stats();
        output_manager.add_message(format!("{} - 可行走點: {}, 不可行走點: {}", map_name, walkable, unwalkable));
        game_world.add_map(map);
    }
    
    // 保存世界元數據
    let _ = game_world.save_metadata();
    
    // 嘗試加載世界元數據
    let _ = game_world.load_metadata();
    
    // 嘗試載入 Me（如果存在）
    let person_dir = format!("{}/persons", game_world.world_dir);
    std::fs::create_dir_all(&person_dir)?;
    
    if let Ok(loaded_me) = Person::load(&person_dir, "me") {
        me = loaded_me;
        output_manager.add_message("已載入角色: Me".to_string());
    } else {
        // 如果沒有存檔，保存初始化的 Me
        let _ = me.save(&person_dir, "me");
        output_manager.add_message("已保存新角色: Me".to_string());
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
            output_manager.add_message(format!("已載入 NPC: {} 在位置 ({}, {})", name, npc.x, npc.y));
        }
    }
    
    output_manager.add_message(format!("已加載 {} 個地圖", game_world.map_count()));

    // 主要事件迴圈
    'main_loop: loop {
        // 更新狀態列（檢查訊息是否過期）
        output_manager.update_status();

        // 繪製終端畫面
        terminal.draw(|f| {
            let size = f.size();

            // 將螢幕分為三個垂直區域：輸出區域、輸入區域、狀態列
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Min(1),      // 輸出區域
                    Constraint::Length(3),   // 輸入區域
                    Constraint::Length(1),   // 狀態列（只有一行）
                ])
                .split(size);

            // 渲染輸出區域（全寬）
            let output_widget = output_manager.render_output(vertical_chunks[0]);
            f.render_widget(output_widget, vertical_chunks[0]);

            // 如果側邊面板打開，渲染懸浮式視窗
            if output_manager.is_side_panel_open() {
                // 計算懸浮視窗的位置和大小（右下角，寬度 40%，高度 60%）
                let floating_width = (size.width as f32 * 0.4) as u16;
                let floating_height = (size.height as f32 * 0.6) as u16;
                let floating_x = size.width.saturating_sub(floating_width + 2);
                let floating_y = 1;
                
                let floating_area = Rect {
                    x: floating_x,
                    y: floating_y,
                    width: floating_width,
                    height: floating_height,
                };
                
                // 渲染懸浮視窗背景（半透明效果用深灰色邊框）
                let side_widget = output_manager.render_side_panel(floating_area);
                f.render_widget(side_widget, floating_area);
            }

            // 渲染輸入區域
            let input_widget = InputDisplay::render_input(input_handler.get_input(), vertical_chunks[1]);
            f.render_widget(input_widget, vertical_chunks[1]);

            // 渲染狀態列
            let status_widget = output_manager.render_status();
            f.render_widget(status_widget, vertical_chunks[2]);
        })?;

        // 檢查是否有鍵盤事件（100ms 超時）
        if event::poll(std::time::Duration::from_millis(100))? {
            let event = event::read()?;
            
            // 處理鍵盤事件
            match event {
                crossterm::event::Event::Key(key) => match key.code {
                    KeyCode::Esc => {
                        // ESC 鍵清除輸入
                        input_handler.clear_input();
                    },
                    KeyCode::F(1) => {
                        // F1 鍵切換側邊面板
                        output_manager.toggle_side_panel();
                    },
                    KeyCode::Up => {
                        // 向上滾動輸出
                        if output_manager.is_side_panel_open() {
                            output_manager.scroll_side_up();
                        } else {
                            output_manager.scroll_up();
                        }
                    },
                    KeyCode::Down => {
                        // 向下滾動輸出
                        let size = terminal.size()?;
                        let message_area_height = if output_manager.is_side_panel_open() {
                            size.height.saturating_sub(2 + 3 + 2) as usize
                        } else {
                            size.height.saturating_sub(2 + 3 + 2) as usize
                        };
                        if output_manager.is_side_panel_open() {
                            output_manager.scroll_side_down(message_area_height);
                        } else {
                            output_manager.scroll_down(message_area_height);
                        }
                    },
                    _ => {
                        // 處理其他鍵盤輸入（字符、Enter、Backspace 等）
                        if let Some(result) = input_handler.handle_event(
                            crossterm::event::Event::Key(key)
                        ) {
                            match result {
                                CommandResult::Exit => {
                                    break 'main_loop;
                                },
                                CommandResult::Output(text) => {
                                    // 正確指令的結果顯示在輸出區，關閉側邊面板
                                    output_manager.add_message(text);
                                    if output_manager.is_side_panel_open() {
                                        output_manager.toggle_side_panel();
                                    }
                                },
                                CommandResult::Ignored => {
                                    // 忽略非命令輸入
                                },
                                CommandResult::Error(err) => {
                                    // 錯誤訊息顯示在狀態列，關閉側邊面板
                                    output_manager.set_status(err);
                                    if output_manager.is_side_panel_open() {
                                        output_manager.toggle_side_panel();
                                    }
                                },
                                CommandResult::Clear => {
                                    // 清除所有訊息，關閉側邊面板
                                    output_manager.clear_messages();
                                    output_manager.set_status("Text area cleared".to_string());
                                    if output_manager.is_side_panel_open() {
                                        output_manager.toggle_side_panel();
                                    }
                                },
                                CommandResult::AddToSide(msg) => {
                                    // 添加訊息到側邊面板
                                    output_manager.add_side_message(msg);
                                    output_manager.set_status("Message added to side panel".to_string());
                                    if output_manager.is_side_panel_open() {
                                        output_manager.toggle_side_panel();
                                    }
                                },
                                CommandResult::ShowStatus => {
                                    // 打開狀態面板，顯示 Me 物件
                                    if !output_manager.is_side_panel_open() {
                                        output_manager.toggle_side_panel();
                                    }
                                    output_manager.set_side_observable(Box::new(me.clone()));
                                    output_manager.set_status(String::new());
                                },
                                CommandResult::ShowWorld => {
                                    // 打開世界資訊面板
                                    if !output_manager.is_side_panel_open() {
                                        output_manager.toggle_side_panel();
                                    }
                                    let world_info = WorldInfo::new(
                                        game_world.metadata.name.clone(),
                                        game_world.metadata.description.clone(),
                                        game_world.metadata.maps.clone(),
                                    );
                                    output_manager.set_side_observable(Box::new(world_info));
                                    output_manager.set_status(String::new());
                                },
                                CommandResult::CloseStatus => {
                                    // 關閉狀態面板
                                    if output_manager.is_side_panel_open() {
                                        output_manager.toggle_side_panel();
                                    }
                                },
                                CommandResult::Look => {
                                    // 查看當前位置
                                    if let Some(current_map) = game_world.get_current_map() {
                                        if let Some(point) = current_map.get_point(me.x, me.y) {
                                            output_manager.add_message(format!("【{}】", point.description));
                                            output_manager.set_status(format!("位置: ({}, {}) - {}", me.x, me.y, current_map.name));
                                        }
                                    }
                                    if output_manager.is_side_panel_open() {
                                        output_manager.toggle_side_panel();
                                    }
                                },
                                CommandResult::Move(dx, dy) => {
                                    // 移動玩家
                                    let new_x = (me.x as i32 + dx) as usize;
                                    let new_y = (me.y as i32 + dy) as usize;
                                    
                                    // 檢查邊界和可走性
                                    if let Some(current_map) = game_world.get_current_map() {
                                        if new_x < current_map.width && new_y < current_map.height {
                                            // 檢查目標點是否可移動
                                            if let Some(point) = current_map.get_point(new_x, new_y) {
                                                if point.walkable {
                                                    me.move_to(new_x, new_y);
                                                    // 保存 Me 的新位置
                                                    let person_dir = format!("{}/persons", game_world.world_dir);
                                                    let _ = me.save(&person_dir, "me");
                                                    let direction = match (dx, dy) {
                                                        (1, 0) => "右",
                                                        (-1, 0) => "左",
                                                        (0, -1) => "上",
                                                        (0, 1) => "下",
                                                        _ => "?",
                                                    };
                                                    output_manager.set_status(format!("往 {} 移動", direction));
                                                } else {
                                                    output_manager.set_status("前方是牆壁，無法通過".to_string());
                                                }
                                            }
                                        } else {
                                            output_manager.set_status("超出地圖範圍".to_string());
                                        }
                                    }
                                    if output_manager.is_side_panel_open() {
                                        output_manager.toggle_side_panel();
                                    }
                                },
                            }
                        }
                    }
                },
                _ => {}
            }
        }
    }

    // 清理終端設定並返回到常規模式
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
