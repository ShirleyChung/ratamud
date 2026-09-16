/// 命令執行器 - 無 UI 模式的命令執行邏輯
/// 這個模組提供了所有遊戲命令的執行邏輯，使用 CoreOutputManager 進行輸出
/// FFI 模式和其他無 UI 模式都可以使用這個模組
use crate::command_handler::CommandResult;
use crate::world::GameWorld;
use crate::person::Person;
use crate::core_output::{OutputZone, trigger_output};
// NPC 互動 / 交易 / 任務 / 戰鬥都在共用核心模組裡，終端與 FFI 兩邊共用
use crate::combat::{handle_combat_skill, handle_escape};
use crate::command_interact as interact;

/// 執行命令並返回是否應該繼續遊戲
/// 返回 true=繼續, false=退出
pub fn execute_command(game_world: &mut GameWorld, command: &str) -> bool {
    use crate::command_handler;
    
    let result = command_handler::parse_command(command);
    let current_id = game_world.current_controlled_id.clone();
    
    match result {
        CommandResult::Exit => {
            trigger_output(OutputZone::Main, "再見！");
            false
        },
        CommandResult::Help => {
            handle_help();
            true
        },
        CommandResult::Clear => {
            trigger_output(OutputZone::Main, "\n\n\n清除輸出\n\n\n");
            true
        },
        CommandResult::Output(msg) => {
            trigger_output(OutputZone::Main, &msg);
            true
        },
        CommandResult::Error(err) => {
            trigger_output(OutputZone::Status, &format!("錯誤: {}", err));
            true
        },
        CommandResult::Look(target) => {
            handle_look(game_world, &current_id, target);
            true
        },
        CommandResult::Move(dx, dy) => {
            handle_move(game_world, &current_id, dx, dy);
            true
        },
        CommandResult::Get(item_name, quantity) => {
            handle_get(game_world, &current_id, item_name, quantity);
            true
        },
        CommandResult::Drop(item_name, quantity) => {
            handle_drop(game_world, &current_id, item_name, quantity);
            true
        },
        CommandResult::Eat(food_name) => {
            handle_eat(game_world, &current_id, food_name);
            true
        },
        CommandResult::Summon(npc_name) => {
            handle_summon(game_world, &current_id, npc_name);
            true
        },
        CommandResult::ListNpcs => {
            handle_list_npcs(game_world);
            true
        },
        CommandResult::CheckNpc(npc_name) => {
            handle_check_npc(game_world, npc_name);
            true
        },
        CommandResult::ShowWorld => {
            handle_show_world(game_world);
            true
        },
        CommandResult::SwitchControl(npc_name) => {
            handle_switch_control(game_world, npc_name);
            true
        },
        CommandResult::Conquer(direction) => {
            handle_conquer(game_world, &current_id, direction);
            true
        },
        CommandResult::FlyTo(target) => {
            handle_flyto(game_world, &current_id, target);
            true
        },
        CommandResult::NameHere(name) => {
            handle_namehere(game_world, &current_id, name);
            true
        },
        CommandResult::Name(target, name) => {
            handle_name(game_world, &current_id, target, name);
            true
        },
        CommandResult::Destroy(target) => {
            handle_destroy(game_world, &current_id, target);
            true
        },
        CommandResult::Create(obj_type, item_type, name) => {
            handle_create(game_world, &current_id, obj_type, item_type, name);
            true
        },
        CommandResult::Set(target, attribute, value) => {
            handle_set(game_world, &current_id, target, attribute, value);
            true
        },
        CommandResult::Give(npc_name, item, quantity) => {
            handle_give(game_world, &current_id, npc_name, item, quantity);
            true
        },
        CommandResult::UseItem(item_name) => {
            handle_use_item(game_world, &current_id, item_name);
            true
        },
        CommandResult::Sleep => {
            handle_sleep(game_world, &current_id);
            true
        },
        CommandResult::Dream(content) => {
            handle_dream(game_world, &current_id, content);
            true
        },
        CommandResult::WakeUp => {
            handle_wakeup(game_world, &current_id);
            true
        },
        CommandResult::Punch(target) => {
            handle_combat_skill(game_world, "punch", target);
            true
        },
        CommandResult::Kick(target) => {
            handle_combat_skill(game_world, "kick", target);
            true
        },
        CommandResult::Escape => {
            handle_escape(game_world);
            true
        },
        CommandResult::QuestList => { interact::handle_quest_list(game_world); true },
        CommandResult::QuestActive => { interact::handle_quest_active(game_world); true },
        CommandResult::QuestAvailable => { interact::handle_quest_available(game_world); true },
        CommandResult::QuestCompleted => { interact::handle_quest_completed(game_world); true },
        CommandResult::QuestInfo(quest_id) => { interact::handle_quest_info(game_world, &quest_id); true },
        CommandResult::QuestStart(quest_id) => { interact::handle_quest_start(game_world, &quest_id); true },
        CommandResult::QuestComplete(quest_id) => { interact::handle_quest_complete(game_world, &quest_id); true },
        CommandResult::QuestAbandon(quest_id) => { interact::handle_quest_abandon(game_world, &quest_id); true },
        CommandResult::Trade(npc_name) => {
            interact::handle_trade(game_world, &npc_name);
            true
        },
        CommandResult::Buy(npc_name, item, quantity) => {
            interact::execute_trade(game_world, &npc_name, &item, quantity, true);
            true
        },
        CommandResult::Sell(npc_name, item, quantity) => {
            interact::execute_trade(game_world, &npc_name, &item, quantity, false);
            true
        },
        CommandResult::Talk(npc_name, topic) => {
            interact::handle_talk(game_world, &current_id, &npc_name, &topic);
            true
        },
        CommandResult::Wait(npc_name) => {
            interact::handle_wait(game_world, &current_id, &npc_name);
            true
        },
        CommandResult::Party(npc_name) => {
            interact::handle_party(game_world, &current_id, &npc_name);
            true
        },
        CommandResult::Disband => {
            interact::handle_disband(game_world);
            true
        },
        CommandResult::UseItemOn(item_name, npc_name) => {
            interact::handle_use_item_on(game_world, &current_id, &item_name, &npc_name);
            true
        },
        CommandResult::SetDialogue(npc, topic, text) => {
            interact::handle_set_dialogue(game_world, &npc, &topic, &text, None);
            true
        },
        CommandResult::SetDialogueWithConditions(npc, topic, text, conditions) => {
            interact::handle_set_dialogue(game_world, &npc, &topic, &text, Some(&conditions));
            true
        },
        CommandResult::SetEagerness(npc, eagerness) => {
            interact::handle_set_eagerness(game_world, &npc, eagerness);
            true
        },
        CommandResult::SetRelationship(npc, value) => {
            interact::handle_set_relationship(game_world, &npc, value, false);
            true
        },
        CommandResult::ChangeRelationship(npc, delta) => {
            interact::handle_set_relationship(game_world, &npc, delta, true);
            true
        },
        // 地圖/背包類「顯示」命令在無 UI 模式改成把面板內容送到 Side 區，
        // host（iOS）收到後可以直接顯示，不必自己重做一套渲染。
        CommandResult::ShowMap => {
            trigger_output(OutputZone::Side, &crate::panel_render::render_map(game_world));
            true
        },
        CommandResult::ShowMinimap => {
            trigger_output(OutputZone::Side, &crate::panel_render::render_minimap(game_world));
            true
        },
        // 純終端外觀的命令，在無 UI 模式沒有對應行為
        CommandResult::HideMinimap | CommandResult::ShowLog | CommandResult::HideLog |
        CommandResult::ToggleTypewriter | CommandResult::ShowHistory(_) => {
            trigger_output(OutputZone::Log, "此命令僅在終端 UI 模式可用");
            true
        },
        CommandResult::AddToSide(msg) => {
            trigger_output(OutputZone::Side, &msg);
            true
        },
    }
}

