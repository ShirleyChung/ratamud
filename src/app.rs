use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use ratatui::layout::{Layout, Constraint, Direction, Rect};
use ratatui::widgets::Clear;
use std::io;
use crossterm::event::{self, KeyCode};
use std::time::{Duration, Instant};

use crate::input::InputHandler;
use crate::output::OutputManager;
use crate::world::GameWorld;
use crate::settings::GameSettings;
use crate::person::Person;
use crate::observable::WorldInfo;
use crate::input::CommandResult;
use crate::item_registry;
use crate::ui::{InputDisplay, HeaderDisplay};

/// 應用程式主迴圈 - 將 main.rs 中的事件迴圈邏輯提取到此
pub fn run_main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    input_handler: &mut InputHandler,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut should_exit = false;
    let mut last_event_check = Instant::now();
    let event_check_interval = Duration::from_millis(100);  // 每0.1秒檢查事件
    
    'main_loop: loop {
        // 更新狀態列（檢查訊息是否過期）
        output_manager.update_status();
        
        // 從時鐘線程同步時間
        game_world.update_time();
        
        // 定期檢查並觸發事件
        let now = Instant::now();
        if now.duration_since(last_event_check) >= event_check_interval {
            check_and_execute_events(game_world, me, output_manager);
            last_event_check = now;
        }
        // 繪製終端畫面
        terminal.draw(|f| {
            let size = f.size();

            // 將螢幕分為四個垂直區域：標題列、輸出區域、輸入區域、狀態列
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),   // 標題列
                    Constraint::Min(1),      // 輸出區域
                    Constraint::Length(3),   // 輸入區域
                    Constraint::Length(1),   // 狀態列
                ])
                .split(size);

            // 渲染標題列
            let current_time_str = game_world.format_time();
            let header_widget = HeaderDisplay::render_header(
                "初始世界",
                &current_time_str
            );
            f.render_widget(header_widget, vertical_chunks[0]);

            // 渲染輸出區域
            let output_widget = output_manager.render_output(vertical_chunks[1]);
            f.render_widget(output_widget, vertical_chunks[1]);

            // 計算小地圖的位置和大小（右上角，根據內容自動調整高度）
            let minimap_width = (size.width as f32 * 0.35) as u16;  // 縮小寬度
            // 小地圖固定顯示: 標題(1) + 位置(1) + 4個方向(4) + 邊框(2) = 8行
            let minimap_height = 8u16;  
            let minimap_x = size.width.saturating_sub(minimap_width);
            let minimap_y = 1;  // 從標題列下方開始
            
            let minimap_area = Rect {
                x: minimap_x,
                y: minimap_y,
                width: minimap_width,
                height: minimap_height,
            };
            // 畫小地圖
            if output_manager.is_minimap_open() {
                let minimap_widget = output_manager.get_minimap(minimap_area);
                f.render_widget(Clear, minimap_area); // 清除背景
                f.render_widget(minimap_widget, minimap_area);
            }

            // 計算日誌視窗位置和大小（右側，在小地圖下方）
            let log_width = minimap_width;  // 與小地圖同寬
            let log_height = (size.height as f32 * 0.45) as u16;  // 增加高度
            let log_x = size.width.saturating_sub(log_width);
            let log_y = minimap_y + minimap_height + 1;  // 緊接著小地圖下方
            
            let log_area = Rect {
                x: log_x,
                y: log_y,
                width: log_width,
                height: log_height,
            };
            // 畫日誌視窗
            if output_manager.is_log_open() {
                let log_widget = output_manager.render_log(log_area);
                f.render_widget(Clear, log_area); // 清除背景
                f.render_widget(log_widget, log_area);
            }
            
            // 側邊面板使用動態高度
            let side_panel_height = if output_manager.is_side_panel_open() {
                let content_height = output_manager.get_side_panel_content_height();
                // 確保不超過螢幕高度，留出空間給輸入和狀態列
                let max_height = size.height.saturating_sub(vertical_chunks[2].height + vertical_chunks[3].height + 2);
                content_height.min(max_height)
            } else {
                minimap_height
            };
            
            let floating_area = Rect {
                x: minimap_x,
                y: minimap_y,
                width: minimap_width,
                height: side_panel_height,
            };
            // 畫側邊面板
            if output_manager.is_side_panel_open() {
                let side_widget = output_manager.get_side_panel(floating_area);
                f.render_widget(Clear, floating_area); // 清除背景
                f.render_widget(side_widget, floating_area);
            }
            
            // 渲染大地圖（置中懸浮視窗）
            if output_manager.is_map_open() {
                if let Some(current_map) = game_world.get_current_map() {
                    // 計算置中的懸浮視窗位置
                    let map_width = (size.width as f32 * 0.8) as u16;
                    let map_height = (size.height as f32 * 0.8) as u16;
                    let map_x = (size.width.saturating_sub(map_width)) / 2;
                    let map_y = (size.height.saturating_sub(map_height)) / 2;
                    
                    let map_area = Rect {
                        x: map_x,
                        y: map_y,
                        width: map_width,
                        height: map_height,
                    };
                    
                    let map_widget = output_manager.render_big_map(map_area, current_map, me.x, me.y, &game_world.npc_manager);
                    f.render_widget(Clear, map_area);
                    f.render_widget(map_widget, map_area);
                }
            }
            
            // 渲染輸入區域
            let input_widget = InputDisplay::render_input(input_handler.get_input(), vertical_chunks[2]);
            f.render_widget(input_widget, vertical_chunks[2]);

            // 渲染狀態列
            let status_widget = output_manager.render_status();
            f.render_widget(status_widget, vertical_chunks[3]);
        })?;

        if should_exit {
            break 'main_loop;
        }

        // 檢查是否有鍵盤事件（100ms 超時）
        if event::poll(Duration::from_millis(100))? {
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
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        // 如果大地圖開啟，q 鍵關閉地圖
                        if output_manager.is_map_open() {
                            output_manager.close_map();
                            output_manager.set_status("大地圖已關閉".to_string());
                        } else {
                            // 否則當作正常輸入處理
                            if let Some(result) = input_handler.handle_event(
                                crossterm::event::Event::Key(key)
                            ) {
                                if let CommandResult::Exit = result {
                                    should_exit = true;
                                } else {
                                    handle_command_result(result, output_manager, game_world, me)?;
                                }
                            }
                        }
                    },
                    // 上下左右鍵優先用於移動
                    KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                        // 如果大地圖開啟，用方向鍵移動地圖視圖
                        if output_manager.is_map_open() {
                            if let Some(current_map) = game_world.get_current_map() {
                                let (dx, dy) = match key.code {
                                    KeyCode::Up => (0, -5),
                                    KeyCode::Down => (0, 5),
                                    KeyCode::Left => (-5, 0),
                                    KeyCode::Right => (5, 0),
                                    _ => (0, 0),
                                };
                                output_manager.move_map_view(dx, dy, current_map.width, current_map.height);
                            }
                        } else {
                            // 否則將方向鍵傳遞給 input_handler 處理移動
                            if let Some(result) = input_handler.handle_event(
                                crossterm::event::Event::Key(key)
                            ) {
                                if let CommandResult::Exit = result {
                                    should_exit = true;
                                } else {
                                    handle_command_result(result, output_manager, game_world, me)?;
                                }
                            }
                        }
                    },
                    _ => {
                        // 處理其他鍵盤輸入（字符、Enter、Backspace 等）
                        if let Some(result) = input_handler.handle_event(
                            crossterm::event::Event::Key(key)
                        ) {
                            if let CommandResult::Exit = result {
                                should_exit = true;
                            } else {
                                handle_command_result(result, output_manager, game_world, me)?;
                            }
                        }
                    }
                },
                _ => {}
            }
        }
    }

    // 保存所有數據
    game_world.save_metadata()?;
    game_world.save_time()?;  // 保存世界時間
    let mut game_settings = GameSettings::default();
    game_settings.show_minimap = output_manager.is_minimap_open();
    game_settings.show_log = output_manager.is_log_open();
    let _ = game_settings.save();

    Ok(())
}

