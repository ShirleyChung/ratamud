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
use crate::ui::InputDisplay;

/// 應用程式主迴圈 - 將 main.rs 中的事件迴圈邏輯提取到此
pub fn run_main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    input_handler: &mut InputHandler,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut should_exit = false;
    // 時間管理：記錄上次更新時間和上次顯示時間
    let mut last_time_update = Instant::now();
    let mut last_time_display = Instant::now();
    let time_update_interval = Duration::from_millis(1000);  // 每1秒更新世界時間（=60遊戲秒）
    let time_display_interval = Duration::from_secs(5);  // 每5秒顯示時間（=5遊戲分鐘）
    
    'main_loop: loop {
        // 更新狀態列（檢查訊息是否過期）
        output_manager.update_status();
        
        // 更新世界時間（每秒更新，1實際秒 = 60遊戲秒）
        let now = Instant::now();
        if now.duration_since(last_time_update) >= time_update_interval {
            game_world.update_time();
            last_time_update = now;
            
            // 檢查並觸發事件
            check_and_execute_events(game_world, me, output_manager);
            
            // 每5秒顯示一次時間到狀態列（=5遊戲分鐘）
            if now.duration_since(last_time_display) >= time_display_interval {
                output_manager.set_current_time(game_world.format_time());
                last_time_display = now;
            }
        }
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

            // 計算懸浮視窗的位置和大小（右上角，寬度 40%，高度 60%）
            let floating_width = (size.width as f32 * 0.4) as u16;
            let floating_height = (size.height as f32 * 0.6) as u16;
            let floating_x = size.width.saturating_sub(floating_width + 2);
            let floating_y = 1;
            
            let minimap_area = Rect {
                x: floating_x,
                y: floating_y,
                width: floating_width,
                height: floating_height,
            };
            // 畫小地圖
            if output_manager.is_minimap_open() {
                let minimap_widget = output_manager.get_minimap(minimap_area);
                f.render_widget(Clear, minimap_area); // 清除背景
                f.render_widget(minimap_widget, minimap_area);
            }
            let floating_area = Rect {
                x: floating_x,
                y: floating_y,
                width: floating_width,
                height: floating_height,
            };
            // 畫側邊面板
            if output_manager.is_side_panel_open() {
                let side_widget = output_manager.get_side_panel(floating_area);
                f.render_widget(Clear, floating_area); // 清除背景
                f.render_widget(side_widget, floating_area);
            }

            // 渲染輸入區域
            let input_widget = InputDisplay::render_input(input_handler.get_input(), vertical_chunks[1]);
            f.render_widget(input_widget, vertical_chunks[1]);

            // 渲染狀態列
            let status_widget = output_manager.render_status();
            f.render_widget(status_widget, vertical_chunks[2]);
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
            
            // 上方
            if me.y > 0 {
                if let Some(p) = current_map.get_point(me.x, me.y - 1) {
                    output_manager.print(format!("↑ 北方: {}", p.description));
                }
            } else {
                output_manager.print("(邊界)".to_string());
            }
            
            // 下方
            if me.y + 1 < current_map.height {
                if let Some(p) = current_map.get_point(me.x, me.y + 1) {
                    output_manager.print(format!("↓ 南方: {}", p.description));
                }
            } else {
                output_manager.print("(邊界)".to_string());
            }
            
            // 左方
            if me.x > 0 {
                if let Some(p) = current_map.get_point(me.x - 1, me.y) {
                    output_manager.print(format!("← 西方: {}", p.description));
                }
            } else {
                output_manager.print("(邊界)".to_string());
            }
            
            // 右方
            if me.x + 1 < current_map.width {
                if let Some(p) = current_map.get_point(me.x + 1, me.y) {
                    output_manager.print(format!("→ 東方: {}", p.description));
                }
            } else {
                output_manager.print("(邊界)".to_string());
            }            
        }
    }
}

