use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use ratatui::layout::{Layout, Constraint, Direction, Rect};
use ratatui::widgets::Clear;
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Style};
use std::io;
use std::time::{Duration, Instant};
use std::sync::mpsc;
use std::thread;

use crate::input::InputHandler;
use crate::output::OutputManager;
use crate::world::GameWorld;
use crate::settings::GameSettings;
use crate::person::Person;
use crate::observable::WorldInfo;
use crate::input::CommandResult;
use crate::quest::{QuestReward};
use crate::item_registry;
use crate::ui::{InputDisplay, HeaderDisplay, Menu};


/// A context struct to hold all the application state references.
/// This helps to avoid passing too many arguments to functions.
pub struct AppContext<'a> {
    pub menu: &'a mut Option<Menu>,
    pub interaction_menu: &'a mut Option<Menu>,  // 新增：互動專用選單（交易、對話等）
    pub should_exit: &'a mut bool,
    pub output_manager: &'a mut OutputManager,
    pub game_world: &'a mut GameWorld,
}



/// 確保 Rect 在邊界內
fn clamp_rect(rect: Rect, max_width: u16, max_height: u16) -> Rect {
    let x = rect.x.min(max_width.saturating_sub(1));
    let y = rect.y.min(max_height.saturating_sub(1));
    let width = rect.width.min(max_width.saturating_sub(x));
    let height = rect.height.min(max_height.saturating_sub(y));
    
    Rect { x, y, width, height }
}

/// 創建 NPC AI 執行緒（新架構：使用 channel）
fn create_npc_thread(
    npc_view_rx: mpsc::Receiver<std::collections::HashMap<String, crate::npc_view::NpcView>>,
    npc_event_tx: mpsc::Sender<crate::game_event::GameEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Ok(npc_views) = npc_view_rx.recv() {
            // 為每個 NPC 決定行為
            for (npc_id, view) in npc_views {
                if let Some(action) = crate::npc_ai::NpcAiController::decide_action(&view) {
                    // 發送行為事件回主執行緒
                    let event = crate::game_event::GameEvent::NpcActions {
                        npc_id,
                        actions: vec![action],
                    };
                    
                    if npc_event_tx.send(event).is_err() {
                        // 主執行緒已關閉，退出
                        return;
                    }
                }
            }
            
            // 休眠 5 秒再處理下一批
            thread::sleep(Duration::from_secs(5));
        }
    })
}

/// 應用程式主迴圈 - 將 main.rs 中的事件迴圈邏輯提取到此
pub fn run_main_loop(
    mut terminal: Terminal<CrosstermBackend<io::Stdout>>,
    mut input_handler: InputHandler,
    mut output_manager: OutputManager,
    mut game_world: GameWorld,
    mut me: Person,
    rx: mpsc::Receiver<crossterm::event::KeyEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 Menu 狀態
    let mut menu: Option<Menu> = None;
    let mut interaction_menu: Option<Menu> = None;  // 互動選單（交易、對話等）
    let mut should_exit = false;
    let mut last_event_check = Instant::now();
    let event_check_interval = Duration::from_millis(100);  // 每0.1秒檢查事件
    
    // === 新架構：創建 channel ===
    let (npc_view_tx, npc_view_rx) = mpsc::channel();
    let (npc_event_tx, npc_event_rx) = mpsc::channel();
    
    // 啟動 NPC AI 執行緒（新架構）
    let _npc_thread_handle = create_npc_thread(npc_view_rx, npc_event_tx);
    
    'main_loop: loop {
        // --- 1. 處理 NPC AI 事件 ---
        while let Ok(event) = npc_event_rx.try_recv() {
            let messages = game_world.apply_event(event);
            for msg in messages {
                if msg.is_log() {
                    output_manager.log(msg.to_display_text());
                } else {
                    output_manager.print(msg.to_display_text());
                }
            }
        }
        
        // --- 1.5 檢測距離變化（靠近/離開通知）---
        check_and_handle_proximity(&mut output_manager, &mut game_world, &me, false);
        
        // --- 2. Input Handling ---
        // Process all pending input events from the channel non-blockingly
        for key in rx.try_iter() {
            let mut context = AppContext {
                menu: &mut menu,
                interaction_menu: &mut interaction_menu,
                should_exit: &mut should_exit,
                output_manager: &mut output_manager,
                game_world: &mut game_world,
            };
            // Call the new method from input_handler
            if let Some(command_result) = input_handler.handle_input_events(key, &mut context) {
                // Now, handle the CommandResult here in app.rs
                if let CommandResult::Exit = command_result {
                    handle_command_result(command_result, &mut output_manager, &mut game_world, &mut me, &mut interaction_menu)?;
                    should_exit = true; // Set should_exit to trigger loop exit
                } else {
                    handle_command_result(command_result, &mut output_manager, &mut game_world, &mut me, &mut interaction_menu)?;
                }
            }
        }
        
        // --- 3. Game State Update ---
        
        if output_manager.is_minimap_open() {
            update_minimap_display(&mut output_manager, &game_world, &me);
        }
        
        output_manager.update_status();
        output_manager.update_typewriter();
        game_world.update_time();
        
        use crate::time_updatable::TimeUpdatable;
        let time_info = game_world.get_time_info();
        me.on_time_update(&time_info);
        
        let now = Instant::now();
        if now.duration_since(last_event_check) >= event_check_interval {
            check_and_execute_events(&mut game_world, &mut me, &mut output_manager);
            last_event_check = now;
        }
        
        // --- 4. 發送 NPC Views 到 AI 執行緒 ---
        let npc_views = game_world.build_npc_views();
        let _ = npc_view_tx.send(npc_views); // 忽略錯誤（AI 執行緒可能已關閉）
        
        // --- 5. Drawing ---
        
        terminal.draw(|f| {
            draw_ui(f, &mut output_manager, &game_world, &input_handler, &me, &menu, &interaction_menu);
        })?;

        if should_exit {
            break 'main_loop;
        }

        thread::sleep(Duration::from_millis(16));
    }

    // --- Shutdown ---
    game_world.save_metadata()?;
    game_world.save_time()?;
    
    // 保存所有 NPC 的狀態
    let person_dir = format!("{}/persons", game_world.world_dir);
    game_world.npc_manager.save_all(&person_dir)?;
    
    let game_settings = GameSettings {
        show_minimap: output_manager.is_minimap_open(),
        show_log: output_manager.is_log_open(),
    };
    let _ = game_settings.save();

    Ok(())
}

/// Helper function to draw the entire UI
fn draw_ui(
    f: &mut ratatui::Frame,
    output_manager: &mut OutputManager,
    game_world: &GameWorld,
    input_handler: &InputHandler,
    me: &Person,
    menu: &Option<Menu>,
    interaction_menu: &Option<Menu>,  // 新增：互動選單
) {
    let size = f.size();
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(size);

    let current_time_str = game_world.format_time();
    let header_widget = HeaderDisplay::render_header("beginWorld", &current_time_str);
    f.render_widget(header_widget, vertical_chunks[0]);

    let output_widget = output_manager.render_output(vertical_chunks[1]);
    f.render_widget(output_widget, vertical_chunks[1]);

    let minimap_width = 42u16;
    let minimap_height = 18u16;
    let minimap_x = size.width.saturating_sub(minimap_width);
    let minimap_y = 1;
    let minimap_area = Rect { x: minimap_x, y: minimap_y, width: minimap_width, height: minimap_height };

    if output_manager.is_minimap_open() {
        let minimap_widget = output_manager.get_minimap(minimap_area);
        let safe_area = clamp_rect(minimap_area, size.width, size.height);
        f.render_widget(Clear, safe_area);
        f.render_widget(minimap_widget, safe_area);
    }

    let log_width = minimap_width;
    let log_height = (size.height as f32 * 0.45) as u16;
    let log_x = size.width.saturating_sub(log_width);
    let log_y = minimap_y + minimap_height + 1;
    let log_area = Rect { x: log_x, y: log_y, width: log_width, height: log_height };

    if output_manager.is_log_open() {
        let log_widget = output_manager.render_log(log_area);
        let safe_area = clamp_rect(log_area, size.width, size.height);
        f.render_widget(Clear, safe_area);
        f.render_widget(log_widget, safe_area);
    }

    let side_panel_height = if output_manager.is_status_panel_open() {
        let content_height = output_manager.get_side_panel_content_height();
        let max_height = size.height.saturating_sub(vertical_chunks[2].height + vertical_chunks[3].height + 2);
        content_height.min(max_height)
    } else {
        minimap_height
    };
    let floating_area = Rect { x: minimap_x, y: minimap_y, width: minimap_width, height: side_panel_height };

    if output_manager.is_status_panel_open() {
        let side_widget = output_manager.get_side_panel(floating_area);
        let safe_area = clamp_rect(floating_area, size.width, size.height);
        f.render_widget(Clear, safe_area);
        f.render_widget(side_widget, safe_area);
    }

    if output_manager.is_map_open() {
        if let Some(current_map) = game_world.get_current_map() {
            let map_width = (size.width as f32 * 0.8) as u16;
            let map_height = (size.height as f32 * 0.8) as u16;
            let map_x = (size.width.saturating_sub(map_width)) / 2;
            let map_y = (size.height.saturating_sub(map_height)) / 2;
            let map_area = Rect { x: map_x, y: map_y, width: map_width, height: map_height };
            let map_widget = output_manager.render_big_map(map_area, current_map, me.x, me.y, &game_world.npc_manager, &game_world.current_map_name);
            let safe_area = clamp_rect(map_area, size.width, size.height);
            f.render_widget(Clear, safe_area);
            f.render_widget(map_widget, safe_area);
        }
    }

    let input_widget = InputDisplay::render_input(input_handler.get_input(), vertical_chunks[2]);
    f.render_widget(input_widget, vertical_chunks[2]);

    let status_widget = output_manager.render_status();
    f.render_widget(status_widget, vertical_chunks[3]);

    if let Some(active_menu) = menu {
        if active_menu.active {
            let menu_width = (size.width as f32 * 0.4) as u16;
            let menu_height = (active_menu.items.len() as u16 + 2).min((size.height as f32 * 0.8) as u16);
            let menu_x = (size.width.saturating_sub(menu_width)) / 2;
            let menu_y = (size.height.saturating_sub(menu_height)) / 2;
            let menu_area = Rect { x: menu_x, y: menu_y, width: menu_width, height: menu_height };
            let safe_menu_area = clamp_rect(menu_area, size.width, size.height);
            f.render_widget(Clear, safe_menu_area);
            f.render_widget(active_menu.render_widget(), safe_menu_area);
        }
    }
    
    // 渲染互動選單（覆蓋在一般選單之上）
    if let Some(active_interaction_menu) = interaction_menu {
        if active_interaction_menu.active {
            let menu_width = (size.width as f32 * 0.5) as u16;
            let menu_height = (active_interaction_menu.items.len() as u16 + 2).min((size.height as f32 * 0.8) as u16);
            let menu_x = (size.width.saturating_sub(menu_width)) / 2;
            let menu_y = (size.height.saturating_sub(menu_height)) / 2;
            let menu_area = Rect { x: menu_x, y: menu_y, width: menu_width, height: menu_height };
            let safe_menu_area = clamp_rect(menu_area, size.width, size.height);
            f.render_widget(Clear, safe_menu_area);
            f.render_widget(active_interaction_menu.render_widget(), safe_menu_area);
        }
    }
}