/// 處理命令結果 - 主分派函式
fn handle_command_result(
    result: CommandResult,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) -> Result<(), Box<dyn std::error::Error>> {
    output_manager.close_side_panel();
    match result {
        CommandResult::Exit => handle_exit(output_manager, game_world)?,
        CommandResult::Help => handle_help(output_manager),
        CommandResult::Output(text) => handle_output(text, output_manager),
        CommandResult::Error(err) => handle_error(err, output_manager),
        CommandResult::Clear => handle_clear(output_manager),
        CommandResult::AddToSide(msg) => handle_add_to_side(msg, output_manager),
        CommandResult::ShowStatus => handle_show_status(output_manager, me),
        CommandResult::ShowWorld => handle_show_world(output_manager, game_world),
        CommandResult::ShowMinimap => handle_show_minimap(output_manager, game_world, me),
        CommandResult::HideMinimap => handle_hide_minimap(output_manager),
        CommandResult::ShowLog => handle_show_log(output_manager),
        CommandResult::HideLog => handle_hide_log(output_manager),
        CommandResult::ShowMap => handle_show_map(output_manager, me),
        CommandResult::Look(target) => display_look(target, output_manager, game_world, me),
        CommandResult::Move(dx, dy) => handle_movement(dx, dy, output_manager, game_world, me)?,
        CommandResult::Get(item_name, quantity) => handle_get(item_name, quantity, output_manager, game_world, me),
        CommandResult::Drop(item_name, quantity) => handle_drop(item_name, quantity, output_manager, game_world, me),
        CommandResult::Summon(npc_name) => handle_summon(npc_name, output_manager, game_world, me),
        CommandResult::Conquer(direction) => handle_conquer(direction, output_manager, game_world, me)?,
        CommandResult::FlyTo(target) => handle_flyto(target, output_manager, game_world, me)?,
        CommandResult::NameHere(name) => handle_namehere(name, output_manager, game_world, me)?,
        CommandResult::Name(target, name) => handle_name(target, name, output_manager, game_world, me)?,
        CommandResult::Destroy(target) => handle_destroy(target, output_manager, game_world, me)?,
    }
    Ok(())
}

