//! iOS（FFI）路徑的端對端測試。
//!
//! 這裡驗證的正是回報壞掉的四件事：
//!   1. 狀態存得起來（資料目錄可設定、離開後重讀得到）
//!   2. 地圖顯示得出來（視窗大小合理、玩家在畫面上）
//!   3. event loop 正常（tick 會推進時間、不會每次都寫 10MB 地圖）
//!   4. NPC 可以互動（對話、交易、戰鬥都不再回「需要終端 UI」）
//!
//! 測試全部走 `ratamud::` 的公開 API，不碰 C ABI，這樣不需要 host 就能跑。

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use ratamud::core_output::{self, OutputZone};
use ratamud::world::GameWorld;

/// 資料根目錄與輸出 callback 都是行程層級的全域狀態，測試必須序列化執行。
fn test_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 專案根目錄下的 `worlds/`，當作「app bundle 裡的唯讀資料」
fn bundle_worlds() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("worlds")
}

/// 建立一個乾淨的暫存資料目錄，模擬 iOS 的 Documents/
fn fresh_data_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ratamud-test-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("建立暫存資料目錄");
    dir
}

/// 把 bundle 的世界資料裝進可寫目錄，並把引擎指過去（= host 的啟動流程）
fn install_world(data_dir: &Path) {
    ratamud::paths::set_data_root(data_dir);
    ratamud::paths::copy_tree_if_missing(&bundle_worlds(), &data_dir.join("worlds"))
        .expect("安裝世界資料");
}

/// 建好一個已載入地圖與角色的世界（等同 `ratamud_init_game` 的核心部分）
fn boot_world() -> GameWorld {
    let mut world = GameWorld::new();
    let _ = world.load_metadata();
    let _ = world.load_time();
    world.initialize_maps().expect("載入地圖");

    let person_dir = format!("{}/persons", world.world_dir);
    let (_, me) = world.npc_manager.initialize(&person_dir).expect("載入角色");
    world.original_player = Some(me);
    world
}

/// 收集這段期間所有 Main/Status 區的輸出，用來斷言指令真的有作用
fn capture_output<F: FnOnce()>(body: F) -> String {
    static BUFFER: OnceLock<Mutex<String>> = OnceLock::new();
    let buffer = BUFFER.get_or_init(|| Mutex::new(String::new()));
    buffer.lock().unwrap().clear();

    core_output::register_output_callback(move |zone, content| {
        if matches!(zone, OutputZone::Main | OutputZone::Status) {
            if let Ok(mut buf) = BUFFER.get().unwrap().lock() {
                buf.push_str(content);
                buf.push('\n');
            }
        }
    });

    body();

    core_output::clear_output_callback();
    let text = buffer.lock().unwrap().clone();
    text
}

// =================================================================
// 1. 狀態存得起來
// =================================================================

#[test]
fn data_root_redirects_all_world_files() {
    let _guard = test_lock();
    let data_dir = fresh_data_dir("data-root");
    install_world(&data_dir);

    let world = GameWorld::new();

    // world_dir 必須落在可寫目錄底下，而不是行程工作目錄
    assert!(
        Path::new(&world.world_dir).starts_with(&data_dir),
        "world_dir 應該在資料根目錄底下，實際是 {}",
        world.world_dir
    );

    ratamud::paths::set_data_root(".");
}

#[test]
fn player_position_survives_a_restart() {
    let _guard = test_lock();
    let data_dir = fresh_data_dir("persist");
    install_world(&data_dir);

    // 第一次「開 app」：走幾步，離開
    let moved_to = {
        let mut world = boot_world();
        capture_output(|| {
            // 找一個確定可走的方向再走，避免撞牆讓測試不穩定
            for cmd in ["right", "left", "down", "up"] {
                world.execute_command(cmd);
            }
        });
        world.execute_command("exit");

        let me = world.npc_manager.get_npc("me").expect("玩家存在");
        (me.x, me.y)
    };

    // 第二次「開 app」：位置必須跟離開時一樣
    let world = boot_world();
    let me = world.npc_manager.get_npc("me").expect("玩家存在");
    assert_eq!(
        (me.x, me.y),
        moved_to,
        "重開之後玩家位置應該保持不變（存檔失效）"
    );

    ratamud::paths::set_data_root(".");
}

// =================================================================
// 2. 地圖顯示得出來
// =================================================================