/// 更新小地圖顯示
fn update_minimap_display(
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
    // 創建一個臨時的 GameWorld 視圖用於檢查
    let current_day = game_world.time.day;
    let current_hour = game_world.time.hour;
    let current_minute = game_world.time.minute;
    let current_second = game_world.time.second;
    let current_map = game_world.current_map.clone();
    
    // 先收集觸發的事件ID
    let triggered_events = {
        // 使用內部作用域來限制借用
        let scheduler = &mut game_world.event_scheduler;
        let manager = &mut game_world.event_manager;
        
        // 檢查時間是否變化
        let should_check = {
            let last_check = scheduler.last_check_time;
            (current_day, current_hour, current_minute) != last_check
        };
        
        if !should_check {
            return;
        }
        
        scheduler.last_check_time = (current_day, current_hour, current_minute);
        
        // 收集所有觸發的事件
        let events: Vec<crate::event::GameEvent> = manager.list_events()
            .iter()
            .map(|e| (*e).clone())
            .collect();
        
        let mut triggered = Vec::new();
        
        for event in events {
            let event_id = event.id.clone();
            
            // 檢查運行時狀態
            if let Some(runtime_state) = manager.get_runtime_state(&event_id) {
                if !event.can_trigger(runtime_state) {
                    continue;
                }
            }
            
            // 檢查觸發條件
            if check_event_trigger(&event, current_minute, current_hour, current_day, current_second) {
                // 檢查條件（人事時地物）
                if check_event_conditions(&event, &current_map, me) {
                    triggered.push(event.clone());
                    manager.trigger_event(&event_id);
                }
            }
        }
        
        triggered
    };
    
    // 執行觸發的事件
    for event in triggered_events {
        // 顯示事件觸發訊息（帶位置信息）
        let location_info = get_event_location_info(&event, game_world);
        output_manager.print(format!("🎭 事件: {}{}", event.name, location_info));
        
        // 執行事件動作
        if let Err(e) = crate::event_executor::EventExecutor::execute_event(
            &event,
            game_world,
            me,
            output_manager
        ) {
            output_manager.print(format!("⚠️  事件執行錯誤: {}", e));
        }
    }
}

/// 獲取事件位置信息字符串
fn get_event_location_info(event: &crate::event::GameEvent, game_world: &GameWorld) -> String {
    if let Some(map_name) = &event.r#where.map {
        if let Some(positions) = &event.r#where.positions {
            if !positions.is_empty() {
                // 獲取該位置的描述
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

/// 檢查事件觸發條件
fn check_event_trigger(
    event: &crate::event::GameEvent,
    minute: u8,
    hour: u8,
    day: u32,
    _second: u8,
) -> bool {
    use crate::event::TriggerType;
    
    match &event.trigger {
        TriggerType::Time { schedule, random_chance, day_range, time_range } => {
            // 檢查 crontab 時間表達式
            if !crate::event_scheduler::CronParser::matches(schedule, minute, hour, day) {
                return false;
            }
            
            // 檢查天數範圍
            if let Some([start_day, end_day]) = day_range {
                if day < *start_day || day > *end_day {
                    return false;
                }
            }
            
            // 檢查時間範圍
            if let Some([start_time, end_time]) = time_range {
                let current_time = format!("{:02}:{:02}:{:02}", hour, minute, _second);
                if current_time < *start_time || current_time > *end_time {
                    return false;
                }
            }
            
            // 檢查隨機機率
            if let Some(chance) = random_chance {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                if rng.gen::<f32>() > *chance {
                    return false;
                }
            }
            
            true
        }
        TriggerType::Random { chance, .. } => {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            rng.gen::<f32>() <= *chance
        }
        _ => false,
    }
}

/// 檢查事件條件
fn check_event_conditions(
    event: &crate::event::GameEvent,
    current_map: &str,
    player: &Person,
) -> bool {
    // 檢查地點條件
    if let Some(map_name) = &event.r#where.map {
        if *current_map != *map_name {
            return false;
        }
    }
    
    if let Some(positions) = &event.r#where.positions {
        let player_pos = (player.x, player.y);
        let mut found = false;
        for pos in positions {
            if pos[0] == player_pos.0 && pos[1] == player_pos.1 {
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
    }
    
    if let Some(area) = &event.r#where.area {
        let in_x_range = player.x >= area.x[0] && player.x <= area.x[1];
        let in_y_range = player.y >= area.y[0] && player.y <= area.y[1];
        if !in_x_range || !in_y_range {
            return false;
        }
    }
    
    // 檢查物品條件
    if let Some(required_items) = &event.what.required_items {
        for item in required_items {
            if !player.items.contains(item) {
                return false;
            }
        }
    }
    
    true
}