/// 處理退出命令
fn handle_exit(
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    game_world.save_metadata()?;
    game_world.save_time()?;  // 保存世界時間
    let mut game_settings = GameSettings::default();
    game_settings.show_minimap = output_manager.is_minimap_open();
    let _ = game_settings.save();
    Ok(())
}

/// 處理幫助命令
fn handle_help(output_manager: &mut OutputManager) {
    output_manager.print("".to_string());
    output_manager.print("═══════════════════════════════════════".to_string());
    output_manager.print("📖 可用指令".to_string());
    output_manager.print("═══════════════════════════════════════".to_string());
    output_manager.print("".to_string());
    
    // 使用 CommandResult 提供的幫助資訊
    for (category, commands) in CommandResult::get_help_info() {
        output_manager.print(category.to_string());
        for (command, description) in commands {
            output_manager.print(format!("  {:<16} - {}", command, description));
        }
        output_manager.print("".to_string());
    }
    
    output_manager.set_status("輸入任意指令開始遊戲".to_string());
}

/// 處理輸出結果
fn handle_output(text: String, output_manager: &mut OutputManager) {
    output_manager.print(text);
}

/// 處理錯誤訊息
fn handle_error(err: String, output_manager: &mut OutputManager) {
    output_manager.set_status(err);
}

/// 處理清除訊息
fn handle_clear(output_manager: &mut OutputManager) {
    output_manager.clear_messages();
    output_manager.set_status("Text area cleared".to_string());
}

/// 處理添加到側邊面板
fn handle_add_to_side(msg: String, output_manager: &mut OutputManager) {
    output_manager.add_side_message(msg);
    output_manager.set_status("Message added to side panel".to_string());
    if output_manager.is_side_panel_open() {
        output_manager.toggle_side_panel();
    }
}

/// 處理顯示狀態面板
fn handle_show_status(output_manager: &mut OutputManager, me: &Person) {
    // 顯示狀態面板
    if !output_manager.is_side_panel_open() {
        output_manager.toggle_side_panel();
    }
    output_manager.set_side_observable(Box::new(me.clone()));
    output_manager.set_status("已顯示角色狀態".to_string());
}

/// 處理顯示世界資訊
fn handle_show_world(output_manager: &mut OutputManager, game_world: &GameWorld) {
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
}

/// 處理顯示小地圖
fn handle_show_minimap(
    output_manager: &mut OutputManager,
    game_world: &GameWorld,
    me: &Person,
) {
    output_manager.show_minimap();
    update_minimap_display(output_manager, game_world, me);
    output_manager.set_status(String::new());
}

/// 處理隱藏小地圖
fn handle_hide_minimap(output_manager: &mut OutputManager) {
    output_manager.hide_minimap();
    output_manager.set_status(String::new());
}

/// 處理顯示日誌視窗
fn handle_show_log(output_manager: &mut OutputManager) {
    output_manager.show_log_window();
    output_manager.set_status("日誌視窗已開啟".to_string());
}

/// 處理隱藏日誌視窗
fn handle_hide_log(output_manager: &mut OutputManager) {
    output_manager.hide_log();
    output_manager.set_status("日誌視窗已關閉".to_string());
}

/// 處理顯示大地圖
fn handle_show_map(output_manager: &mut OutputManager, me: &Person) {
    output_manager.show_map(me.x, me.y);
    output_manager.set_status("大地圖已開啟 (↑↓←→移動, q退出)".to_string());
}