/// 處理命令結果 - 主分派函式
fn handle_command_result(
    result: CommandResult,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
    interaction_menu: &mut Option<Menu>,  // 新增：互動選單
) -> Result<(), Box<dyn std::error::Error>> {
    output_manager.close_status_panel();
    
    // 檢查是否在睡眠狀態，如果是則只允許 dream 和 wakeup 命令
    if me.is_sleeping {
        match result {
            CommandResult::Dream(content) => handle_dream(content, output_manager),
            CommandResult::WakeUp => handle_wakeup(output_manager, me),
            _ => {
                output_manager.print("你正在睡覺，只能使用 dream 或 wakeup 指令！".to_string());
            }
        }
        return Ok(())
    }

    me.check_mp(-1); // 每執行一個命令消耗 1 MP
    
    match result {
        CommandResult::Exit => handle_exit(output_manager, game_world, me)?,
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
        CommandResult::Eat(food_name) => handle_eat(food_name, output_manager, me),
        CommandResult::UseItem(item_name) => handle_use_item(item_name, output_manager, me),
        CommandResult::UseItemOn(item_name, target_name) => handle_use_item_on(item_name, target_name, output_manager, game_world, me),
        CommandResult::Sleep => handle_sleep(output_manager, me),
        CommandResult::Dream(_) => {
            output_manager.print("你需要先睡覺才能做夢！使用 sleep 指令進入睡眠。".to_string());
        },
        CommandResult::WakeUp => {
            output_manager.print("你還沒睡覺呢！".to_string());
        },
        CommandResult::Summon(npc_name) => handle_summon(npc_name, output_manager, game_world, me),
        CommandResult::Conquer(direction) => handle_conquer(direction, output_manager, game_world, me)?,
        CommandResult::FlyTo(target) => handle_flyto(target, output_manager, game_world, me)?,
        CommandResult::NameHere(name) => handle_namehere(name, output_manager, game_world, me)?,
        CommandResult::Name(target, name) => handle_name(target, name, output_manager, game_world, me)?,
        CommandResult::Destroy(target) => handle_destroy(target, output_manager, game_world, me)?,
        CommandResult::Create(obj_type, item_type, name) => handle_create(obj_type, item_type, name, output_manager, game_world, me)?,
        CommandResult::Set(target, attribute, value) => handle_set(target, attribute, value, output_manager, game_world, me)?,
        CommandResult::SwitchControl(npc_name) => handle_switch_control(npc_name, output_manager, game_world, me)?,
        CommandResult::Trade(npc_name) => handle_trade(npc_name, output_manager, game_world, me, interaction_menu)?,
        CommandResult::Buy(npc_name, item, quantity) => handle_buy(npc_name, item, quantity, output_manager, game_world, me)?,
        CommandResult::Sell(npc_name, item, quantity) => handle_sell(npc_name, item, quantity, output_manager, game_world, me)?,
        CommandResult::Give(npc_name, item, quantity) => handle_give(npc_name, item, quantity, output_manager, game_world, me)?,
        CommandResult::SetDialogue(npc_name, topic, dialogue) => handle_set_dialogue(npc_name, topic, dialogue, output_manager, game_world)?,
        CommandResult::SetDialogueWithConditions(npc_name, topic, dialogue, conditions) => handle_set_dialogue_with_conditions(npc_name, topic, dialogue, conditions, output_manager, game_world)?,
        CommandResult::SetEagerness(npc_name, eagerness) => handle_set_eagerness(npc_name, eagerness, output_manager, game_world)?,
        CommandResult::SetRelationship(npc_name, relationship) => handle_set_relationship(npc_name, relationship, output_manager, game_world)?,
        CommandResult::ChangeRelationship(npc_name, delta) => handle_change_relationship(npc_name, delta, output_manager, game_world)?,
        CommandResult::Talk(npc_name, topic) => handle_talk(npc_name, topic, output_manager, game_world, me)?,
        CommandResult::Wait(npc_name) => handle_wait(npc_name, output_manager, game_world, me)?,
        CommandResult::ListNpcs => handle_list_npcs(output_manager, game_world),
        CommandResult::CheckNpc(npc_name) => handle_check_npc(npc_name, output_manager, game_world),
        CommandResult::ToggleTypewriter => handle_toggle_typewriter(output_manager),
        // 任務系統
        CommandResult::QuestList => handle_quest_list(output_manager, game_world),
        CommandResult::QuestActive => handle_quest_active(output_manager, game_world),
        CommandResult::QuestAvailable => handle_quest_available(output_manager, game_world),
        CommandResult::QuestCompleted => handle_quest_completed(output_manager, game_world),
        CommandResult::QuestInfo(quest_id) => handle_quest_info(quest_id, output_manager, game_world),
        CommandResult::QuestStart(quest_id) => handle_quest_start(quest_id, output_manager, game_world)?,
        CommandResult::QuestComplete(quest_id) => handle_quest_complete(quest_id, output_manager, game_world, me)?,
        CommandResult::QuestAbandon(quest_id) => handle_quest_abandon(quest_id, output_manager, game_world)?,
    }
    
    // 玩家指令執行後，檢測靠近/離開（玩家主動行動）
    check_and_handle_proximity(output_manager, game_world, me, true);
    
    // 所有命令執行後，如果小地圖已打開，更新小地圖資料
    if output_manager.is_minimap_open() {
        update_minimap_display(output_manager, game_world, me);
    }
    
    Ok(())
}

/// 處理退出命令
fn handle_exit(
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 保存世界元數據和時間
    game_world.save_metadata()?;
    game_world.save_time()?;
    
    let person_dir = format!("{}/persons", game_world.world_dir);
    std::fs::create_dir_all(&person_dir)?;
    
    // 同步當前 me 到 npc_manager（避免 npc_manager 中的舊資料覆蓋）
    if game_world.current_controlled_id == "me" {
        // 控制 me：更新 npc_manager 中的 me
        let aliases = vec![me.name.clone()];
        game_world.npc_manager.add_npc("me".to_string(), me.clone(), aliases);
    } else {
        // 控制其他 NPC：將當前 NPC 狀態同步回 npc_manager
        let id = game_world.current_controlled_id.clone();
        let aliases = vec![me.name.clone()];
        game_world.npc_manager.add_npc(id, me.clone(), aliases);
        
        // 並更新原始玩家到 npc_manager
        if let Some(original_player) = &game_world.original_player {
            let aliases = vec![original_player.name.clone()];
            game_world.npc_manager.add_npc("me".to_string(), original_player.clone(), aliases);
        }
    }
    
    // 保存所有 NPC 的狀態（已包含更新後的 me）
    game_world.npc_manager.save_all(&person_dir)?;
    
    // 保存遊戲設置
    let game_settings = GameSettings {
        show_minimap: output_manager.is_minimap_open(),
        ..Default::default()
    };
    let _ = game_settings.save();
    
    output_manager.print("遊戲狀態已保存".to_string());
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
            output_manager.print(format!("  {command:<16} - {description}"));
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
    if output_manager.is_status_panel_open() {
        output_manager.toggle_status_panel();
    }
}

/// 處理顯示狀態面板
fn handle_show_status(output_manager: &mut OutputManager, me: &Person) {
    // 顯示狀態面板
    if !output_manager.is_status_panel_open() {
        output_manager.toggle_status_panel();
    }
    output_manager.set_side_observable(Box::new(me.clone()));
    output_manager.set_status("已顯示角色狀態".to_string());
}