fn handle_help() {
    use crate::command_handler::CommandResult;
    let help_info = CommandResult::get_help_info();
    trigger_output(OutputZone::Main, "=== RataMUD 指令說明 ===");
    for (category, commands) in help_info {
        trigger_output(OutputZone::Main, &format!("\n{}", category));
        for (usage, desc) in commands {
            trigger_output(OutputZone::Main, &format!("  {} - {}", usage, desc));
        }
    }
}

fn handle_look(game_world: &GameWorld, current_id: &str, target: Option<String>) {
    // 指定目標時改成查看該 NPC（與終端版 `look <npc>` 一致）
    if let Some(npc_name) = target {
        look_npc(game_world, &npc_name);
        return;
    }

    // 獲取當前角色位置
    let Some(me) = game_world.npc_manager.get_npc(current_id) else {
        trigger_output(OutputZone::Status, "找不到當前控制的角色");
        return;
    };
    let (x, y) = (me.x, me.y);

    // 顯示當前位置資訊
    let Some(map) = game_world.get_current_map() else { return };
    trigger_output(OutputZone::Main, &format!("📍 {}", map.name));
    trigger_output(OutputZone::Main, &map.description);

    if let Some(point) = map.get_point(x, y) {
        trigger_output(OutputZone::Main, &format!("【當前位置: ({x}, {y})】"));
        trigger_output(OutputZone::Main, &format!("【{}】", point.description));
        if !point.name.is_empty() {
            trigger_output(OutputZone::Main, &format!("此處是【{}】", point.name));
        }

        // 顯示當前位置的物品
        if !point.objects.is_empty() {
            trigger_output(OutputZone::Main, "🎁 此處物品:");
            for (item, count) in &point.objects {
                let display_name = crate::item_registry::get_item_display_name(item);
                trigger_output(OutputZone::Main, &format!("  • {display_name} x{count}"));
            }
        }
    } else {
        trigger_output(OutputZone::Main, &format!("你在 ({x}, {y})"));
    }

    // 顯示同格的 NPC，並觸發見面對話（終端版 display_location_npcs 的行為）
    look_npcs_here(game_world, current_id, me, x, y);
}