/// 處理關閉狀態面板
#[allow(dead_code)]
fn handle_close_status(output_manager: &mut OutputManager) {
    if output_manager.is_side_panel_open() {
        output_manager.toggle_side_panel();
    }
}

/// 顯示 look 命令的結果
fn display_look(
    target: Option<String>,
    output_manager: &mut OutputManager,
    game_world: &GameWorld,
    me: &Person,
) {
    // 如果有指定目標，則查看 NPC
    if let Some(target_name) = target {
        if let Some(npc) = game_world.npc_manager.get_npc(&target_name) {
            // 顯示 NPC 資訊
            output_manager.print("".to_string());
            output_manager.print(format!("👤 {}", npc.name));
            output_manager.print("═".repeat(40));
            output_manager.print(format!("📝 {}", npc.description));
            output_manager.print(format!("📍 位置: ({}, {})", npc.x, npc.y));
            output_manager.print(format!("💫 狀態: {}", npc.status));
            
            if !npc.abilities.is_empty() {
                output_manager.print("\n✨ 能力:".to_string());
                for ability in &npc.abilities {
                    output_manager.print(format!("  • {}", ability));
                }
            }
            
            if !npc.items.is_empty() {
                output_manager.print("\n🎒 攜帶物品:".to_string());
                for (item, count) in &npc.items {
                    let display_name = item_registry::get_item_display_name(item);
                    output_manager.print(format!("  • {} x{}", display_name, count));
                }
            }
            
            output_manager.print("".to_string());
        } else {
            output_manager.set_status(format!("找不到 NPC: {}", target_name));
        }
        return;
    }
    
    // 否則查看當前位置
    if let Some(current_map) = game_world.get_current_map() {
        // 顯示當前位置信息
        if let Some(point) = current_map.get_point(me.x, me.y) {
            output_manager.print( format!("【當前位置: ({}, {})】\n【{}】", me.x, me.y, point.description) );
            
            // 顯示地點名稱（如果有）
            if !point.name.is_empty() {
                output_manager.print(format!("此處是【{}】", point.name));
            }
            
            // 顯示當前位置的 items
            if !point.objects.is_empty() {
                output_manager.print(format!("\n🎁 此處物品:"));
                for (obj, count) in &point.objects {
                    let display_name = item_registry::get_item_display_name(obj);
                    output_manager.print(format!("  • {} x{}", display_name, count));
                }
            }
            
            // 顯示當前位置的 NPC
            let npcs_here = game_world.npc_manager.get_npcs_at(me.x, me.y);
            if !npcs_here.is_empty() {
                output_manager.print(format!("\n👥 此處的人物:"));
                for npc in npcs_here {
                    output_manager.print(format!("  • {} - {}", npc.name, npc.description));
                }
            }
            
            output_manager.print("".to_string());          
        }
    }
}

/// 更新小地圖顯示
pub fn update_minimap_display(
    output_manager: &mut OutputManager,
    game_world: &GameWorld,
    me: &Person,
) {
    if let Some(current_map) = game_world.get_current_map() {
        let mut minimap_data = vec![format!("【位置: ({}, {})】", me.x, me.y)];
        
        // 上方
        if me.y > 0 {
            if let Some(point) = current_map.get_point(me.x, me.y - 1) {
                let walkable = if point.walkable { '\u{2713}' } else { '\u{2718}' };
                minimap_data.push(format!("↑ {} {}", point.description, walkable));
            }
        } else {
            minimap_data.push("↑ (邊界)".to_string());
        }
        
        // 下方
        if me.y + 1 < current_map.height {
            if let Some(point) = current_map.get_point(me.x, me.y + 1) {
                let walkable = if point.walkable { '\u{2713}' } else { '\u{2718}' };
                minimap_data.push(format!("↓ {} {}", point.description, walkable));
            }
        } else {
            minimap_data.push("↓ (邊界)".to_string());
        }
        
        // 左方
        if me.x > 0 {
            if let Some(point) = current_map.get_point(me.x - 1, me.y) {
                let walkable = if point.walkable { '\u{2713}' } else { '\u{2718}' };
                minimap_data.push(format!("← {} {}", point.description, walkable));
            }
        } else {
            minimap_data.push("← (邊界)".to_string());
        }
        
        // 右方
        if me.x + 1 < current_map.width {
            if let Some(point) = current_map.get_point(me.x + 1, me.y) {
                let walkable = if point.walkable { '\u{2713}' } else { '\u{2718}' };
                minimap_data.push(format!("→ {} {}", point.description, walkable));
            }
        } else {
            minimap_data.push("→ (邊界)".to_string());
        }
        
        output_manager.update_minimap(minimap_data);
    }
}

