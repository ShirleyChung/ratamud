// 模組聲明
mod input;
mod output;
mod ui;
mod world;
mod person;
mod npc_manager;
mod npc_ai;
mod trade;
mod quest;
mod map;
mod time_updatable;
mod time_thread;
mod item;
mod item_registry;
mod settings;
mod app;
mod event;
mod event_scheduler;
mod event_executor;
mod event_loader;
mod ffi;

// 新架構模組
mod npc_view;
mod npc_action;
mod game_event;
mod message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = ffi::ratamud_start_game();
    if result != 0 {
        return Err("遊戲啟動失敗".into());
    }
    Ok(())
}