/// 查看單一 NPC 的詳細資料
fn look_npc(game_world: &GameWorld, npc_name: &str) {
    let Some(npc) = game_world.npc_manager.get_npc(npc_name) else {
        trigger_output(OutputZone::Status, &format!("找不到 NPC: {npc_name}"));
        return;
    };

    trigger_output(OutputZone::Main, &format!("👤 {}", npc.name));
    trigger_output(OutputZone::Main, &"═".repeat(20));
    trigger_output(OutputZone::Main, &format!("📝 {}", npc.description));
    trigger_output(OutputZone::Main, &format!("📍 位置: ({}, {})", npc.x, npc.y));
    trigger_output(OutputZone::Main, &format!("💫 狀態: {}", npc.status));

    if !npc.abilities.is_empty() {
        trigger_output(OutputZone::Main, "✨ 能力:");
        for ability in &npc.abilities {
            trigger_output(OutputZone::Main, &format!("  • {ability}"));
        }
    }

    if !npc.items.is_empty() {
        trigger_output(OutputZone::Main, "🎒 攜帶物品:");
        for (item, count) in &npc.items {
            let display_name = crate::item_registry::get_item_display_name(item);
            trigger_output(OutputZone::Main, &format!("  • {display_name} x{count}"));
        }
    }
}

/// 列出同一格的 NPC，並觸發「見面」對話
fn look_npcs_here(game_world: &GameWorld, current_id: &str, me: &Person, x: usize, y: usize) {
    let npcs_here = game_world.npc_manager.get_npcs_at_in_map_excluding(
        &game_world.current_map_name,
        x,
        y,
        current_id,
    );

    if npcs_here.is_empty() {
        return;
    }

    trigger_output(OutputZone::Main, "👥 此處的人物:");
    for npc in npcs_here {
        let mut desc = format!("  • {} - {}", npc.name, npc.description);
        if let Some(ref leader) = npc.party_leader {
            desc.push_str(&format!(" (已與\"{leader}\"組隊)"));
        }
        trigger_output(OutputZone::Main, &desc);

        if let Some(greeting) = npc.try_talk("見面", me) {
            trigger_output(OutputZone::Main, &format!("💬 {} 說：「{}」", npc.name, greeting));
        }
    }
}

fn handle_move(game_world: &mut GameWorld, current_id: &str, dx: i32, dy: i32) {
    use crate::world::CombatState;

    // 戰鬥中無法移動（與終端版一致）
    if !matches!(game_world.combat_state, CombatState::None) {
        trigger_output(OutputZone::Main, "戰鬥中無法移動！");
        return;
    }

    // 獲取當前位置
    let (old_x, old_y) = if let Some(me) = game_world.npc_manager.get_npc(current_id) {
        (me.x, me.y)
    } else {
        trigger_output(OutputZone::Status, "找不到當前控制的角色");
        return;
    };

    // 往左/上走到 0 會 underflow 成極大值，先擋掉
    if (dx < 0 && old_x == 0) || (dy < 0 && old_y == 0) {
        trigger_output(OutputZone::Status, "超出地圖範圍");
        return;
    }
    let new_x = (old_x as i32 + dx) as usize;
    let new_y = (old_y as i32 + dy) as usize;

    // 檢查邊界與可行走性
    let Some(map) = game_world.get_current_map() else {
        trigger_output(OutputZone::Status, "當前沒有地圖");
        return;
    };
    if new_x >= map.width || new_y >= map.height {
        trigger_output(OutputZone::Status, "超出地圖範圍");
        return;
    }
    let can_walk = map.get_point(new_x, new_y).map(|p| p.walkable).unwrap_or(false);
    if !can_walk {
        trigger_output(OutputZone::Status, "前方是牆壁，無法通過");
        return;
    }

    // 更新位置
    let person_dir = format!("{}/persons", game_world.world_dir);
    if let Some(me) = game_world.npc_manager.get_npc_mut(current_id) {
        me.x = new_x;
        me.y = new_y;
        let _ = me.save(&person_dir, current_id);
    }

    let direction = match (dx, dy) {
        (0, -1) => "北",
        (0, 1) => "南",
        (1, 0) => "東",
        (-1, 0) => "西",
        _ => "未知方向",
    };
    trigger_output(OutputZone::Main, &format!("你向{direction}移動到 ({new_x}, {new_y})"));

    // 組隊中的 NPC 跟著走
    move_party_npcs(game_world, new_x, new_y);

    // 更新靠近/離開狀態（玩家主動移動）
    report_proximity(game_world, true);

    // 移動後自動 look，讓 host 不必再送一次指令
    handle_look(game_world, current_id, None);
}

/// 讓組隊的 NPC 跟隨玩家移動
fn move_party_npcs(game_world: &mut GameWorld, player_x: usize, player_y: usize) {
    let current_map = game_world.current_map_name.clone();
    let person_dir = format!("{}/persons", game_world.world_dir);

    for npc_id in game_world.npc_manager.get_all_npc_ids() {
        let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) else { continue };
        if npc.party_leader.as_deref() != Some("me") || npc.map != current_map {
            continue;
        }
        npc.x = player_x;
        npc.y = player_y;
        let _ = npc.save(&person_dir, &npc_id);
        trigger_output(OutputZone::Main, &format!("{} 跟隨你移動", npc.name));
    }
}