/// 處理移動命令
fn handle_movement(
    dx: i32,
    dy: i32,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) -> Result<(), Box<dyn std::error::Error>> {
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
                    
                    // 移動後執行look
                    display_look(None, output_manager, game_world, me);
                    
                    // 如果小地圖已打開，更新小地圖資料
                    if output_manager.is_minimap_open() {
                        update_minimap_display(output_manager, game_world, me);
                    }
                } else {
                    output_manager.set_status("前方是牆壁，無法通過".to_string());
                }
            }
        } else {
            output_manager.set_status("超出地圖範圍".to_string());
        }
    }
    Ok(())
}

/// 處理 get 命令 - 撿起當前位置的物品
fn handle_get(
    item_name: Option<String>,
    quantity: u32,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) {
    let mut should_save_map = false;
    
    if let Some(current_map) = game_world.get_current_map_mut() {
        if let Some(point) = current_map.get_point_mut(me.x, me.y) {
            if point.objects.is_empty() {
                output_manager.print("此處沒有物品。".to_string());
                return;
            }
            
            match item_name {
                None => {
                    // 沒有指定物品名稱，撿起所有物品
                    let mut total_items = 0;
                    for (obj_name, count) in point.objects.clone() {
                        me.add_items(obj_name.clone(), count);
                        let display_name = item_registry::get_item_display_name(&obj_name);
                        output_manager.print(format!("✓ 撿起了: {} x{}", display_name, count));
                        total_items += count;
                    }
                    point.objects.clear();
                    output_manager.set_status(format!("撿起了 {} 個物品", total_items));
                    should_save_map = true;
                }
                Some(name) => {
                    // 解析物品名稱（支援英文和中文）
                    let resolved_name = item_registry::resolve_item_name(&name);
                    let available = point.get_object_count(&resolved_name);
                    
                    if available == 0 {
                        output_manager.print(format!("找不到 \"{}\"。", name));
                        return;
                    }
                    
                    // 取較小值：要求數量 vs 實際數量
                    let actual_quantity = quantity.min(available);
                    let removed = point.remove_objects(&resolved_name, actual_quantity);
                    
                    if removed > 0 {
                        me.add_items(resolved_name.clone(), removed);
                        let display_name = item_registry::get_item_display_name(&resolved_name);
                        output_manager.print(format!("✓ 撿起了: {} x{}", display_name, removed));
                        if removed < quantity {
                            output_manager.set_status(format!("只撿起了 {} 個 (要求 {})", removed, quantity));
                        } else {
                            output_manager.set_status(format!("撿起: {} x{}", display_name, removed));
                        }
                        should_save_map = true;
                    }
                }
            }
        }
    }
    
    // 保存角色物品和地圖
    if should_save_map {
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = me.save(&person_dir, "me");
        if let Some(current_map) = game_world.get_current_map() {
            let _ = game_world.save_map(current_map);
        }
    }
}

fn handle_drop(
    item_name: String,
    quantity: u32,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) {
    // 解析物品名稱（支援英文和中文）
    let resolved_name = item_registry::resolve_item_name(&item_name);
    let owned = me.get_item_count(&resolved_name);
    
    if owned == 0 {
        output_manager.print(format!("你沒有 \"{}\"。", item_name));
        return;
    }
    
    // 取較小值：要求數量 vs 持有數量
    let actual_quantity = quantity.min(owned);
    
    let mut should_save_map = false;
    
    if me.drop_items(&resolved_name, actual_quantity).is_some() {
        if let Some(current_map) = game_world.get_current_map_mut() {
            if let Some(point) = current_map.get_point_mut(me.x, me.y) {
                point.add_objects(resolved_name.clone(), actual_quantity);
                let display_name = item_registry::get_item_display_name(&resolved_name);
                output_manager.print(format!("✓ 放下了: {} x{}", display_name, actual_quantity));
                if actual_quantity < quantity {
                    output_manager.set_status(format!("只放下了 {} 個 (要求 {})", actual_quantity, quantity));
                } else {
                    output_manager.set_status(format!("放下: {} x{}", display_name, actual_quantity));
                }
                should_save_map = true;
            }
        }
    }
    
    // 保存角色物品和地圖
    if should_save_map {
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = me.save(&person_dir, "me");
        if let Some(current_map) = game_world.get_current_map() {
            let _ = game_world.save_map(current_map);
        }
    }
}

