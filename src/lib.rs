pub mod map;
pub mod observable;
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
pub mod npc_ai_thread;
pub mod input;
pub mod output;
pub mod settings;
pub mod ui;
pub mod app;
pub mod ffi;
pub mod callback;
pub mod command_processor;  // 新增：純文本命令處理器
pub mod game_engine;        // 新增：無頭遊戲引擎