/// 處理顯示世界資訊
fn handle_show_world(output_manager: &mut OutputManager, game_world: &GameWorld) {
    if !output_manager.is_status_panel_open() {
        output_manager.toggle_status_panel();
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
    if output_manager.is_status_panel_open() {
        output_manager.toggle_status_panel();
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
                    output_manager.print(format!("  • {ability}"));
                }
            }
            
            if !npc.items.is_empty() {
                output_manager.print("\n🎒 攜帶物品:".to_string());
                for (item, count) in &npc.items {
                    let display_name = item_registry::get_item_display_name(item);
                    output_manager.print(format!("  • {display_name} x{count}"));
                }
            }
            
            output_manager.print("".to_string());
        } else {
            output_manager.set_status(format!("找不到 NPC: {target_name}"));
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
                output_manager.print("\n🎁 此處物品:".to_string());
                for (obj, count) in &point.objects {
                    let display_name = item_registry::get_item_display_name(obj);
                    
                    // 顯示物品年齡信息
                    if let Some(ages) = point.object_ages.get(obj) {
                        if !ages.is_empty() {
                            let avg_age = ages.iter().sum::<u64>() / ages.len() as u64;
                            let days = avg_age / 86400;
                            let hours = (avg_age % 86400) / 3600;
                            output_manager.print(format!("  • {display_name} x{count} (平均存在: {days}天{hours}時)"));
                        } else {
                            output_manager.print(format!("  • {display_name} x{count}"));
                        }
                    } else {
                        output_manager.print(format!("  • {display_name} x{count}"));
                    }
                }
            }
            
            // 顯示當前位置的 NPC（排除當前控制的角色）
            let npcs_here = game_world.npc_manager.get_npcs_at_in_map_excluding(
                &game_world.current_map_name, 
                me.x, 
                me.y,
                &game_world.current_controlled_id
            );
            if !npcs_here.is_empty() {
                output_manager.print("\n👥 此處的人物:".to_string());
                for npc in npcs_here {
                    output_manager.print(format!("  • {} - {}", npc.name, npc.description));
                    
                    // 嘗試觸發 NPC 對話（"見面"場景，根據玩家屬性）
                    if let Some(greeting) = npc.try_talk("見面", me) {
                        output_manager.print(format!("💬 {} 說：「{}」", npc.name, greeting));
                    }
                }
            }
            
            output_manager.print("".to_string());          
        }
    }
}

/// 檢測並處理靠近/離開通知
fn check_and_handle_proximity(
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &Person,
    player_initiated: bool,  // true = 玩家主動行動, false = NPC AI 移動
) {
    let current_controlled_id = &game_world.current_controlled_id;
    let (current_x, current_y, current_map) = if current_controlled_id == "me" {
        (me.x, me.y, me.map.clone())
    } else if let Some(npc) = game_world.npc_manager.get_npc(current_controlled_id) {
        (npc.x, npc.y, npc.map.clone())
    } else {
        (me.x, me.y, me.map.clone())
    };
    
    let proximity_notifications = game_world.npc_manager.update_proximity(
        current_controlled_id,
        current_x,
        current_y,
        &current_map,
        player_initiated,
    );
    
    for (npc_id, message, should_greet) in proximity_notifications {
        output_manager.print(message);
        
        // 如果應該說見面語
        if should_greet {
            if let Some(npc) = game_world.npc_manager.get_npc(&npc_id) {
                // 檢查是否有"見面"對話
                if let Some(greeting) = npc.get_weighted_dialogue("見面", me) {
                    output_manager.print(format!("{} 說：「{}」", npc.name, greeting));
                }
            }
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
        let mut minimap_data: Vec<Line<'static>> = vec![Line::from(format!("【位置: ({}, {})", me.x, me.y))];
        
        // 上方
        if me.y > 0 {
            if let Some(point) = current_map.get_point(me.x, me.y - 1) {
                let walkable = if point.walkable { '\u{2713}' } else { '\u{2718}' };
                minimap_data.push(Line::from(format!("↑ {} {}", point.description, walkable)));
            }
        } else {
            minimap_data.push(Line::from("↑ (邊界)".to_string()));
        }
        
        // 下方
        if me.y + 1 < current_map.height {
            if let Some(point) = current_map.get_point(me.x, me.y + 1) {
                let walkable = if point.walkable { '\u{2713}' } else { '\u{2718}' };
                minimap_data.push(Line::from(format!("↓ {} {}", point.description, walkable)));
            }
        } else {
            minimap_data.push(Line::from("↓ (邊界)".to_string()));
        }
        
        // 左方
        if me.x > 0 {
            if let Some(point) = current_map.get_point(me.x - 1, me.y) {
                let walkable = if point.walkable { '\u{2713}' } else { '\u{2718}' };
                minimap_data.push(Line::from(format!("← {} {}", point.description, walkable)));
            }
        } else {
            minimap_data.push(Line::from("← (邊界)".to_string()));
        }
        
        // 右方
        if me.x + 1 < current_map.width {
            if let Some(point) = current_map.get_point(me.x + 1, me.y) {
                let walkable = if point.walkable { '\u{2713}' } else { '\u{2718}' };
                minimap_data.push(Line::from(format!("→ {} {}", point.description, walkable)));
            }
        } else {
            minimap_data.push(Line::from("→ (邊界)".to_string()));
        }
        
        // 添加分隔線
        minimap_data.push(Line::from("────────────────────────────────────────".to_string()));
        
        // 添加 40x10 網格視圖（玩家周圍，寬40高10）
        let grid_width = 40;
        let grid_height = 10;
        let half_width = grid_width / 2;
        let half_height = grid_height / 2;
        
        for dy in 0..grid_height {
            let mut spans: Vec<Span<'static>> = Vec::new();
            
            for dx in 0..grid_width {
                let calc_x = me.x as i32 - half_width + dx;
                let calc_y = me.y as i32 - half_height + dy;
                
                // 檢查是否超出邊界（包括負數）
                if calc_x < 0 || calc_y < 0 || 
                   calc_x >= current_map.width as i32 || calc_y >= current_map.height as i32 {
                    // 邊界外 - 空白
                    spans.push(Span::styled(
                        String::from(" "),
                        Style::default()
                    ));
                    continue;
                }
                
                let check_x = calc_x as usize;
                let check_y = calc_y as usize;
                
                // 檢查是否是玩家位置
                if check_x == me.x && check_y == me.y {
                    // 玩家位置 - 紅色 P
                    spans.push(Span::styled(
                        String::from("P"),
                        Style::default().fg(Color::Red)
                    ));
                } else {
                    // 檢查該位置是否有 NPC（排除當前控制的角色）
                    let npcs_at_pos = game_world.npc_manager.get_npcs_at_in_map_excluding(
                        &game_world.current_map_name, 
                        check_x, 
                        check_y,
                        &game_world.current_controlled_id
                    );
                    let has_merchant = npcs_at_pos.iter().any(|npc| 
                        npc.name.contains("商人") || npc.name.to_lowercase().contains("merchant")
                    );
                    let has_other_npc = !npcs_at_pos.is_empty();
                    
                    // 檢查該位置是否有物品
                    let has_item = if let Some(point) = current_map.get_point(check_x, check_y) {
                        !point.objects.is_empty()
                    } else {
                        false
                    };
                    
                    // 根據優先級顯示
                    if has_merchant {
                        // 商人 - 綠色 M
                        spans.push(Span::styled(
                            String::from("M"),
                            Style::default().fg(Color::Green)
                        ));
                    } else if has_other_npc {
                        // 其他 NPC - 藍色 N
                        spans.push(Span::styled(
                            String::from("N"),
                            Style::default().fg(Color::Blue)
                        ));
                    } else if has_item {
                        // 物品 - 黃色 I
                        spans.push(Span::styled(
                            String::from("I"),
                            Style::default().fg(Color::Yellow)
                        ));
                    } else if let Some(point) = current_map.get_point(check_x, check_y) {
                        if point.walkable {
                            // 可走 - 深灰色 ·
                            spans.push(Span::styled(
                                String::from("·"),
                                Style::default().fg(Color::Gray)
                            ));
                        } else {
                            // 牆壁 - 白色 ▓
                            spans.push(Span::styled(
                                String::from("▓"),
                                Style::default().fg(Color::White)
                            ));
                        }
                    } else {
                        // 未知 - 灰色 ?
                        spans.push(Span::styled(
                            String::from("?"),
                            Style::default().fg(Color::DarkGray)
                        ));
                    }
                }
            }
            
            minimap_data.push(Line::from(spans));
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
                    output_manager.set_status(format!("往 {direction} 移動"));
                    
                    // 移動後執行look
                    display_look(None, output_manager, game_world, me);
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
                        output_manager.print(format!("✓ 撿起了: {display_name} x{count}"));
                        total_items += count;
                    }
                    point.objects.clear();
                    output_manager.set_status(format!("撿起了 {total_items} 個物品"));
                    should_save_map = true;
                }
                Some(name) => {
                    // 解析物品名稱（支援英文和中文）
                    let resolved_name = item_registry::resolve_item_name(&name);
                    let available = point.get_object_count(&resolved_name);
                    
                    if available == 0 {
                        output_manager.print(format!("找不到 \"{name}\"."));
                        return;
                    }
                    
                    // 取較小值：要求數量 vs 實際數量
                    let actual_quantity = quantity.min(available);
                    let removed = point.remove_objects(&resolved_name, actual_quantity);
                    
                    if removed > 0 {
                        me.add_items(resolved_name.clone(), removed);
                        let display_name = item_registry::get_item_display_name(&resolved_name);
                        output_manager.print(format!("✓ 撿起了: {display_name} x{removed}"));
                        if removed < quantity {
                            output_manager.set_status(format!("只撿起了 {removed} 個 (要求 {quantity})"));
                        } else {
                            output_manager.set_status(format!("撿起: {display_name} x{removed}"));
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
        output_manager.print(format!("你沒有 \"{item_name}\"."));
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
                output_manager.print(format!("✓ 放下了: {display_name} x{actual_quantity}"));
                if actual_quantity < quantity {
                    output_manager.set_status(format!("只放下了 {actual_quantity} 個 (要求 {quantity})"));
                } else {
                    output_manager.set_status(format!("放下: {display_name} x{actual_quantity}"));
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

/// 處理吃食物
fn handle_eat(
    food_name: String,
    output_manager: &mut OutputManager,
    me: &mut Person,
) {
    // 解析物品名稱（支援英文和中文）
    let resolved_name = item_registry::resolve_item_name(&food_name);
    
    // 檢查是否持有該物品
    let owned = me.get_item_count(&resolved_name);
    if owned == 0 {
        output_manager.print(format!("你沒有 \"{food_name}\"."));
        return;
    }
    
    // 檢查是否為食物
    if !item_registry::is_food(&resolved_name) {
        output_manager.print(format!("\"{resolved_name}\" 不是食物，無法食用！"));
        return;
    }
    
    // 獲取食物回復值
    if let Some(hp_restore) = item_registry::get_food_hp(&resolved_name) {
        // 消耗一個食物
        me.drop_items(&resolved_name, 1);
        
        // 回復 HP
        let old_hp = me.hp;
        me.hp += hp_restore;
        let actual_restore = me.hp - old_hp;
        
        output_manager.print(format!("你吃了 \"{}\"，回復了 {} HP！", item_registry::get_item_display_name(&resolved_name), actual_restore));
        output_manager.print(format!("目前 HP: {}", me.hp));
    }
}

/// 處理使用物品命令
fn handle_use_item(
    item_name: String,
    output_manager: &mut OutputManager,
    target: &mut Person,
) {
    match target.use_item(&item_name) {
        Ok(message) => {
            output_manager.print(message);
        },
        Err(error) => {
            output_manager.print(error);
        }
    }
}

fn handle_use_item_on(
    item_name: String,
    target_name: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) {
    // 檢查 NPC 是否在同一位置
    let npcs_here: Vec<&crate::person::Person> = game_world.npc_manager.get_npcs_at_in_map(&game_world.current_map_name, me.x, me.y);
    
    let npc_id = npcs_here.iter()
        .find(|n| n.name.to_lowercase() == target_name.to_lowercase())
        .map(|n| n.name.clone());
    
    if let Some(npc_id) = npc_id {
        // 先給物品給 NPC
        if let Ok(()) = handle_give(target_name.clone(), item_name.clone(), 1, output_manager, game_world, me) {
            // 對實際的 NPC 使用物品
            if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) {
                handle_use_item(item_name.clone(), output_manager, npc);
            } else {
                output_manager.print(format!("無法找到 NPC {target_name}"));
            }
        } else {
            output_manager.print(format!("在 {target_name} 使用 {item_name} 失敗。"));
        }
    } else {
        output_manager.print(format!("這裡沒有名為 {target_name} 的目標。"));
    }
}

/// 處理睡眠命令
fn handle_sleep(
    output_manager: &mut OutputManager,
    me: &mut Person,
) {
    if me.is_sleeping {
        output_manager.print("你已經在睡覺了！".to_string());
        return;
    }
    
    me.is_sleeping = true;
    me.set_status("睡眠中".to_string());
    output_manager.print("💤 你進入了睡眠狀態...".to_string());
    output_manager.print("在睡眠中，你不會消耗 HP，並且每 10 分鐘恢復 10% MP。".to_string());
    output_manager.print("你可以使用 dream 做夢，或使用 wakeup 醒來。".to_string());
}

/// 處理做夢命令
fn handle_dream(
    content: Option<String>,
    output_manager: &mut OutputManager,
) {
    if let Some(dream_content) = content {
        output_manager.print(format!("💭 你夢見了：{dream_content}"));
    } else {
        let dreams = ["你夢見自己在飛翔...",
            "你夢見了一片美麗的花田...",
            "你夢見自己在海邊漫步...",
            "你夢見了童年的回憶...",
            "你夢見了一座神秘的城堡...",
            "你夢見自己成為了英雄..."];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..dreams.len());
        output_manager.print(format!("💭 {}", dreams[idx]));
    }
}

/// 處理醒來命令
fn handle_wakeup(
    output_manager: &mut OutputManager,
    me: &mut Person,
) {
    if !me.is_sleeping {
        output_manager.print("你還沒睡覺呢！".to_string());
        return;
    }
    
    me.is_sleeping = false;
    me.set_status("正常".to_string());
    output_manager.print("☀️ 你醒來了！感覺精神充沛！".to_string());
    output_manager.print(format!("目前 MP: {}", me.mp));
}

/// 處理召喚 NPC
fn handle_summon(
    npc_name: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &Person,
) {
    // 先檢查 NPC 是否存在並獲取名稱
    let npc_info = game_world.npc_manager.get_npc(&npc_name).map(|npc| (npc.name.clone(), npc.x, npc.y));
    
    if let Some((name, old_x, old_y)) = npc_info {
        // 移動 NPC 到玩家位置和地圖
        if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
            npc.move_to(me.x, me.y);
            npc.map = game_world.current_map_name.clone();  // 更新到玩家當前地圖
        }
        
        output_manager.print(format!("你召喚了 {name} 到這裡"));
        output_manager.log(format!("{} 從 ({}, {}) 傳送到 {} ({}, {})", 
            name, old_x, old_y, game_world.current_map_name, me.x, me.y));
    } else {
        output_manager.set_status(format!("找不到 NPC: {npc_name}"));
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
            output_manager.set_status(format!("未知方向: {direction}，請使用 up/down/left/right"));
            return Ok(())
        }
    };
    
    // 計算目標位置
    let target_x = (me.x as i32 + dx) as usize;
    let target_y = (me.y as i32 + dy) as usize;
    
    // 先獲取地圖名稱
    let map_name = game_world.current_map_name.clone();
    
    // 獲取當前地圖並修改
    if let Some(current_map) = game_world.get_current_map_mut() {
        // 檢查目標位置是否在地圖範圍內
        if target_x >= current_map.width || target_y >= current_map.height {
            output_manager.set_status("目標位置超出地圖範圍".to_string());
            return Ok(())
        }
        
        // 獲取目標點
        if let Some(point) = current_map.get_point_mut(target_x, target_y) {
            if point.walkable {
                output_manager.set_status(format!("{dir_name} 方已經是可行走的了"));
            } else {
                // 設置為可行走
                point.walkable = true;
                output_manager.print(format!("你征服了 {dir_name} 方的障礙！"));
                output_manager.print(format!("位置 ({target_x}, {target_y}) 現在可以行走了"));
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
                output_manager.print(format!("你飛到了位置 ({x}, {y})"));
                output_manager.log(format!("玩家傳送到 ({x}, {y})"));
                
                // 保存玩家位置
                let person_dir = format!("{}/persons", game_world.world_dir);
                me.save(&person_dir, "me")?;
                
                // 自動執行 look
                display_look(None, output_manager, game_world, me);
                return Ok(())
            }
            output_manager.set_status("座標超出地圖範圍".to_string());
            return Ok(())
        }
    }
    
    // 嘗試作為地圖名稱
    if game_world.maps.contains_key(&target) {
        game_world.current_map_name = target.clone();
        me.map = target.clone();  // 更新玩家所在地圖
        // 將玩家移動到地圖中心
        if let Some(new_map) = game_world.get_current_map() {
            let center_x = new_map.width / 2;
            let center_y = new_map.height / 2;
            me.move_to(center_x, center_y);
            output_manager.print(format!("你飛到了地圖「{target}」"));
            output_manager.log(format!("玩家傳送到地圖「{target}」({center_x}, {center_y})"));
            
            // 保存玩家位置和世界狀態
            let person_dir = format!("{}/persons", game_world.world_dir);
            me.save(&person_dir, "me")?;
            game_world.save_metadata()?;
            
            // 自動執行 look
            display_look(None, output_manager, game_world, me);
            return Ok(())
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
                    return Ok(())
                }
            }
        }
    }
    
    output_manager.set_status(format!("找不到目標: {target}（請使用座標x,y、地圖名或地點名）"));
    Ok(())
}