/// 更新玩家與 NPC 的距離，輸出「往這邊走來／離開了」與見面語。
///
/// 對應終端版的 `check_and_handle_proximity`。`player_just_moved` 用來區分是
/// 玩家走過去還是 NPC 走過來，訊息文字不同。
pub fn report_proximity(game_world: &mut GameWorld, player_just_moved: bool) -> bool {
    let controlled_id = game_world.current_controlled_id.clone();
    let Some(controlled) = game_world.npc_manager.get_npc(&controlled_id) else {
        return false;
    };
    let (x, y, map) = (controlled.x, controlled.y, controlled.map.clone());

    let notifications = game_world
        .npc_manager
        .update_proximity(&controlled_id, x, y, &map, player_just_moved);

    if notifications.is_empty() {
        return false;
    }

    // 對話需要拿當前角色當作條件評估對象，先複製一份避免借用衝突
    let controlled_snapshot = game_world.npc_manager.get_npc(&controlled_id).cloned();

    for (npc_id, message, should_greet) in notifications {
        trigger_output(OutputZone::Main, &message);

        if should_greet {
            if let (Some(npc), Some(ref me)) =
                (game_world.npc_manager.get_npc(&npc_id), &controlled_snapshot)
            {
                if let Some(greeting) = npc.get_weighted_dialogue("見面", me) {
                    trigger_output(
                        OutputZone::Main,
                        &format!("{} 說：「{}」", npc.name, greeting),
                    );
                }
            }
        }
    }

    true
}

fn handle_get(game_world: &mut GameWorld, current_id: &str, item_name: Option<String>, quantity: u32) {
    let (x, y) = {
        let me = match game_world.npc_manager.get_npc(current_id) {
            Some(npc) => npc,
            None => {
                trigger_output(OutputZone::Status, "找不到當前控制的角色");
                return;
            }
        };
        (me.x, me.y)
    };
    
    let map_name = game_world.current_map_name.clone();
    
    // 如果沒有指定物品名稱，撿起所有物品
    if item_name.is_none() {
        let mut items_to_get = vec![];
        if let Some(map) = game_world.get_current_map() {
            if let Some(point) = map.get_point(x, y) {
                for (item, count) in &point.objects {
                    items_to_get.push((item.clone(), *count));
                }
            }
        }
        
        if items_to_get.is_empty() {
            trigger_output(OutputZone::Main, "這裡沒有物品");
            return;
        }
        
        for (item, count) in items_to_get {
            if let Some(me) = game_world.npc_manager.get_npc_mut(current_id) {
                *me.items.entry(item.clone()).or_insert(0) += count;
            }
            if let Some(map) = game_world.get_current_map_mut() {
                if let Some(point) = map.get_point_mut(x, y) {
                    point.objects.remove(&item);
                }
            }
            trigger_output(OutputZone::Main, &format!("你撿起了 {} x{}", item, count));
        }
        
        // 地圖只標記為已變動（約 2.4MB/張，不能每個指令都寫），角色直接存
        if let Some(map) = game_world.maps.get_mut(&map_name) {
            map.mark_dirty();
        }
        let person_dir = format!("{}/persons", game_world.world_dir);
        if let Some(me) = game_world.npc_manager.get_npc(current_id) {
            let _ = me.save(&person_dir, &format!("{}.json", current_id));
        }
        return;
    }
    
    // 撿起指定物品
    let item_name = item_name.unwrap();
    let resolved_item = crate::item_registry::resolve_item_name(&item_name);
    
    if let Some(map) = game_world.get_current_map_mut() {
        if let Some(point) = map.get_point_mut(x, y) {
            if let Some(available) = point.objects.get_mut(&resolved_item) {
                let to_get = quantity.min(*available);
                *available -= to_get;
                if *available == 0 {
                    point.objects.remove(&resolved_item);
                }
                
                if let Some(me) = game_world.npc_manager.get_npc_mut(current_id) {
                    *me.items.entry(resolved_item.clone()).or_insert(0) += to_get;
                }
                trigger_output(OutputZone::Main, &format!("你撿起了 {} x{}", resolved_item, to_get));
                
                // 地圖只標記為已變動（約 2.4MB/張，不能每個指令都寫），角色直接存
                if let Some(map) = game_world.maps.get_mut(&map_name) {
                    map.mark_dirty();
                }
                let person_dir = format!("{}/persons", game_world.world_dir);
                if let Some(me) = game_world.npc_manager.get_npc(current_id) {
                    let _ = me.save(&person_dir, &format!("{}.json", current_id));
                }
            } else {
                trigger_output(OutputZone::Status, &format!("這裡沒有 {}", item_name));
            }
        }
    }
}

