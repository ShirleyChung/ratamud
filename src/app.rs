use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use ratatui::layout::{Layout, Constraint, Direction, Rect};
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
    let time_update_interval = Duration::from_millis(1000);  // 每1秒更新時間
    let mut cnt_time = 0;
    'main_loop: loop {
        // 更新狀態列（檢查訊息是否過期）
        output_manager.update_status();
        
        // 更新世界時間
        let now = Instant::now();
        if now.duration_since(last_time_update) >= time_update_interval {
            game_world.update_time();
            cnt_time += 1;
            // 每次更新都設置當前時間顯示
            output_manager.set_current_time(game_world.format_time());
            if (cnt_time % 60) == 0 {
                // 每60秒顯示一次時間
                output_manager.set_status(format!("時間: {}", game_world.format_time()));     
            }
            last_time_update = now;
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
    output_manager.add_message(text);
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
            let mut look_output = format!("【當前位置: ({}, {})】\n【{}】\n\n", me.x, me.y, point.description);
            
            // 上方
            look_output.push_str("↑ 北方: ");
            if me.y > 0 {
                if let Some(p) = current_map.get_point(me.x, me.y - 1) {
                    look_output.push_str(&format!("{}\n", p.description));
                }
            } else {
                look_output.push_str("(邊界)\n");
            }
            
            // 下方
            look_output.push_str("↓ 南方: ");
            if me.y + 1 < current_map.height {
                if let Some(p) = current_map.get_point(me.x, me.y + 1) {
                    look_output.push_str(&format!("{}\n", p.description));
                }
            } else {
                look_output.push_str("(邊界)\n");
            }
            
            // 左方
            look_output.push_str("← 西方: ");
            if me.x > 0 {
                if let Some(p) = current_map.get_point(me.x - 1, me.y) {
                    look_output.push_str(&format!("{}\n", p.description));
                }
            } else {
                look_output.push_str("(邊界)\n");
            }
            
            // 右方
            look_output.push_str("→ 東方: ");
            if me.x + 1 < current_map.width {
                if let Some(p) = current_map.get_point(me.x + 1, me.y) {
                    look_output.push_str(&format!("{}\n", p.description));
                }
            } else {
                look_output.push_str("(邊界)\n");
            }
            
            output_manager.add_message(look_output);
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
                minimap_data.push(format!("↑ {}", point.description));
            }
        } else {
            minimap_data.push("↑ (邊界)".to_string());
        }
        
        // 下方
        if me.y + 1 < current_map.height {
            if let Some(point) = current_map.get_point(me.x, me.y + 1) {
                minimap_data.push(format!("↓ {}", point.description));
            }
        } else {
            minimap_data.push("↓ (邊界)".to_string());
        }
        
        // 左方
        if me.x > 0 {
            if let Some(point) = current_map.get_point(me.x - 1, me.y) {
                minimap_data.push(format!("← {}", point.description));
            }
        } else {
            minimap_data.push("← (邊界)".to_string());
        }
        
        // 右方
        if me.x + 1 < current_map.width {
            if let Some(point) = current_map.get_point(me.x + 1, me.y) {
                minimap_data.push(format!("→ {}", point.description));
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
                    
                    // 移動後顯示當前位置的描述
                    if let Some(point) = current_map.get_point(me.x, me.y) {
                        output_manager.add_message(format!("【{}】", point.description));
                    }
                    
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