/// 處理召喚 NPC
fn handle_summon(
    npc_name: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &Person,
) {
    // 先檢查 NPC 是否存在並獲取名稱
    let npc_info = if let Some(npc) = game_world.npc_manager.get_npc(&npc_name) {
        Some((npc.name.clone(), npc.x, npc.y))
    } else {
        None
    };
    
    if let Some((name, old_x, old_y)) = npc_info {
        // 移動 NPC 到玩家位置
        if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
            npc.move_to(me.x, me.y);
        }
        
        // 保存 NPC 位置
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = game_world.npc_manager.save_all(&person_dir);
        
        output_manager.print(format!("你召喚了 {} 到這裡", name));
        output_manager.log(format!("{} 從 ({}, {}) 傳送到 ({}, {})", name, old_x, old_y, me.x, me.y));
    } else {
        output_manager.set_status(format!("找不到 NPC: {}", npc_name));
    }
}

/// 處理征服指令 - 使指定方向可行走
fn handle_conquer(
    direction: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 解析方向
    let (dx, dy, dir_name) = match direction.to_lowercase().as_str() {
        "up" | "u" => (0, -1, "上"),
        "down" | "d" => (0, 1, "下"),
        "left" | "l" => (-1, 0, "左"),
        "right" | "r" => (1, 0, "右"),
        _ => {
            output_manager.set_status(format!("未知方向: {}，請使用 up/down/left/right", direction));
            return Ok(());
        }
    };
    
    // 計算目標位置
    let target_x = (me.x as i32 + dx) as usize;
    let target_y = (me.y as i32 + dy) as usize;
    
    // 先獲取地圖名稱
    let map_name = game_world.current_map.clone();
    
    // 獲取當前地圖並修改
    if let Some(current_map) = game_world.get_current_map_mut() {
        // 檢查目標位置是否在地圖範圍內
        if target_x >= current_map.width || target_y >= current_map.height {
            output_manager.set_status("目標位置超出地圖範圍".to_string());
            return Ok(());
        }
        
        // 獲取目標點
        if let Some(point) = current_map.get_point_mut(target_x, target_y) {
            if point.walkable {
                output_manager.set_status(format!("{} 方已經是可行走的了", dir_name));
            } else {
                // 設置為可行走
                point.walkable = true;
                output_manager.print(format!("你征服了 {} 方的障礙！", dir_name));
                output_manager.print(format!("位置 ({}, {}) 現在可以行走了", target_x, target_y));
                output_manager.log(format!("玩家在 ({}, {}) 征服了 {} 方 ({}, {})", me.x, me.y, dir_name, target_x, target_y));
            }
        }
    }
    
    // 保存地圖 (使用地圖名稱)
    if let Some(map) = game_world.maps.get(&map_name) {
        game_world.save_map(map)?;
    }
    
    Ok(())
}

/// 處理飛到指令 - 傳送到指定位置/地圖/地點
fn handle_flyto(
    target: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 嘗試解析為坐標 (x,y)
    if let Some((x, y)) = parse_coordinates(&target) {
        // 檢查坐標是否在當前地圖範圍內
        if let Some(current_map) = game_world.get_current_map() {
            if x < current_map.width && y < current_map.height {
                me.move_to(x, y);
                output_manager.print(format!("你飛到了位置 ({}, {})", x, y));
                output_manager.log(format!("玩家傳送到 ({}, {})", x, y));
                
                // 保存玩家位置
                let person_dir = format!("{}/persons", game_world.world_dir);
                me.save(&person_dir, "me")?;
                
                // 自動執行 look
                display_look(None, output_manager, game_world, me);
                return Ok(());
            } else {
                output_manager.set_status("座標超出地圖範圍".to_string());
                return Ok(());
            }
        }
    }
    
    // 嘗試作為地圖名稱
    if game_world.maps.contains_key(&target) {
        game_world.current_map = target.clone();
        // 將玩家移動到地圖中心
        if let Some(new_map) = game_world.get_current_map() {
            let center_x = new_map.width / 2;
            let center_y = new_map.height / 2;
            me.move_to(center_x, center_y);
            output_manager.print(format!("你飛到了地圖「{}」", target));
            output_manager.log(format!("玩家傳送到地圖「{}」({}, {})", target, center_x, center_y));
            
            // 保存玩家位置和世界狀態
            let person_dir = format!("{}/persons", game_world.world_dir);
            me.save(&person_dir, "me")?;
            game_world.save_metadata()?;
            
            // 自動執行 look
            display_look(None, output_manager, game_world, me);
            return Ok(());
        }
    }
    
    // 嘗試作為地點名稱
    if let Some(current_map) = game_world.get_current_map() {
        for row in &current_map.points {
            for point in row {
                if !point.name.is_empty() && point.name == target {
                    me.move_to(point.x, point.y);
                    output_manager.print(format!("你飛到了地點「{}」({}, {})", target, point.x, point.y));
                    output_manager.log(format!("玩家傳送到地點「{}」({}, {})", target, point.x, point.y));
                    
                    // 保存玩家位置
                    let person_dir = format!("{}/persons", game_world.world_dir);
                    me.save(&person_dir, "me")?;
                    
                    // 自動執行 look
                    display_look(None, output_manager, game_world, me);
                    return Ok(());
                }
            }
        }
    }
    
    output_manager.set_status(format!("找不到目標: {}（請使用座標x,y、地圖名或地點名）", target));
    Ok(())
}

