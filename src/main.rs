// Core game modules
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
mod event;
mod event_scheduler;
mod event_executor;
mod event_loader;
mod command_handler;  // Command parsing (shared by terminal-ui and FFI)
mod command_executor; // Command execution (shared by all modes)
mod ffi;
mod core_output;

// New architecture modules
mod npc_view;
mod npc_action;
mod game_event;
mod message;

// Terminal UI modules (only compiled with terminal-ui feature)
#[cfg(feature = "terminal-ui")]
mod input;
#[cfg(feature = "terminal-ui")]
mod output;
#[cfg(feature = "terminal-ui")]
mod ui;
#[cfg(feature = "terminal-ui")]
mod app;

#[cfg(feature = "terminal-ui")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = ffi::terminal_ui_ffi::ratamud_start_game();
    if result != 0 {
        return Err("遊戲啟動失敗".into());
    }
    Ok(())
}

#[cfg(not(feature = "terminal-ui"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RataMUD compiled without terminal UI.");
    println!("Use FFI functions to integrate with your application.");
    println!("Example: ratamud_register_output_callback(), ratamud_input_command()");
    Ok(())
}



