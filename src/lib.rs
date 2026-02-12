// Core game modules (always available)
pub mod map;
pub mod item;
pub mod item_registry;
pub mod person;
pub mod npc_manager;
pub mod npc_ai;
pub mod trade;
pub mod quest;
pub mod world;
pub mod event;
pub mod event_loader;
pub mod event_executor;
pub mod event_scheduler;
pub mod time_updatable;
pub mod time_thread;
pub mod settings;
pub mod ffi;

// New architecture modules
pub mod npc_view;
pub mod npc_action;
pub mod game_event;
pub mod message;

// Terminal UI modules (only available with terminal-ui feature)
#[cfg(feature = "terminal-ui")]
pub mod input;
#[cfg(feature = "terminal-ui")]
pub mod output;
#[cfg(feature = "terminal-ui")]
pub mod ui;
#[cfg(feature = "terminal-ui")]
pub mod app;

// Core output interface (always available)
pub mod core_output;