fn handle_drop(game_world: &mut GameWorld, current_id: &str, item_name: String, quantity: u32) {
    let resolved_item = crate::item_registry::resolve_item_name(&item_name);
    
    let (x, y, to_drop) = {
        let me = match game_world.npc_manager.get_npc_mut(current_id) {
            Some(npc) => npc,
            None => {
                trigger_output(OutputZone::Status, "找不到當前控制的角色");
                return;
            }
        };
        
        if let Some(count) = me.items.get_mut(&resolved_item) {
            let drop_amount = quantity.min(*count);
            *count -= drop_amount;
            if *count == 0 {
                me.items.remove(&resolved_item);
            }
            (me.x, me.y, drop_amount)
        } else {
            trigger_output(OutputZone::Status, &format!("你沒有 {}", item_name));
            return;
        }
    };
    
    // 放到地圖上
    let map_name = game_world.current_map_name.clone();
    if let Some(map) = game_world.get_current_map_mut() {
        if let Some(point) = map.get_point_mut(x, y) {
            *point.objects.entry(resolved_item.clone()).or_insert(0) += to_drop;
        }
    }
    
    trigger_output(OutputZone::Main, &format!("你放下了 {} x{}", resolved_item, to_drop));
    
    // 地圖只標記為已變動（約 2.4MB/張，不能每個指令都寫），角色直接存
    if let Some(map) = game_world.maps.get_mut(&map_name) {
        map.mark_dirty();
    }
    let person_dir = format!("{}/persons", game_world.world_dir);
    if let Some(me) = game_world.npc_manager.get_npc(current_id) {
        let _ = me.save(&person_dir, &format!("{}.json", current_id));
    }
}

fn handle_eat(game_world: &mut GameWorld, current_id: &str, food_name: String) {
    let me = match game_world.npc_manager.get_npc_mut(current_id) {
        Some(npc) => npc,
        None => {
            trigger_output(OutputZone::Status, "找不到當前控制的角色");
            return;
        }
    };
    
    let resolved_food = crate::item_registry::resolve_item_name(&food_name);
    
    if let Some(count) = me.items.get_mut(&resolved_food) {
        if *count > 0 {
            *count -= 1;
            if *count == 0 {
                me.items.remove(&resolved_food);
            }
            
            // 回復 HP
            let heal_amount = 50; // 固定回復量
            me.check_hp(heal_amount);
            
            trigger_output(OutputZone::Main, &format!("你吃了 {}，回復了 {} HP", resolved_food, heal_amount));
            trigger_output(OutputZone::Status, &format!("HP: {}/{}", me.hp, me.max_hp));
            
            // 保存角色
            let person_dir = format!("{}/persons", game_world.world_dir);
            let _ = me.save(&person_dir, &format!("{}.json", current_id));
        }
    } else {
        trigger_output(OutputZone::Status, &format!("你沒有 {}", food_name));
    }
}

fn handle_summon(game_world: &mut GameWorld, current_id: &str, npc_name: String) {
    let me_pos = if let Some(me) = game_world.npc_manager.get_npc(current_id) {
        (me.x, me.y)
    } else {
        trigger_output(OutputZone::Status, "找不到當前控制的角色");
        return;
    };
    
    // 查找 NPC
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
        npc.x = me_pos.0;
        npc.y = me_pos.1;
        trigger_output(OutputZone::Main, &format!("{} 被召喚到了這裡", npc.name));
        
        // 保存 NPC
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = npc.save(&person_dir, &format!("{}.json", npc_name));
    } else {
        trigger_output(OutputZone::Status, &format!("找不到名為 {} 的 NPC", npc_name));
    }
}

fn handle_list_npcs(game_world: &GameWorld) {
    trigger_output(OutputZone::Main, "=== 所有 NPC ===");
    for npc in game_world.npc_manager.get_all_npcs() {
        trigger_output(OutputZone::Main, &format!("  {} 在 ({}, {}) - HP:{}/{}", 
            npc.name, npc.x, npc.y, npc.hp, npc.max_hp));
    }
}

fn handle_check_npc(game_world: &GameWorld, npc_name: String) {
    if let Some(npc) = game_world.npc_manager.get_npc(&npc_name) {
        trigger_output(OutputZone::Main, &format!("=== {} ===", npc.name));
        trigger_output(OutputZone::Main, &format!("位置: ({}, {})", npc.x, npc.y));
        trigger_output(OutputZone::Main, &format!("HP: {}/{}", npc.hp, npc.max_hp));
        trigger_output(OutputZone::Main, &format!("力量: {}", npc.strength));
        trigger_output(OutputZone::Main, &format!("知識: {}", npc.knowledge));
        trigger_output(OutputZone::Main, &format!("社交: {}", npc.sociality));
        
        if !npc.items.is_empty() {
            trigger_output(OutputZone::Main, "\n物品:");
            for (item, count) in &npc.items {
                trigger_output(OutputZone::Main, &format!("  {} x{}", item, count));
            }
        }
    } else {
        trigger_output(OutputZone::Status, &format!("找不到名為 {} 的 NPC", npc_name));
    }
}