/// 處理 namehere 指令 - 命名當前地點
fn handle_namehere(
    name: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &Person,
) -> Result<(), Box<dyn std::error::Error>> {
    let map_name = game_world.current_map_name.clone();
    
    if let Some(current_map) = game_world.get_current_map_mut() {
        if let Some(point) = current_map.get_point_mut(me.x, me.y) {
            let old_name = if point.name.is_empty() {
                "（無名）".to_string()
            } else {
                point.name.clone()
            };
            
            point.name = name.clone();
            output_manager.print(format!("你將此地命名為「{name}」"));
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
    _me: &Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 嘗試解析為坐標
    if let Some((x, y)) = parse_coordinates(&target) {
        let map_name = game_world.current_map_name.clone();
        
        if let Some(current_map) = game_world.get_current_map_mut() {
            if x < current_map.width && y < current_map.height {
                if let Some(point) = current_map.get_point_mut(x, y) {
                    let old_name = if point.name.is_empty() {
                        "（無名）".to_string()
                    } else {
                        point.name.clone()
                    };
                    
                    point.name = new_name.clone();
                    output_manager.print(format!("你將位置 ({x}, {y}) 命名為「{new_name}」"));
                    output_manager.log(format!("位置 ({x}, {y}) 從 {old_name} 更名為「{new_name}」"));
                }
            } else {
                output_manager.set_status("座標超出地圖範圍".to_string());
                return Ok(())
            }
        }
        
        // 保存地圖
        if let Some(map) = game_world.maps.get(&map_name) {
            game_world.save_map(map)?;
        }
        
        return Ok(())
    }
    
    // 嘗試作為 NPC
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&target) {
        let old_name = npc.name.clone();
        npc.name = new_name.clone();
        output_manager.print(format!("你將「{old_name}」改名為「{new_name}」"));
        output_manager.log(format!("NPC 從「{old_name}」更名為「{new_name}」"));
        
        return Ok(())
    }
    
    output_manager.set_status(format!("找不到目標: {target}（請使用座標x,y或NPC名稱）"));
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
    if let Some((npc_id, npc)) = game_world.npc_manager.remove_npc_at(&target, me.x, me.y) {
        let npc_name = npc.name.clone();
        output_manager.print(format!("你摧毀了 NPC「{npc_name}」"));
        output_manager.log(format!("NPC「{}」在 ({}, {}) 被刪除", npc_name, me.x, me.y));
        
        // 刪除 NPC 的 JSON 文件
        let person_dir = format!("{}/persons", game_world.world_dir);
        let npc_file_path = format!("{person_dir}/{npc_id}");
        
        if let Err(e) = std::fs::remove_file(&npc_file_path) {
            output_manager.log(format!("⚠️  刪除 NPC 文件失敗: {e}"));
        } else {
            output_manager.log(format!("✅ 已刪除 NPC 文件: {npc_id}.json"));
        }
        
        return Ok(())
    }
    
    // 嘗試作為物品
    let item_name = item_registry::resolve_item_name(&target);
    let map_name = game_world.current_map_name.clone();
    
    if let Some(current_map) = game_world.get_current_map_mut() {
        if let Some(point) = current_map.get_point_mut(me.x, me.y) {
            if let Some(count) = point.objects.get(&item_name) {
                let count_value = *count;
                point.objects.remove(&item_name);
                
                output_manager.print(format!("你摧毀了物品「{}」x{}", item_registry::get_item_display_name(&item_name), count_value));
                output_manager.log(format!("物品「{}」x{} 在 ({}, {}) 被刪除", item_registry::get_item_display_name(&item_name), count_value, me.x, me.y));
                
                // 保存地圖
                if let Some(map) = game_world.maps.get(&map_name) {
                    game_world.save_map(map)?;
                }
                
                return Ok(())
            }
        }
    }
    
    output_manager.set_status(format!("此處找不到「{target}」（NPC 或物品）"));
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
    
    game_world.event_scheduler.last_check_time = (current_day, current_hour, current_minute);
    
    // === 檢查事件 ===
    let events: Vec<&crate::event::GameEvent> = game_world.event_manager.list_events(); // 取得事件的參考
    let mut triggered_event_ids = Vec::new();
    for event in &events {
        let event_id = event.id.clone();
        if let Some(runtime_state) = game_world.event_manager.get_runtime_state(&event_id) {
            if !event.can_trigger(runtime_state) {
                continue;
            }
        }
        let trigger_check = crate::event_scheduler::EventScheduler::new()
            .check_trigger(event, game_world);
        let condition_check = crate::event_scheduler::EventScheduler::new()
            .check_conditions(event, game_world, me);
        if trigger_check && condition_check {
            triggered_event_ids.push(event_id.clone());
        }
    }
    for event_id in triggered_event_ids {
        game_world.event_manager.trigger_event(&event_id);
        if let Some(event) = game_world.event_manager.get_event(&event_id) {
            let event_clone = event.clone();
            let location_info = get_event_location_info(&event_clone, game_world);
            output_manager.log(format!("🎭 事件: {}{}", event_clone.name, location_info));
            
            if let Err(e) = crate::event_executor::EventExecutor::execute_event(
                &event_clone,
                game_world,
                output_manager
            ) {
                output_manager.log(format!("⚠️  事件執行錯誤: {e}"));
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
        return format!(" 在 {map_name}");
    }
    String::new()
}

/// 解析 NPC 類型簡稱
fn resolve_npc_type(type_code: &str) -> String {
    match type_code.to_lowercase().as_str() {
        "m" => "商人".to_string(),
        "w" => "工人".to_string(),
        "d" => "醫生".to_string(),
        "wr" => "戰士".to_string(),
        "en" => "工程師".to_string(),
        "tr" => "老師".to_string(),
        // 如果不是簡稱，返回原始輸入
        _ => type_code.to_string(),
    }
}

/// 處理 create 指令 - 創建物件
fn handle_create(
    obj_type: String,
    item_type: String,
    name: Option<String>,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &Person,
) -> Result<(), Box<dyn std::error::Error>> {
    match obj_type.to_lowercase().as_str() {
        "npc" => {
            // 解析 NPC 類型（支持簡稱）
            let resolved_type = resolve_npc_type(&item_type);
            
            // 創建 NPC
            let npc_name = name.unwrap_or_else(|| resolved_type.clone());
            
            // 檢查 NPC 是否已存在
            if game_world.npc_manager.get_npc(&npc_name).is_some() {
                output_manager.set_status(format!("NPC「{npc_name}」已經存在"));
                return Ok(())
            }
            
            let description = format!("一個{resolved_type}");
            
            // 創建新的 Person 作為 NPC
            let mut npc = Person::new(npc_name.clone(), description);
            npc.x = me.x;
            npc.y = me.y;
            npc.map = game_world.current_map_name.clone();  // 設置在當前地圖
            
            // 確保 NPC 有預設的 10000 金幣
            npc.items.insert("金幣".to_string(), 10_000);
            
            // 使用 NPC 名稱作為 ID（如果重複會被覆蓋）
            let npc_id = npc_name.clone();
            
            // 添加到 NPC 管理器
            game_world.npc_manager.add_npc(npc_id.clone(), npc, vec![]);
            
            output_manager.print(format!("你創建了 NPC「{npc_name}」(類型: {resolved_type})"));
            output_manager.log(format!("NPC「{}」在 ({}, {}) 被創建", npc_name, me.x, me.y));
        },
        "item" => {
            // 創建物品
            let item_name = item_registry::resolve_item_name(&item_type);
            let display_name = name.as_ref().unwrap_or(&item_type);
            let map_name = game_world.current_map_name.clone();
            
            if let Some(current_map) = game_world.get_current_map_mut() {
                if let Some(point) = current_map.get_point_mut(me.x, me.y) {
                    // 添加物品到當前位置
                    *point.objects.entry(item_name.clone()).or_insert(0) += 1;
                    
                    output_manager.print(format!("你創建了物品「{display_name}」(類型: {item_type})"));
                    output_manager.log(format!("物品「{}」在 ({}, {}) 被創建", display_name, me.x, me.y));
                    
                    // 保存地圖
                    if let Some(map) = game_world.maps.get(&map_name) {
                        game_world.save_map(map)?;
                    }
                } else {
                    output_manager.set_status("無法在當前位置創建物品".to_string());
                }
            } else {
                output_manager.set_status("找不到當前地圖".to_string());
            }
        },
        _ => {
            output_manager.set_status(format!("未知類型: {obj_type}，請使用 item 或 npc"));
        }
    }
    
    Ok(())
}

/// 處理 set 命令
fn handle_set(
    target: String,
    attribute: String,
    value: i32,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 檢查是否為設置物品價格：set item <item_name> price <value>
    // 注意：由於命令解析的限制，這裡我們需要檢查 target 是否為 "item"
    // 實際的命令格式應該是：set item <item_name> price <value>
    // 但目前的解析器會將 "item" 作為 target，"<item_name>" 作為 attribute，"price" 作為 value（錯誤）
    // 所以我們需要特殊處理：如果 target == "item"，則 attribute 是物品名稱，value 是價格
    
    if target.to_lowercase() == "item" {
        // 設置物品價格：set item <item_name> <price>
        // 注意：這裡 attribute 實際上是物品名稱，value 是價格
        let item_name = attribute.clone();
        let price = value.max(0) as u32;
        
        crate::trade::TradeSystem::set_item_price(&item_name, price);
        output_manager.print(format!("物品「{item_name}」的價格設置為 {price} 金幣"));
        return Ok(());
    }
    
    // 判斷目標是 me 還是 NPC
    let is_me = target.to_lowercase() == "me" || target == "我";
    
    if is_me {
        // 設置玩家屬性
        match attribute.to_lowercase().as_str() {
            "hp" => {
                me.hp = value;
                output_manager.print(format!("你的 HP 設置為 {value}"));
            },
            "mp" => {
                me.mp = value;
                output_manager.print(format!("你的 MP 設置為 {value}"));
            },
            "strength" | "str" => {
                me.strength = value;
                output_manager.print(format!("你的力量設置為 {value}"));
            },
            "knowledge" | "kno" => {
                me.knowledge = value;
                output_manager.print(format!("你的知識設置為 {value}"));
            },
            "sociality" | "soc" => {
                me.sociality = value;
                output_manager.print(format!("你的交誼設置為 {value}"));
            },
            "gold" | "金幣" | "goldcoin" => {
                let gold_value = value.max(0) as u32;
                me.items.insert("金幣".to_string(), gold_value);
                output_manager.print(format!("你的金幣設置為 {gold_value}"));                
            },
             "sex" | "性別" => {
                me.gender = value.to_string();           
                // 保存玩家
            },
            _ => {
                output_manager.set_status(format!("未知屬性: {attribute}，支持: hp, mp, strength, knowledge, sociality, gold/金幣"));
            }
        }
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = me.save(&person_dir, "me");
    } else {
        // 設置 NPC 屬性
        if let Some(npc) = game_world.npc_manager.get_npc_mut(&target) {
            match attribute.to_lowercase().as_str() {
                "hp" => {
                    npc.hp = value;
                    output_manager.print(format!("{target}的 HP 設置為 {value}"));
                },
                "mp" => {
                    npc.mp = value;
                    output_manager.print(format!("{target}的 MP 設置為 {value}"));
                },
                "strength" | "str" => {
                    npc.strength = value;
                    output_manager.print(format!("{target}的力量設置為 {value}"));
                },
                "knowledge" | "kno" => {
                    npc.knowledge = value;
                    output_manager.print(format!("{target}的知識設置為 {value}"));
                },
                "sociality" | "soc" => {
                    npc.sociality = value;
                    output_manager.print(format!("{target}的交誼設置為 {value}"));
                },
                "gold" | "金幣" | "goldcoin" => {
                    let gold_value = value.max(0) as u32;
                    npc.items.insert("金幣".to_string(), gold_value);
                    output_manager.print(format!("{target}的金幣設置為 {gold_value}"));
                },
                _ => {
                    output_manager.set_status(format!("未知屬性: {attribute}，支持: hp, mp, strength, knowledge, sociality, gold/金幣"));
                }
            }
        } else {
            output_manager.set_status(format!("找不到 NPC: {target}"));
        }
    }
    
    Ok(())
}

/// 處理切換操控角色命令
fn handle_switch_control(
    npc_name: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 步驟1: 如果當前控制的不是原始玩家，先把狀態同步回去並重新加入 NPC 列表
    if game_world.current_controlled_id != "me" {
        // 將當前操控的角色（me）加回 NPC 列表
        let npc_to_restore = me.clone();
        let id = game_world.current_controlled_id.clone();
        // 使用名字作為別名
        let aliases = vec![npc_to_restore.name.clone()];
        game_world.npc_manager.add_npc(id, npc_to_restore, aliases);
    } else {
        // 如果當前控制的是原始玩家，更新 original_player 的狀態
        game_world.original_player = Some(me.clone());
    }
    
    // 步驟 3: 檢查是否切換回原始玩家
    if npc_name.to_lowercase() == "me" || npc_name == "我" || npc_name.to_lowercase() == "player" {
        if let Some(original) = &game_world.original_player {
            *me = original.clone();
            game_world.current_controlled_id = "me".to_string();
            output_manager.print("已切換回原始角色".to_string());
            output_manager.set_status(format!("現在操控: {}", me.name));
        } else {
            output_manager.set_status("你本來就是原始角色！".to_string());
        }
        return Ok(())
    }
    
    // 步驟 4: 切換到指定 NPC（並從 NPC 列表中移除）
    if let Some(npc) = game_world.npc_manager.remove_npc(&npc_name) {
        let npc_id = npc_name.clone();
        *me = npc;  // 直接使用移除的 NPC，不需要克隆
        game_world.current_controlled_id = npc_id;
        
        output_manager.print(format!("已切換控制角色為: {}", me.name));
        output_manager.set_status(format!("現在操控: {}", me.name));
    } else {
        output_manager.set_status(format!("找不到 NPC: {npc_name}"));
    }
    
    Ok(())
}

/// 處理查看 NPC 商品
fn handle_trade(
    npc_name: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    _me: &Person,  // 保留參數以保持 API 一致性，但使用 game_world.player
    interaction_menu: &mut Option<Menu>,  // 新增：互動選單
) -> Result<(), Box<dyn std::error::Error>> {
    // 使用當前控制的角色（game_world.player）而不是 me 參數
    let current_player = &game_world.player;
    
    // 先檢查 NPC 是否在同一位置，並收集必要資訊
    let (npc_found, npc_id, npc_display_name, goods_data) = {
        let npcs_here = game_world.npc_manager.get_npcs_at_in_map(&game_world.current_map_name, current_player.x, current_player.y);
        
        // 在當前位置的 NPC 中查找（支援名稱和別名）
        // 先嘗試通過 get_npc 查找（支援別名），然後檢查是否在當前位置
        let npc_opt = if let Some(global_npc) = game_world.npc_manager.get_npc(&npc_name) {
            // 找到 NPC 後，檢查是否在當前位置
            npcs_here.iter().find(|n| n.name == global_npc.name)
        } else {
            // 如果通過別名找不到，直接在當前位置的 NPC 中搜尋名稱
            npcs_here.iter().find(|n| n.name.to_lowercase() == npc_name.to_lowercase())
        };
        
        if let Some(npc) = npc_opt {
            let id = npc.name.clone();
            let display = npc.name.clone();
            
            // 如果是 Buying 狀態，收集商品資料
            let goods = if matches!(game_world.interaction_state, crate::world::InteractionState::Buying { .. }) {
                Some(crate::trade::TradeSystem::get_npc_goods(npc))
            } else {
                None
            };
            
            (true, id, display, goods)
        } else {
            (false, String::new(), String::new(), None)
        }
    }; // 釋放 npc_manager 的借用
    
    if !npc_found {
        output_manager.set_status(format!("此處找不到 {npc_name}"));
        return Ok(());
    }
    
    // 根據互動狀態決定顯示什麼選單
    match &game_world.interaction_state {
        crate::world::InteractionState::Buying { .. } => {
            // 顯示購買物品選單
            if let Some(goods) = goods_data {
                if goods.is_empty() {
                    output_manager.print(format!("{npc_display_name} 目前沒有商品"));
                    game_world.interaction_state = crate::world::InteractionState::None;
                    // 取消 NPC 的互動狀態
                    if let Some(npc_mut) = game_world.npc_manager.get_npc_mut(&npc_id) {
                        npc_mut.is_interacting = false;
                    }
                } else {
                    let mut items = Vec::new();
                    for (item_name, quantity, price) in goods {
                        let display_name = item_registry::get_item_display_name(&item_name);
                        items.push(format!("{display_name} x{quantity} - {price} 金幣"));
                    }
                    items.push("返回".to_string());
                    
                    let mut menu = Menu::new(
                        format!("購買物品 - {npc_display_name}"),
                        items,
                    );
                    menu.activate();
                    *interaction_menu = Some(menu);
                }
            }
        },
        crate::world::InteractionState::Selling { .. } => {
            // 顯示出售物品選單（顯示當前控制的角色持有的物品）
            let player_items = crate::trade::TradeSystem::get_player_items(&game_world.player);
            
            if player_items.is_empty() {
                output_manager.print("你目前沒有可以出售的物品".to_string());
                game_world.interaction_state = crate::world::InteractionState::None;
                // 取消 NPC 的互動狀態
                if let Some(npc_mut) = game_world.npc_manager.get_npc_mut(&npc_id) {
                    npc_mut.is_interacting = false;
                }
            } else {
                let mut items = Vec::new();
                for (item_name, quantity, price) in player_items {
                    let display_name = item_registry::get_item_display_name(&item_name);
                    items.push(format!("{display_name} x{quantity} - {price} 金幣/個"));
                }
                items.push("返回".to_string());
                
                let mut menu = Menu::new(
                    format!("出售物品 - {npc_display_name}"),
                    items,
                );
                menu.activate();
                *interaction_menu = Some(menu);
            }
        },
        _ => {
            // 顯示交易主選單（買/賣選擇）
            game_world.interaction_state = crate::world::InteractionState::Trading { 
                npc_name: npc_id.clone() 
            };
            
            // 設定 NPC 為互動中狀態
            if let Some(npc_mut) = game_world.npc_manager.get_npc_mut(&npc_id) {
                npc_mut.is_interacting = true;
            }
            
            let mut menu = Menu::new(
                format!("與 {npc_display_name} 交易"),
                vec![
                    "購買物品".to_string(),
                    "出售物品".to_string(),
                    "離開".to_string(),
                ],
            );
            menu.activate();
            *interaction_menu = Some(menu);
            
            output_manager.print(format!("開始與 {npc_display_name} 交易"));
        }
    }
    
    Ok(())
}

/// 處理購買物品
fn handle_buy(
    npc_name: String,
    item_name: String,
    quantity: u32,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 檢查 NPC 是否在同一位置
    let npcs_here: Vec<&crate::person::Person> = game_world.npc_manager.get_npcs_at_in_map(&game_world.current_map_name, me.x, me.y);
    
    // 使用別名系統查找 NPC
    let npc_found = if let Some(npc) = game_world.npc_manager.get_npc(&npc_name) {
        npcs_here.iter().any(|n| n.name == npc.name)
    } else {
        false
    };
    
    if !npc_found {
        output_manager.set_status(format!("此處找不到 {npc_name}"));
        return Ok(())
    }
    
    // 解析物品名稱
    let resolved_item = item_registry::resolve_item_name(&item_name);
    
    // 計算價格
    let price = crate::trade::TradeSystem::calculate_buy_price(&resolved_item, quantity);
    
    // 獲取 NPC 名稱的克隆，以便在調用 buy_from_npc 時釋放 game_world 的可變借用
    let npc_name_clone_for_trade = {
        if let Some(npc) = game_world.npc_manager.get_npc(&npc_name) {
            let npc_id = npc.name.clone();
            // 確認這個 NPC 在當前位置
            if npcs_here.iter().any(|n| n.name == npc_id) {
                Some(npc_id)
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(npc_id) = npc_name_clone_for_trade {
        let result = crate::trade::TradeSystem::buy_from_npc(game_world, &npc_id, &resolved_item, quantity, price);
        
        match result {
            crate::trade::TradeResult::Success(msg) => {
                output_manager.print(msg);
                
                // 同步 me 和 game_world.player（因為 buy_from_npc 修改了 world.player）
                *me = game_world.player.clone();
                
                // 保存玩家
                let person_dir = format!("{}/persons", game_world.world_dir);
                let _ = me.save(&person_dir, "me");
            },
            crate::trade::TradeResult::Failed(msg) => {
                output_manager.set_status(msg);
            },
        }
    } else {
        output_manager.set_status(format!("此處找不到 {npc_name}"));
    }
    
    Ok(())
}

/// 處理出售物品
fn handle_sell(
    npc_name: String,
    item_name: String,
    quantity: u32,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 檢查 NPC 是否在同一位置
    let npcs_here: Vec<&crate::person::Person> = game_world.npc_manager.get_npcs_at_in_map(&game_world.current_map_name, me.x, me.y);
    
    // 使用別名系統查找 NPC
    let npc_found = if let Some(npc) = game_world.npc_manager.get_npc(&npc_name) {
        npcs_here.iter().any(|n| n.name == npc.name)
    } else {
        false
    };
    
    if !npc_found {
        output_manager.set_status(format!("此處找不到 {npc_name}"));
        return Ok(())
    }
    
    // 解析物品名稱
    let resolved_item = item_registry::resolve_item_name(&item_name);
    
    // 計算價格
    let price = crate::trade::TradeSystem::calculate_sell_price(&resolved_item, quantity);
    
    // 獲取 NPC 名稱的克隆，以便在調用 sell_to_npc 時釋放 game_world 的可變借用
    let npc_name_clone_for_trade = {
        if let Some(npc) = game_world.npc_manager.get_npc(&npc_name) {
            let npc_id = npc.name.clone();
            // 確認這個 NPC 在當前位置
            if npcs_here.iter().any(|n| n.name == npc_id) {
                Some(npc_id)
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(npc_id) = npc_name_clone_for_trade {
        let result = crate::trade::TradeSystem::sell_to_npc(game_world, &npc_id, &resolved_item, quantity, price);
         match result {
            crate::trade::TradeResult::Success(msg) => {
                output_manager.print(msg);
                
                // 同步 me 和 game_world.player（因為 sell_to_npc 修改了 world.player）
                *me = game_world.player.clone();
                
                // 保存玩家
                let person_dir = format!("{}/persons", game_world.world_dir);
                let _ = me.save(&person_dir, "me");
            },
            crate::trade::TradeResult::Failed(msg) => {
                output_manager.set_status(msg);
            },
        }
    } else {
        output_manager.set_status(format!("此處找不到 {npc_name}"));
    }
    
    Ok(())
}

/// 處理給予物品給 NPC
fn handle_give(
    npc_name: String,
    item_name: String,
    quantity: u32,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 檢查 NPC 是否在同一位置
    let npcs_here: Vec<&crate::person::Person> = game_world.npc_manager.get_npcs_at_in_map(&game_world.current_map_name, me.x, me.y);
    
    let npc_found = npcs_here.iter().any(|n| {
        n.name.to_lowercase() == npc_name.to_lowercase() ||
        (npc_name.to_lowercase() == "merchant" && n.description.contains("商"))
    });
    
    if !npc_found {
        output_manager.set_status(format!("此處找不到 {npc_name}"));
        return Ok(())
    }
    
    // 解析物品名稱
    let resolved_item = item_registry::resolve_item_name(&item_name);
    
    // 檢查玩家是否擁有該物品
    if !me.items.contains_key(&resolved_item) {
        output_manager.set_status(format!("你沒有 {resolved_item}"));
        return Ok(())
    }
    
    let player_quantity = me.get_item_count(&resolved_item);
    if player_quantity < quantity {
        output_manager.set_status(format!("你只有 {player_quantity} 個 {resolved_item}，不足 {quantity} 個"));
        return Ok(())
    }
    
    // 從玩家背包移除物品
    if me.drop_items(&resolved_item, quantity).is_none() {
        output_manager.set_status(format!("無法移除物品 {resolved_item}"));
        return Ok(())
    }
    
    // 將物品加到 NPC 背包
    let npc_id = {
        let npcs_at_pos = game_world.npc_manager.get_npcs_at_in_map(&game_world.current_map_name, me.x, me.y);
        npcs_at_pos.iter()
            .find(|n| 
                n.name.to_lowercase() == npc_name.to_lowercase()
            )
            .map(|n| n.name.clone())
    };
    
    if let Some(npc_id) = npc_id {
        if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) {
            npc.add_items(resolved_item.clone(), quantity);
            
            output_manager.print(format!("🎁 你給了 {npc_name} {quantity} 個 {resolved_item}"));
            
            // 增加好感度（可選）
            npc.relationship = (npc.relationship + 5).min(100);
            output_manager.print(format!("💖 {npc_name} 對你的好感度增加了！(現在: {})", npc.relationship));
            
            // 保存玩家
            let person_dir = format!("{}/persons", game_world.world_dir);
            let _ = me.save(&person_dir, "me");
        } else {
            output_manager.set_status(format!("無法找到 NPC {npc_name}"));
        }
    } else {
        output_manager.set_status(format!("此處找不到 {npc_name}"));
    }
    
    Ok(())
}

/// 處理列出所有 NPC
fn handle_list_npcs(
    output_manager: &mut OutputManager,
    game_world: &GameWorld,
) {
    let all_npcs = game_world.npc_manager.get_all_npcs();
    
    if all_npcs.is_empty() {
        output_manager.print("目前沒有任何 NPC".to_string());
    } else {
        output_manager.print("".to_string());
        output_manager.print("═══ 所有 NPC ═══".to_string());
        output_manager.print("".to_string());
        
        for npc in &all_npcs {
            output_manager.print(format!("  {} - {} 位於 ({}, {})", 
                npc.name, 
                npc.description,
                npc.x,
                npc.y
            ));
        }
        
        output_manager.print("".to_string());
        output_manager.print(format!("共 {} 個 NPC", all_npcs.len()));
    }
}

fn handle_check_npc(
    npc_name: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
) {
    if let Some(npc) = game_world.npc_manager.get_npc(&npc_name) {
        output_manager.print(npc.show_detail());
    } else {
        output_manager.print(format!("找不到 NPC: {npc_name}"));
    }
}

/// 處理打字機效果切換
fn handle_toggle_typewriter(output_manager: &mut OutputManager) {
    output_manager.typewriter_enabled = !output_manager.typewriter_enabled; // Corrected: Direct field access
    if output_manager.typewriter_enabled { // Corrected: Direct field access
        output_manager.print("打字機效果已開啟".to_string());
    } else {
        output_manager.print("打字機效果已關閉".to_string());
    }
}

/// 處理設置 NPC 對話
fn handle_set_dialogue(
    npc_name: String,
    topic: String,
    dialogue: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
        npc.set_dialogue(topic.clone(), dialogue.clone());
        output_manager.print(format!("已設置 {} 在話題「{}」的對話", npc.name, topic));
    } else {
        output_manager.set_status(format!("找不到 NPC: {npc_name}"));
    }
    Ok(())
}

/// 處理設置帶條件的 NPC 對話
fn handle_set_dialogue_with_conditions(
    npc_name: String,
    topic: String,
    dialogue: String,
    conditions_str: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::person::DialogueOption;
    
    // 解析條件字串 (例如: "顏值>80 and 性別=女 and mp>500")
    let conditions = parse_conditions(&conditions_str);
    
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
        let option = DialogueOption::with_conditions(dialogue.clone(), conditions);
        npc.add_dialogue_option(topic.clone(), option);
        output_manager.print(format!("已設置 {} 在話題「{}」的條件對話（條件: {}）", 
            npc.name, topic, conditions_str));
    } else {
        output_manager.set_status(format!("找不到 NPC: {npc_name}"));
    }
    Ok(())
}

/// 解析條件字串
fn parse_conditions(conditions_str: &str) -> Vec<crate::person::DialogueCondition> {
    use crate::person::DialogueCondition;
    
    // 分割 "and" 來獲取多個條件
    let parts: Vec<&str> = conditions_str.split(" and ").collect();
    let mut conditions = Vec::new();
    
    for part in parts {
        let part = part.trim();
        
        // 嘗試匹配不同的運算子
        if let Some((attr, value)) = part.split_once(">=") {
            conditions.push(DialogueCondition {
                attribute: attr.trim().to_string(),
                operator: ">=".to_string(),
                value: value.trim().to_string(),
            });
        } else if let Some((attr, value)) = part.split_once("<=") {
            conditions.push(DialogueCondition {
                attribute: attr.trim().to_string(),
                operator: "<=".to_string(),
                value: value.trim().to_string(),
            });
        } else if let Some((attr, value)) = part.split_once("!=") {
            conditions.push(DialogueCondition {
                attribute: attr.trim().to_string(),
                operator: "!=".to_string(),
                value: value.trim().to_string(),
            });
        } else if let Some((attr, value)) = part.split_once('>') {
            conditions.push(DialogueCondition {
                attribute: attr.trim().to_string(),
                operator: ">".to_string(),
                value: value.trim().to_string(),
            });
        } else if let Some((attr, value)) = part.split_once('<') {
            conditions.push(DialogueCondition {
                attribute: attr.trim().to_string(),
                operator: "<".to_string(),
                value: value.trim().to_string(),
            });
        } else if let Some((attr, value)) = part.split_once('=') {
            conditions.push(DialogueCondition {
                attribute: attr.trim().to_string(),
                operator: "=".to_string(),
                value: value.trim().to_string(),
            });
        }
    }
    
    conditions
}

/// 處理設置 NPC 說話積極度
fn handle_set_eagerness(
    npc_name: String,
    eagerness: u8,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
        npc.set_talk_eagerness(eagerness); // Corrected method name
        output_manager.print(format!("已設置 {} 的說話積極度為 {}", npc.name, eagerness));
    } else {
        output_manager.set_status(format!("找不到 NPC: {npc_name}"));
    }
    Ok(())
}

/// 處理設置 NPC 好感度
fn handle_set_relationship(
    npc_name: String,
    relationship: i32,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
        npc.relationship = relationship; // Corrected: Direct field access
        output_manager.print(format!("已設置 {} 對你的好感度為 {}", npc.name, relationship));
    } else {
        output_manager.set_status(format!("找不到 NPC: {npc_name}"));
    }
    Ok(())
}

/// 處理改變 NPC 好感度
fn handle_change_relationship(
    npc_name: String,
    delta: i32,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
        npc.change_relationship(delta); // Corrected: Removed "player" argument
        let current_rel = npc.relationship; // Corrected: Direct field access
        output_manager.print(format!("{} 對你的好感度變為 {}", npc.name, current_rel));
    } else {
        output_manager.set_status(format!("找不到 NPC: {npc_name}"));
    }
    Ok(())
}

/// 處理與 NPC 對話
fn handle_talk(
    npc_name: String,
    topic: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &mut Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 檢查 NPC 是否在同一位置
    let npcs_here: Vec<_> = game_world.npc_manager
        .get_npcs_at_in_map(&game_world.current_map_name, me.x, me.y)
        .into_iter()
        .cloned()
        .collect();
    
    let npc_to_talk = npcs_here.iter().find(|n| n.name.to_lowercase() == npc_name.to_lowercase());
    
    if let Some(npc) = npc_to_talk {
        // 觸發對話（使用指定話題，根據玩家屬性評估條件）
        if let Some(dialogue) = npc.try_talk(&topic, me) {
            output_manager.print(format!("💬 跟{}開始{topic}...", npc.name));
            output_manager.print(format!("{} 說：「{}」", npc.name, dialogue));
        } else {
            output_manager.print(format!("{} 對「{}」這個話題似乎不想說話。", npc.name, topic));
        }
    } else {
        output_manager.set_status(format!("此處找不到 {npc_name}"));
    }
    
    Ok(())
}

/// 嘗試叫住單個 NPC，返回是否成功
fn try_stop_npc(
    npc: &mut crate::person::Person,
    output_manager: &mut OutputManager,
) -> bool {
    use rand::Rng;
    
    let success_rate = (50 + npc.relationship / 2).clamp(0, 100);
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(0..100);
    
    if roll < success_rate {
        npc.is_interacting = true;
        output_manager.print(format!("你叫住了 {}", npc.name));
        
        if let Some(response) = npc.get_dialogue("被叫住") {
            output_manager.print(format!("{} 說：「{}」", npc.name, response));
        }
        true
    } else {
        output_manager.print(format!("{} 沒有理會你", npc.name));
        false
    }
}

/// 處理 wait 命令 - 叫住 NPC
fn handle_wait(
    npc_name: String,
    output_manager: &mut OutputManager,
    game_world: &mut GameWorld,
    me: &Person,
) -> Result<(), Box<dyn std::error::Error>> {
    // 如果 npc_name 為空，叫住當前位置所有的 NPC
    if npc_name.is_empty() {
        let npcs_here = game_world.npc_manager.get_npcs_at_in_map(&game_world.current_map_name, me.x, me.y);
        
        if npcs_here.is_empty() {
            output_manager.print("此處沒有 NPC".to_string());
            return Ok(());
        }
        
        let npc_names: Vec<String> = npcs_here.iter().map(|npc| npc.name.clone()).collect();
        let mut success_count = 0;
        let total_count = npc_names.len();
        
        for name in npc_names {
            if let Some(npc) = game_world.npc_manager.get_npc_mut(&name) {
                if try_stop_npc(npc, output_manager) {
                    success_count += 1;
                }
            }
        }
        
        output_manager.set_status(format!("叫住了 {success_count}/{total_count} 個 NPC"));
        return Ok(());
    } 
    // 指定了 NPC 名稱的情況
    else if let Some(npc) =  game_world.npc_manager.get_npc_mut(&npc_name) {
        if npc.map != game_world.current_map_name {
            output_manager.print(format!("{} 不在這個地圖", npc.name));
            return Ok(());
        }
        
        let distance = ((npc.x as i32 - me.x as i32).abs() + (npc.y as i32 - me.y as i32).abs()) as usize;
        
        if distance > 1 {
            output_manager.print(format!("{} 距離太遠，無法叫住", npc.name));
            return Ok(());
        }
        
        try_stop_npc(npc, output_manager);
    } else {
        output_manager.set_status(format!("找不到 {npc_name}"));
    }
    
    Ok(())
}

// =================================================================
// Quest System Handlers
// =================================================================

/// 處理列出所有任務
fn handle_quest_list(output_manager: &mut OutputManager, game_world: &GameWorld) {
    let quests = game_world.quest_manager.quests.values(); // Corrected: Direct access to values
    output_manager.print("".to_string());
    output_manager.print("═══ 所有任務 ═══".to_string());
    for quest in quests {
        output_manager.print(format!("  [{}]{} - {}", quest.get_status_char(), quest.id, quest.name)); // Corrected: quest.name
    }
}

/// 處理列出進行中的任務
fn handle_quest_active(output_manager: &mut OutputManager, game_world: &GameWorld) {
    let quests = game_world.quest_manager.get_active_quests(); // Corrected method name
    output_manager.print("".to_string());
    output_manager.print("═══ 進行中的任務 ═══".to_string());
    if quests.is_empty() {
        output_manager.print("  沒有進行中的任務。".to_string()); // Corrected: to_string()
    } else {
        for quest in quests {
            output_manager.print(format!("  • {} - {}", quest.id, quest.name)); // Corrected: quest.name
        }
    }
}

/// 處理列出可接取的任務
fn handle_quest_available(output_manager: &mut OutputManager, game_world: &GameWorld) {
    let quests = game_world.quest_manager.get_available_quests(); // Corrected method name
    output_manager.print("".to_string());
    output_manager.print("═══ 可接取的任務 ═══".to_string());
    if quests.is_empty() {
        output_manager.print("  沒有可接取的任務。".to_string()); // Corrected: to_string()
    } else {
        for quest in quests {
            output_manager.print(format!("  • {} - {}", quest.id, quest.name)); // Corrected: quest.name
        }
    }
}

/// 處理列出已完成的任務
fn handle_quest_completed(output_manager: &mut OutputManager, game_world: &GameWorld) {
    let quests = game_world.quest_manager.get_completed_quests(); // Corrected method name
    output_manager.print("".to_string());
    output_manager.print("═══ 已完成的任務 ═══".to_string());
    if quests.is_empty() {
        output_manager.print("  尚未完成任何任務。".to_string()); // Corrected: to_string()
    } else {
        for quest in quests {
            output_manager.print(format!("  • {} - {}", quest.id, quest.name)); // Corrected: quest.name
        }
    }
}

/// 處理顯示任務詳情
fn handle_quest_info(quest_id: String, output_manager: &mut OutputManager, game_world: &GameWorld) {
    if let Some(quest) = game_world.quest_manager.get_quest(&quest_id) {
        output_manager.print("".to_string());
        output_manager.print(format!("═══ {} ═══", quest.name)); // Corrected: quest.name
        output_manager.print(format!("ID: {}", quest.id));
        output_manager.print(format!("狀態: {:?}", quest.status));
        output_manager.print(format!("\n目標:\n  {}", quest.description));
        // Removed quest.progress_text as it doesn't exist
    } else {
        output_manager.set_status(format!("找不到任務: {quest_id}"));
    }
}

/// 處理開始任務
fn handle_quest_start(quest_id: String, output_manager: &mut OutputManager, game_world: &mut GameWorld) -> Result<(), Box<dyn std::error::Error>> {
    match game_world.quest_manager.start_quest(&quest_id) {
        Ok(msg) => output_manager.print(msg), // start_quest returns a message string
        Err(e) => output_manager.set_status(e.to_string()),
    }
    Ok(())
}

/// 處理完成任務
fn handle_quest_complete(quest_id: String, output_manager: &mut OutputManager, game_world: &mut GameWorld, me: &mut Person) -> Result<(), Box<dyn std::error::Error>> {
    match game_world.quest_manager.complete_quest(&quest_id) { // Removed 'me' argument
        Ok(rewards_vec) => { // Now returns Vec<QuestReward>
            if let Some(quest) = game_world.quest_manager.get_quest(&quest_id) {
                output_manager.print(format!("任務完成: {}", quest.name)); // Use quest.name
                output_manager.print("獲得獎勵:".to_string());
                apply_quest_reward(rewards_vec, output_manager, me, game_world)?; // Pass Vec<QuestReward>
            } else {
                output_manager.set_status(format!("任務完成但找不到任務詳情: {quest_id}"));
            }
        },
        Err(e) => output_manager.set_status(e.to_string()),
    }
    Ok(())
}

/// 處理放棄任務
fn handle_quest_abandon(quest_id: String, output_manager: &mut OutputManager, game_world: &mut GameWorld) -> Result<(), Box<dyn std::error::Error>> {
    match game_world.quest_manager.abandon_quest(&quest_id) {
        Ok(msg) => output_manager.print(msg), // abandon_quest returns a message string
        Err(e) => output_manager.set_status(e.to_string()),
    }
    Ok(())
}

/// 應用任務獎勵
fn apply_quest_reward(
    rewards: Vec<QuestReward>, // Corrected: takes Vec<QuestReward>
    output_manager: &mut OutputManager,
    me: &mut Person,
    game_world: &mut GameWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    output_manager.print("獲得獎勵:".to_string());
    for reward_item in rewards { // Iterate through each QuestReward
        match reward_item {
            QuestReward::Item { item, count } => {
                let display_name = item_registry::get_item_display_name(&item);
                output_manager.print(format!("  - 物品: {display_name} x{count}"));
                me.add_items(item.to_string(), count); // item is String, no need for .to_string() here, but keep for consistency with other add_items
            },
            QuestReward::Experience { amount } => {
                output_manager.print(format!("  - 經驗值: {amount}"));
                // TODO: Add actual XP gain to player
            },
            QuestReward::Relationship { npc_id, change } => {
                if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) {
                    npc.change_relationship(change); // Corrected: only takes delta
                    output_manager.print(format!("  - {npc_id} 對你的好感度變化: {change}"));
                }
            },
            QuestReward::UnlockDialogue { npc_id, scene, text } => {
                if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) {
                    npc.set_dialogue(scene.to_string(), text.to_string()); // Corrected arguments
                    output_manager.print(format!("  - 解鎖 {npc_id} 的 {scene} 對話"));
                }
            },
            QuestReward::StatBoost { stat, amount } => {
                output_manager.print(format!("  - 屬性提升: {stat} +{amount}"));
                // TODO: Apply stat boost to player
            },
        }
    }
    Ok(())
}