/// 處理 namehere 指令 - 命名當前地點
fn handle_namehere(
    name: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &Person,
) -> Result<(), Box<dyn std::error::Error>> {
    let map_name = game_world.current_map.clone();
    
    if let Some(current_map) = game_world.get_current_map_mut() {
        if let Some(point) = current_map.get_point_mut(me.x, me.y) {
            let old_name = if point.name.is_empty() {
                "（無名）".to_string()
            } else {
                point.name.clone()
            };
            
            point.name = name.clone();
            output_manager.print(format!("你將此地命名為「{}」", name));
            output_manager.log(format!("位置 ({}, {}) 從 {} 更名為「{}」", me.x, me.y, old_name, name));
        }
    }
    
    // 保存地圖
    if let Some(map) = game_world.maps.get(&map_name) {
        game_world.save_map(map)?;
    }
    
    Ok(())
}

/// 處理 name 指令 - 命名 NPC 或地點
fn handle_name(
    target: String,
    new_name: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 嘗試解析為坐標
    if let Some((x, y)) = parse_coordinates(&target) {
        let map_name = game_world.current_map.clone();
        
        if let Some(current_map) = game_world.get_current_map_mut() {
            if x < current_map.width && y < current_map.height {
                if let Some(point) = current_map.get_point_mut(x, y) {
                    let old_name = if point.name.is_empty() {
                        "（無名）".to_string()
                    } else {
                        point.name.clone()
                    };
                    
                    point.name = new_name.clone();
                    output_manager.print(format!("你將位置 ({}, {}) 命名為「{}」", x, y, new_name));
                    output_manager.log(format!("位置 ({}, {}) 從 {} 更名為「{}」", x, y, old_name, new_name));
                }
            } else {
                output_manager.set_status("座標超出地圖範圍".to_string());
                return Ok(());
            }
        }
        
        // 保存地圖
        if let Some(map) = game_world.maps.get(&map_name) {
            game_world.save_map(map)?;
        }
        
        return Ok(());
    }
    
    // 嘗試作為 NPC
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&target) {
        let old_name = npc.name.clone();
        npc.name = new_name.clone();
        output_manager.print(format!("你將「{}」改名為「{}」", old_name, new_name));
        output_manager.log(format!("NPC 從「{}」更名為「{}」", old_name, new_name));
        
        // 保存 NPC
        let person_dir = format!("{}/persons", game_world.world_dir);
        game_world.npc_manager.save_all(&person_dir)?;
        
        return Ok(());
    }
    
    output_manager.set_status(format!("找不到目標: {}（請使用座標x,y或NPC名稱）", target));
    Ok(())
}

/// 處理 destroy 指令 - 刪除當前位置的 NPC 或物品
fn handle_destroy(
    target: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 先嘗試作為 NPC（在當前位置）
    if let Some((id, npc)) = game_world.npc_manager.remove_npc_at(&target, me.x, me.y) {
        let npc_name = npc.name.clone();
        output_manager.print(format!("你摧毀了 NPC「{}」", npc_name));
        output_manager.log(format!("NPC「{}」在 ({}, {}) 被刪除", npc_name, me.x, me.y));
        
        // 保存 NPC 狀態
        let person_dir = format!("{}/persons", game_world.world_dir);
        game_world.npc_manager.save_all(&person_dir)?;
        
        return Ok(());
    }
    
    // 嘗試作為物品
    let item_name = item_registry::resolve_item_name(&target);
    let map_name = game_world.current_map.clone();
    
    if let Some(current_map) = game_world.get_current_map_mut() {
        if let Some(point) = current_map.get_point_mut(me.x, me.y) {
            if let Some(count) = point.objects.get(&item_name) {
                let count_value = *count;
                point.objects.remove(&item_name);
                
                let display_name = item_registry::get_item_display_name(&item_name);
                output_manager.print(format!("你摧毀了物品「{}」x{}", display_name, count_value));
                output_manager.log(format!("物品「{}」x{} 在 ({}, {}) 被刪除", display_name, count_value, me.x, me.y));
                
                // 保存地圖
                if let Some(map) = game_world.maps.get(&map_name) {
                    game_world.save_map(map)?;
                }
                
                return Ok(());
            }
        }
    }
    
    output_manager.set_status(format!("此處找不到「{}」（NPC 或物品）", target));
    Ok(())
}