fn handle_show_world(game_world: &GameWorld) {
    trigger_output(OutputZone::Main, &format!("=== {} ===", game_world.metadata.name));
    trigger_output(OutputZone::Main, &game_world.metadata.description);
    trigger_output(OutputZone::Main, &format!("\n時間: {}", game_world.format_time()));
    trigger_output(OutputZone::Main, &format!("地圖數量: {}", game_world.maps.len()));
    trigger_output(OutputZone::Main, &format!("NPC 數量: {}", game_world.npc_manager.get_all_npcs().len()));
}

fn handle_switch_control(game_world: &mut GameWorld, npc_name: String) {
    if game_world.npc_manager.get_npc(&npc_name).is_some() {
        game_world.current_controlled_id = npc_name.clone();
        trigger_output(OutputZone::Main, &format!("現在控制 {}", npc_name));
    } else {
        trigger_output(OutputZone::Status, &format!("找不到名為 {} 的 NPC", npc_name));
    }
}

fn handle_conquer(game_world: &mut GameWorld, current_id: &str, direction: String) {
    let (x, y) = if let Some(me) = game_world.npc_manager.get_npc(current_id) {
        (me.x, me.y)
    } else {
        trigger_output(OutputZone::Status, "找不到當前控制的角色");
        return;
    };
    
    let (dx, dy) = match direction.as_str() {
        "up" | "u" | "north" | "n" | "北" => (0, -1),
        "down" | "d" | "south" | "s" | "南" => (0, 1),
        "left" | "l" | "west" | "w" | "西" => (-1, 0),
        "right" | "r" | "east" | "e" | "東" => (1, 0),
        _ => {
            trigger_output(OutputZone::Status, "無效的方向");
            return;
        }
    };
    
    let new_x = (x as i32 + dx) as usize;
    let new_y = (y as i32 + dy) as usize;
    
    let map_name = game_world.current_map_name.clone();
    if let Some(map) = game_world.get_current_map_mut() {
        if let Some(point) = map.get_point_mut(new_x, new_y) {
            point.walkable = true;
            trigger_output(OutputZone::Main, &format!("你征服了 {} 方向，現在可以通行了", direction));
            
            // 地圖只標記為已變動，實際寫檔交給節流存檔
            if let Some(map) = game_world.maps.get_mut(&map_name) {
                map.mark_dirty();
            }
        } else {
            trigger_output(OutputZone::Status, "該位置超出地圖範圍");
        }
    }
}

fn handle_flyto(game_world: &mut GameWorld, current_id: &str, target: String) {
    // 嘗試解析為坐標 (x,y)
    if let Some((x, y)) = parse_coordinates(&target) {
        if let Some(me) = game_world.npc_manager.get_npc_mut(current_id) {
            me.x = x;
            me.y = y;
            trigger_output(OutputZone::Main, &format!("你傳送到了 ({}, {})", x, y));
            
            let person_dir = format!("{}/persons", game_world.world_dir);
            let _ = me.save(&person_dir, &format!("{}.json", current_id));
        }
        return;
    }
    
    // 嘗試作為地圖名稱
    if game_world.maps.contains_key(&target) {
        game_world.current_map_name = target.clone();
        trigger_output(OutputZone::Main, &format!("你傳送到了地圖 {}", target));
        return;
    }
    
    trigger_output(OutputZone::Status, &format!("無法傳送到 {}", target));
}

fn parse_coordinates(s: &str) -> Option<(usize, usize)> {
    // 解析 "x,y" 或 "(x,y)" 格式
    let s = s.trim().trim_matches(|c| c == '(' || c == ')');
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() == 2 {
        if let (Ok(x), Ok(y)) = (parts[0].trim().parse(), parts[1].trim().parse()) {
            return Some((x, y));
        }
    }
    None
}

fn handle_namehere(game_world: &mut GameWorld, current_id: &str, name: String) {
    let (x, y) = if let Some(me) = game_world.npc_manager.get_npc(current_id) {
        (me.x, me.y)
    } else {
        trigger_output(OutputZone::Status, "找不到當前控制的角色");
        return;
    };
    
    let map_name = game_world.current_map_name.clone();
    if let Some(map) = game_world.get_current_map_mut() {
        if let Some(point) = map.get_point_mut(x, y) {
            point.name = name.clone();
            trigger_output(OutputZone::Main, &format!("你將這裡命名為「{}」", name));
            
            // 地圖只標記為已變動，實際寫檔交給節流存檔
            if let Some(map) = game_world.maps.get_mut(&map_name) {
                map.mark_dirty();
            }
        }
    }
}

fn handle_name(game_world: &mut GameWorld, _current_id: &str, target: String, name: String) {
    // 嘗試作為 NPC 名稱
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&target) {
        let old_name = npc.name.clone();
        npc.name = name.clone();
        trigger_output(OutputZone::Main, &format!("你將 {} 重命名為 {}", old_name, name));
        
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = npc.save(&person_dir, &format!("{}.json", target));
        return;
    }
    
    trigger_output(OutputZone::Status, &format!("找不到 {}", target));
}