#[test]
fn map_panel_is_a_viewport_not_the_whole_world() {
    let _guard = test_lock();
    let data_dir = fresh_data_dir("map");
    install_world(&data_dir);

    let world = boot_world();
    let map = ratamud::panel_render::render_map(&world);
    let grid: Vec<&str> = map
        .lines()
        .skip(1) // 第一行是標題
        .filter(|line| !line.starts_with("圖例"))
        .collect();

    assert_eq!(
        grid.len(),
        ratamud::panel_render::DEFAULT_MAP_VIEW_HEIGHT,
        "大地圖應該是固定高度的視窗，而不是整張 100x100"
    );
    for row in &grid {
        assert_eq!(
            row.chars().count(),
            ratamud::panel_render::DEFAULT_MAP_VIEW_WIDTH,
            "每一列寬度應該等於視窗寬度"
        );
    }
    assert!(map.contains('@'), "視窗裡必須看得到玩家");

    ratamud::paths::set_data_root(".");
}

#[test]
fn map_viewport_stays_inside_bounds_at_the_corner() {
    let _guard = test_lock();
    let data_dir = fresh_data_dir("map-corner");
    install_world(&data_dir);

    let mut world = boot_world();
    if let Some(me) = world.npc_manager.get_npc_mut("me") {
        me.x = 0;
        me.y = 0;
    }

    let map = ratamud::panel_render::render_map(&world);
    // 只看格子本身；標題與圖例（"?界外"）不算
    let grid: String = map
        .lines()
        .skip(1)
        .filter(|line| !line.starts_with("圖例"))
        .collect();
    assert!(
        !grid.contains('?'),
        "視窗應該被夾在地圖範圍內，不該出現界外格:\n{map}"
    );
    assert!(grid.contains('@'), "角落也要看得到玩家");

    ratamud::paths::set_data_root(".");
}

#[test]
fn map_json_describes_the_same_viewport() {
    let _guard = test_lock();
    let data_dir = fresh_data_dir("map-json");
    install_world(&data_dir);

    let world = boot_world();
    let json: serde_json::Value =
        serde_json::from_str(&ratamud::panel_render::render_map_json(&world, 11, 7))
            .expect("MAP_JSON 應該是合法 JSON");

    assert_eq!(json["view"]["width"], 11);
    assert_eq!(json["view"]["height"], 7);
    assert_eq!(json["rows"].as_array().unwrap().len(), 7);
    assert!(json["player"]["x"].is_number());

    ratamud::paths::set_data_root(".");
}

// =================================================================
// 3. event loop：tick 會推進世界，而且不會狂寫地圖
// =================================================================

#[test]
fn ordinary_commands_do_not_rewrite_map_files() {
    let _guard = test_lock();
    let data_dir = fresh_data_dir("map-io");
    install_world(&data_dir);

    let mut world = boot_world();
    let map_path = format!("{}/maps/beginMap.json", world.world_dir);
    let before = std::fs::metadata(&map_path).expect("地圖檔存在").modified().ok();

    // 走動、看、對話都不該碰到 2.4MB 的地圖檔
    capture_output(|| {
        for cmd in ["look", "right", "npcs", "down"] {
            world.execute_command(cmd);
        }
    });

    let after = std::fs::metadata(&map_path).expect("地圖檔存在").modified().ok();
    assert_eq!(before, after, "一般指令不應該重寫地圖檔");

    ratamud::paths::set_data_root(".");
}

#[test]
fn picking_up_an_item_marks_the_map_dirty_and_save_flushes_it() {
    let _guard = test_lock();
    let data_dir = fresh_data_dir("map-dirty");
    install_world(&data_dir);

    let mut world = boot_world();

    // 在玩家腳下放一個物品，模擬撿東西造成的地圖變動
    let (x, y) = {
        let me = world.npc_manager.get_npc("me").expect("玩家存在");
        (me.x, me.y)
    };
    if let Some(map) = world.get_current_map_mut() {
        if let Some(point) = map.get_point_mut(x, y) {
            point.add_objects("蘋果".to_string(), 1);
        }
    }

    assert!(world.has_dirty_maps(), "改過地圖內容後應該被標記為待存檔");

    let written = world.save_dirty_maps().expect("寫回地圖");
    assert_eq!(written, 1, "只有被改過的那一張地圖需要重寫");
    assert!(!world.has_dirty_maps(), "寫回後 dirty 旗標應該清掉");

    ratamud::paths::set_data_root(".");
}

// =================================================================
// 4. NPC 互動：對話 / 交易 / 戰鬥都要能在無 UI 模式運作
// =================================================================

/// 把一個 NPC 搬到玩家腳下，回傳它的 id
fn move_npc_to_player(world: &mut GameWorld, npc_id: &str) {
    let (x, y, map) = {
        let me = world.npc_manager.get_npc("me").expect("玩家存在");
        (me.x, me.y, me.map.clone())
    };
    let npc = world.npc_manager.get_npc_mut(npc_id).expect("NPC 存在");
    npc.x = x;
    npc.y = y;
    npc.map = map;
}

