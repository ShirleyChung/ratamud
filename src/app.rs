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
    let event_check_interval = Duration::from_millis(500);  // 每0.5秒檢查事件
    
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
            
            // 側邊面板使用相同的位置和大小
            let floating_area = Rect {
                x: minimap_x,
                y: minimap_y,
                width: minimap_width,
                height: minimap_height + 10,  // 側邊面板稍大一些
            };
            // 畫側邊面板
            if output_manager.is_side_panel_open() {
                let side_widget = output_manager.get_side_panel(floating_area);
                f.render_widget(Clear, floating_area); // 清除背景
                f.render_widget(side_widget, floating_area);
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
                    // 上下左右鍵優先用於移動
                    KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                        // 將方向鍵傳遞給 input_handler 處理移動
                        if let Some(result) = input_handler.handle_event(
                            crossterm::event::Event::Key(key)
                        ) {
                            if let CommandResult::Exit = result {
                                should_exit = true;
                            } else {
                                handle_command_result(result, output_manager, game_world, me)?;
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
        CommandResult::Look => display_look(output_manager, game_world, me),
        CommandResult::Move(dx, dy) => handle_movement(dx, dy, output_manager, game_world, me)?,
        CommandResult::Get(item_name) => handle_get(item_name, output_manager, game_world, me),
        CommandResult::Drop(item_name) => handle_drop(item_name, output_manager, game_world, me),
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

/// 處理輸出結果
fn handle_output(text: String, output_manager: &mut OutputManager) {
    output_manager.print(text);
    // 只關閉 minimap，不關閉側邊面板（側邊面板應該保持開啟直到使用者明確關閉）
    if output_manager.is_minimap_open() {
        output_manager.hide_minimap();
    }
}

/// 處理錯誤訊息
fn handle_error(err: String, output_manager: &mut OutputManager) {
    output_manager.set_status(err);
    // 只關閉 minimap，不關閉側邊面板
    if output_manager.is_minimap_open() {
        output_manager.hide_minimap();
    }
}

/// 處理清除訊息
fn handle_clear(output_manager: &mut OutputManager) {
    output_manager.clear_messages();
    output_manager.set_status("Text area cleared".to_string());
    // 只關閉 minimap，不關閉側邊面板
    if output_manager.is_minimap_open() {
        output_manager.hide_minimap();
    }
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

/// 處理關閉狀態面板
#[allow(dead_code)]
fn handle_close_status(output_manager: &mut OutputManager) {
    if output_manager.is_side_panel_open() {
        output_manager.toggle_side_panel();
    }
}

/// 顯示 look 命令的結果
fn display_look(
    output_manager: &mut OutputManager,
    game_world: &GameWorld,
    me: &Person,
) {
    if let Some(current_map) = game_world.get_current_map() {
        // 顯示當前位置信息
        if let Some(point) = current_map.get_point(me.x, me.y) {
            output_manager.print( format!("【當前位置: ({}, {})】\n【{}】", me.x, me.y, point.description) );
            
            // 顯示當前位置的 items
            if !point.objects.is_empty() {
                output_manager.print(format!("\n🎁 此處物品:"));
                for obj in &point.objects {
                    output_manager.print(format!("  • {}", obj));
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
                    display_look(output_manager, game_world, me);
                    
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
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) {
    if let Some(current_map) = game_world.get_current_map_mut() {
        if let Some(point) = current_map.get_point_mut(me.x, me.y) {
            if point.objects.is_empty() {
                output_manager.print("此處沒有物品。".to_string());
                return;
            }
            
            match item_name {
                None => {
                    // 沒有指定物品名稱，撿起所有物品
                    let items_count = point.objects.len();
                    for obj in point.objects.drain(..) {
                        me.add_item(obj.clone());
                        output_manager.print(format!("✓ 撿起了: {}", obj));
                    }
                    output_manager.set_status(format!("撿起了 {} 個物品", items_count));
                    
                    // 保存角色物品
                    let person_dir = format!("{}/persons", game_world.world_dir);
                    let _ = me.save(&person_dir, "me");
                }
                Some(name) => {
                    // 尋找指定名稱的物品
                    if let Some(pos) = point.objects.iter().position(|obj| obj.contains(&name)) {
                        let item = point.objects.remove(pos);
                        me.add_item(item.clone());
                        output_manager.print(format!("✓ 撿起了: {}", item));
                        output_manager.set_status(format!("撿起: {}", name));
                        
                        // 保存角色物品
                        let person_dir = format!("{}/persons", game_world.world_dir);
                        let _ = me.save(&person_dir, "me");
                    } else {
                        output_manager.print(format!("找不到 \"{}\" 的物品。", name));
                    }
                }
            }
        }
    }
}

fn handle_drop(
    item_name: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) {
    if let Some(item) = me.drop_item(&item_name) {
        if let Some(current_map) = game_world.get_current_map_mut() {
            if let Some(point) = current_map.get_point_mut(me.x, me.y) {
                point.objects.push(item.clone());
                output_manager.print(format!("✓ 放下了: {}", item));
                output_manager.set_status(format!("放下: {}", item_name));
                
                // 保存角色物品
                let person_dir = format!("{}/persons", game_world.world_dir);
                let _ = me.save(&person_dir, "me");
            }
        }
    } else {
        output_manager.print(format!("身上沒有 \"{}\" 的物品。", item_name));
    }
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
    
    if (current_day, current_hour, current_minute) == game_world.event_scheduler.last_check_time {
        return;
    }
    
    game_world.event_scheduler.last_check_time = (current_day, current_hour, current_minute);
    
    let events: Vec<crate::event::GameEvent> = game_world.event_manager.list_events()
        .iter()
        .map(|e| (*e).clone())
        .collect();
    
    let mut triggered_event_ids = Vec::new();
    
    for event in events {
        let event_id = event.id.clone();
        
        if let Some(runtime_state) = game_world.event_manager.get_runtime_state(&event_id) {
            if !event.can_trigger(runtime_state) {
                continue;
            }
        }
        
        let should_trigger = crate::event_scheduler::EventScheduler::new()
            .check_trigger(&event, game_world) &&
            crate::event_scheduler::EventScheduler::new()
            .check_conditions(&event, game_world, me);
        
        if should_trigger {
            triggered_event_ids.push(event_id.clone());
            game_world.event_manager.trigger_event(&event_id);
        }
    }
    
    for event_id in triggered_event_ids {
        if let Some(event) = game_world.event_manager.get_event(&event_id) {
            let event_clone = event.clone();
            let location_info = get_event_location_info(&event_clone, game_world);
            output_manager.print(format!("🎭 事件: {}{}", event_clone.name, location_info));
            
            if let Err(e) = crate::event_executor::EventExecutor::execute_event(
                &event_clone,
                game_world,
                me,
                output_manager
            ) {
                output_manager.print(format!("⚠️  事件執行錯誤: {}", e));
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