fn handle_destroy(game_world: &mut GameWorld, current_id: &str, target: String) {
    // 嘗試刪除 NPC
    if game_world.npc_manager.get_npc(&target).is_some() {
        game_world.npc_manager.remove_npc(&target);
        trigger_output(OutputZone::Main, &format!("你刪除了 NPC {}", target));
        
        // 刪除檔案
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = std::fs::remove_file(format!("{}/{}.json", person_dir, target));
        return;
    }
    
    // 嘗試刪除當前位置的物品
    let (x, y) = if let Some(me) = game_world.npc_manager.get_npc(current_id) {
        (me.x, me.y)
    } else {
        trigger_output(OutputZone::Status, "找不到當前控制的角色");
        return;
    };
    
    let resolved_item = crate::item_registry::resolve_item_name(&target);
    let map_name = game_world.current_map_name.clone();
    if let Some(map) = game_world.get_current_map_mut() {
        if let Some(point) = map.get_point_mut(x, y) {
            if point.objects.remove(&resolved_item).is_some() {
                trigger_output(OutputZone::Main, &format!("你刪除了物品 {}", target));
                
                // 地圖只標記為已變動，實際寫檔交給節流存檔
                if let Some(map) = game_world.maps.get_mut(&map_name) {
                    map.mark_dirty();
                }
                return;
            }
        }
    }
    
    trigger_output(OutputZone::Status, &format!("找不到 {}", target));
}

fn handle_create(game_world: &mut GameWorld, current_id: &str, obj_type: String, item_type: String, name: Option<String>) {
    let (x, y) = if let Some(me) = game_world.npc_manager.get_npc(current_id) {
        (me.x, me.y)
    } else {
        trigger_output(OutputZone::Status, "找不到當前控制的角色");
        return;
    };
    
    match obj_type.as_str() {
        "npc" => {
            let npc_name = name.unwrap_or_else(|| item_type.clone());
            let mut npc = crate::person::Person::new(npc_name.clone(), "".to_string());
            npc.x = x;
            npc.y = y;
            npc.items.insert("金幣".to_string(), 10000);
            
            let npc_id = npc_name.clone();
            game_world.npc_manager.add_npc(npc_id.clone(), npc, vec![]);
            trigger_output(OutputZone::Main, &format!("你創建了 NPC「{}」", npc_name));
            
            // 保存 NPC
            let person_dir = format!("{}/persons", game_world.world_dir);
            if let Some(npc) = game_world.npc_manager.get_npc(&npc_id) {
                let _ = npc.save(&person_dir, &format!("{}.json", npc_id));
            }
        },
        "item" => {
            let item_name = crate::item_registry::resolve_item_name(&item_type);
            let map_name = game_world.current_map_name.clone();
            
            if let Some(map) = game_world.get_current_map_mut() {
                if let Some(point) = map.get_point_mut(x, y) {
                    *point.objects.entry(item_name.clone()).or_insert(0) += 1;
                    trigger_output(OutputZone::Main, &format!("你創建了物品「{}」", item_name));
                    
                    // 地圖只標記為已變動，實際寫檔交給節流存檔
                    if let Some(map) = game_world.maps.get_mut(&map_name) {
                        map.mark_dirty();
                    }
                }
            }
        },
        _ => {
            trigger_output(OutputZone::Status, &format!("未知類型: {}，請使用 item 或 npc", obj_type));
        }
    }
}

fn handle_set(game_world: &mut GameWorld, current_id: &str, target: String, attribute: String, value: i32) {
    // 檢查是否為設置物品價格
    if target.to_lowercase() == "item" {
        let price = value.max(0) as u32;
        crate::trade::TradeSystem::set_item_price(&attribute, price);
        trigger_output(OutputZone::Main, &format!("物品「{}」的價格設置為 {} 金幣", attribute, price));
        return;
    }
    
    // 設置 me 的屬性
    let is_me = target.to_lowercase() == "me" || target == "我";
    if is_me {
        if let Some(me) = game_world.npc_manager.get_npc_mut(current_id) {
            set_person_attribute(me, &attribute, value);
            
            let person_dir = format!("{}/persons", game_world.world_dir);
            let _ = me.save(&person_dir, &format!("{}.json", current_id));
        }
        return;
    }
    
    // 設置 NPC 的屬性
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&target) {
        set_person_attribute(npc, &attribute, value);
        
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = npc.save(&person_dir, &format!("{}.json", target));
    } else {
        trigger_output(OutputZone::Status, &format!("找不到 {}", target));
    }
}