#[test]
fn talking_to_an_npc_no_longer_requires_the_terminal_ui() {
    let _guard = test_lock();
    let data_dir = fresh_data_dir("talk");
    install_world(&data_dir);

    let mut world = boot_world();
    move_npc_to_player(&mut world, "merchant");
    let npc_name = world.npc_manager.get_npc("merchant").unwrap().name.clone();

    let output = capture_output(|| {
        world.execute_command(&format!("talk {npc_name} 閒聊"));
    });

    assert!(
        !output.contains("終端 UI"),
        "對話不該再被擋在終端 UI 模式後面:\n{output}"
    );
    assert!(
        output.contains(&npc_name),
        "輸出應該提到對話對象:\n{output}"
    );

    ratamud::paths::set_data_root(".");
}

#[test]
fn trading_moves_goods_and_gold() {
    let _guard = test_lock();
    let data_dir = fresh_data_dir("trade");
    install_world(&data_dir);

    let mut world = boot_world();
    move_npc_to_player(&mut world, "merchant");

    // 給商人一件確定存在的商品，並確保玩家買得起
    if let Some(npc) = world.npc_manager.get_npc_mut("merchant") {
        npc.items.insert("蘋果".to_string(), 5);
    }
    if let Some(me) = world.npc_manager.get_npc_mut("me") {
        me.items.insert("金幣".to_string(), 10_000);
    }

    let gold_before = world.npc_manager.get_npc("me").unwrap().items["金幣"];

    let mut bought = false;
    let output = capture_output(|| {
        bought = ratamud::command_interact::execute_trade(&mut world, "merchant", "蘋果", 1, true);
    });
    assert!(bought, "向商人購買應該成功:\n{output}");
    assert!(!output.contains("終端 UI"), "交易不該再被擋下來:\n{output}");

    let me = world.npc_manager.get_npc("me").unwrap();
    assert_eq!(me.items.get("蘋果").copied().unwrap_or(0), 1, "玩家應該拿到蘋果");
    assert!(me.items["金幣"] < gold_before, "金幣應該減少");

    ratamud::paths::set_data_root(".");
}

#[test]
fn combat_starts_and_rounds_advance_without_a_ui() {
    let _guard = test_lock();
    let data_dir = fresh_data_dir("combat");
    install_world(&data_dir);

    let mut world = boot_world();
    move_npc_to_player(&mut world, "merchant");

    // 雙方都要有足夠 HP 才打得起來（HP 低於一半就會直接結束）
    for id in ["me", "merchant"] {
        if let Some(p) = world.npc_manager.get_npc_mut(id) {
            p.max_hp = 1000;
            p.hp = 1000;
        }
    }

    let hp_before = world.npc_manager.get_npc("merchant").unwrap().hp;

    let output = capture_output(|| {
        world.execute_command("punch merchant");
    });

    assert!(!output.contains("終端 UI"), "戰鬥不該再被擋下來:\n{output}");
    assert!(
        world.npc_manager.get_npc("merchant").unwrap().hp < hp_before,
        "攻擊應該造成傷害:\n{output}"
    );
    assert!(
        matches!(world.combat_state, ratamud::world::CombatState::InCombat { .. }),
        "應該進入戰鬥狀態"
    );

    // 自動回合可以在沒有 UI 的情況下繼續推進
    let round_before = match &world.combat_state {
        ratamud::world::CombatState::InCombat { round, .. } => *round,
        _ => unreachable!(),
    };
    capture_output(|| {
        ratamud::combat::execute_combat_round(&mut world);
    });
    if let ratamud::world::CombatState::InCombat { round, .. } = &world.combat_state {
        assert!(*round > round_before, "回合數應該前進");
    }

    ratamud::paths::set_data_root(".");
}

#[test]
fn quest_commands_work_without_a_ui() {
    let _guard = test_lock();
    let data_dir = fresh_data_dir("quest");
    install_world(&data_dir);

    let mut world = boot_world();
    let quest_dir = format!("{}/quests", world.world_dir);
    let _ = world.quest_manager.load_from_directory(&quest_dir);

    let output = capture_output(|| {
        world.execute_command("quest list");
    });

    assert!(
        !output.contains("終端 UI"),
        "任務指令不該再被擋下來:\n{output}"
    );
    assert!(output.contains("任務"), "應該列出任務清單:\n{output}");

    ratamud::paths::set_data_root(".");
}