/// 解析坐標字串 "x,y"
fn parse_coordinates(s: &str) -> Option<(usize, usize)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() == 2 {
        if let (Ok(x), Ok(y)) = (parts[0].trim().parse::<usize>(), parts[1].trim().parse::<usize>()) {
            return Some((x, y));
        }
    }
    None
}

/// 檢查並執行事件
fn check_and_execute_events(
    game_world: &mut GameWorld,
    me: &mut Person,
    output_manager: &mut OutputManager,
) {
    let current_day = game_world.time.day;
    let current_hour = game_world.time.hour;
    let current_minute = game_world.time.minute;
    
    // 如果是同一分鐘，不重複檢查
    if (current_day, current_hour, current_minute) == game_world.event_scheduler.last_check_time {
        return;
    }
    
    // 調試訊息
    output_manager.log(format!("🔍 [DEBUG] 檢查事件 Day {} {:02}:{:02}", current_day, current_hour, current_minute));
    
    game_world.event_scheduler.last_check_time = (current_day, current_hour, current_minute);
    
    let events: Vec<crate::event::GameEvent> = game_world.event_manager.list_events()
        .iter()
        .map(|e| (*e).clone())
        .collect();
    
    output_manager.log(format!("🔍 [DEBUG] 共 {} 個事件", events.len()));
    
    let mut triggered_event_ids = Vec::new();
    
    for event in events {
        let event_id = event.id.clone();
        
        if let Some(runtime_state) = game_world.event_manager.get_runtime_state(&event_id) {
            if !event.can_trigger(runtime_state) {
                output_manager.log(format!("🔍 [DEBUG] {} - 冷卻中", event.name));
                continue;
            }
        }
        
        let trigger_check = crate::event_scheduler::EventScheduler::new()
            .check_trigger(&event, game_world);
        let condition_check = crate::event_scheduler::EventScheduler::new()
            .check_conditions(&event, game_world, me);
        
        output_manager.log(format!("🔍 [DEBUG] {} - trigger: {}, condition: {}", 
            event.name, trigger_check, condition_check));
        
        if trigger_check && condition_check {
            triggered_event_ids.push(event_id.clone());
            game_world.event_manager.trigger_event(&event_id);
        }
    }
    
    output_manager.log(format!("🔍 [DEBUG] 觸發 {} 個事件", triggered_event_ids.len()));
    
    for event_id in triggered_event_ids {
        if let Some(event) = game_world.event_manager.get_event(&event_id) {
            let event_clone = event.clone();
            let location_info = get_event_location_info(&event_clone, game_world);
            output_manager.log(format!("🎭 事件: {}{}", event_clone.name, location_info));
            
            if let Err(e) = crate::event_executor::EventExecutor::execute_event(
                &event_clone,
                game_world,
                me,
                output_manager
            ) {
                output_manager.log(format!("⚠️  事件執行錯誤: {}", e));
            }
        }
    }
}

/// 獲取事件位置信息字符串
fn get_event_location_info(event: &crate::event::GameEvent, game_world: &GameWorld) -> String {
    if let Some(map_name) = &event.r#where.map {
        if let Some(positions) = &event.r#where.positions {
            if !positions.is_empty() {
                if let Some(map) = game_world.maps.get(map_name) {
                    if let Some(point) = map.get_point(positions[0][0], positions[0][1]) {
                        return format!(" 在 {}({}, {}) - {}", 
                            map_name, positions[0][0], positions[0][1], point.description);
                    }
                }
                return format!(" 在 {}({}, {})", map_name, positions[0][0], positions[0][1]);
            }
        } else if let Some(area) = &event.r#where.area {
            return format!(" 在 {} 區域({}-{}, {}-{})", 
                map_name, area.x[0], area.x[1], area.y[0], area.y[1]);
        }
        return format!(" 在 {}", map_name);
    }
    String::new()
}