fn set_person_attribute(person: &mut Person, attribute: &str, value: i32) {
    match attribute.to_lowercase().as_str() {
        "hp" => {
            person.set_hp(value);
            trigger_output(OutputZone::Main, &format!("{} 的 HP 設置為 {}", person.name, value));
        },
        "mp" => {
            person.mp = value;
            trigger_output(OutputZone::Main, &format!("{} 的 MP 設置為 {}", person.name, value));
        },
        "strength" | "str" | "力量" => {
            person.set_strength(value);
            trigger_output(OutputZone::Main, &format!("{} 的力量設置為 {}", person.name, value));
        },
        "knowledge" | "kno" | "知識" => {
            person.knowledge = value;
            trigger_output(OutputZone::Main, &format!("{} 的知識設置為 {}", person.name, value));
        },
        "sociality" | "soc" | "交誼" => {
            person.sociality = value;
            trigger_output(OutputZone::Main, &format!("{} 的交誼設置為 {}", person.name, value));
        },
        "gold" | "金幣" | "goldcoin" => {
            let gold_value = value.max(0) as u32;
            person.items.insert("金幣".to_string(), gold_value);
            trigger_output(OutputZone::Main, &format!("{} 的金幣設置為 {}", person.name, gold_value));
        },
        _ => {
            trigger_output(OutputZone::Status, &format!("未知屬性: {}", attribute));
        }
    }
}

fn handle_give(game_world: &mut GameWorld, current_id: &str, npc_name: String, item: String, quantity: u32) {
    let resolved_item = crate::item_registry::resolve_item_name(&item);
    
    // 從 me 移除物品
    let to_give = {
        let me = match game_world.npc_manager.get_npc_mut(current_id) {
            Some(npc) => npc,
            None => {
                trigger_output(OutputZone::Status, "找不到當前控制的角色");
                return;
            }
        };
        
        if let Some(count) = me.items.get_mut(&resolved_item) {
            let give_amount = quantity.min(*count);
            *count -= give_amount;
            if *count == 0 {
                me.items.remove(&resolved_item);
            }
            give_amount
        } else {
            trigger_output(OutputZone::Status, &format!("你沒有 {}", item));
            return;
        }
    };
    
    // 給 NPC
    if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_name) {
        *npc.items.entry(resolved_item.clone()).or_insert(0) += to_give;
        trigger_output(OutputZone::Main, &format!("你給了 {} {} x{}", npc_name, resolved_item, to_give));
        
        // 保存兩個角色
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = npc.save(&person_dir, &format!("{}.json", npc_name));
        
        // 保存 me
        if let Some(me) = game_world.npc_manager.get_npc(current_id) {
            let _ = me.save(&person_dir, &format!("{}.json", current_id));
        }
    } else {
        trigger_output(OutputZone::Status, &format!("找不到名為 {} 的 NPC", npc_name));
    }
}

fn handle_use_item(game_world: &mut GameWorld, current_id: &str, item_name: String) {
    let me = match game_world.npc_manager.get_npc_mut(current_id) {
        Some(npc) => npc,
        None => {
            trigger_output(OutputZone::Status, "找不到當前控制的角色");
            return;
        }
    };
    
    let resolved_item = crate::item_registry::resolve_item_name(&item_name);
    
    if let Some(count) = me.items.get_mut(&resolved_item) {
        if *count > 0 {
            *count -= 1;
            if *count == 0 {
                me.items.remove(&resolved_item);
            }
            
            // 使用物品效果（簡化版）
            let heal_amount = 50;
            me.check_hp(heal_amount);
            
            trigger_output(OutputZone::Main, &format!("你使用了 {}，回復了 {} HP", resolved_item, heal_amount));
            trigger_output(OutputZone::Status, &format!("HP: {}/{}", me.hp, me.max_hp));
            
            let person_dir = format!("{}/persons", game_world.world_dir);
            let _ = me.save(&person_dir, &format!("{}.json", current_id));
        }
    } else {
        trigger_output(OutputZone::Status, &format!("你沒有 {}", item_name));
    }
}

fn handle_sleep(game_world: &mut GameWorld, current_id: &str) {
    if let Some(me) = game_world.npc_manager.get_npc_mut(current_id) {
        me.is_sleeping = true;
        trigger_output(OutputZone::Main, "你進入了睡眠狀態...");
        trigger_output(OutputZone::Status, "使用 'wakeup' 或 'wake' 醒來，使用 'dream [內容]' 做夢");
        
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = me.save(&person_dir, &format!("{}.json", current_id));
    }
}

fn handle_dream(game_world: &mut GameWorld, current_id: &str, content: Option<String>) {
    if let Some(me) = game_world.npc_manager.get_npc(current_id) {
        if me.is_sleeping {
            let dream_text = content.unwrap_or_else(|| "一場美好的夢境...".to_string());
            trigger_output(OutputZone::Main, &format!("💭 {}", dream_text));
        } else {
            trigger_output(OutputZone::Status, "你需要先睡覺才能做夢！使用 sleep 指令進入睡眠");
        }
    }
}

fn handle_wakeup(game_world: &mut GameWorld, current_id: &str) {
    if let Some(me) = game_world.npc_manager.get_npc_mut(current_id) {
        if me.is_sleeping {
            me.is_sleeping = false;
            trigger_output(OutputZone::Main, "你醒來了！");
            
            let person_dir = format!("{}/persons", game_world.world_dir);
            let _ = me.save(&person_dir, &format!("{}.json", current_id));
        } else {
            trigger_output(OutputZone::Status, "你還沒睡覺呢！");
        }
    }
}
