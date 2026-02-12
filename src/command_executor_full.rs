/// 命令執行器 - 無 UI 模式的命令執行邏輯
/// 這個模組提供了所有遊戲命令的執行邏輯，使用 CoreOutputManager 進行輸出
/// FFI 模式和其他無 UI 模式都可以使用這個模組

use crate::command_handler::CommandResult;
use crate::world::GameWorld;
use crate::person::Person;
use crate::core_output::{OutputZone, trigger_output};

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
        // UI 相關命令（在無 UI 模式中忽略）
        CommandResult::ShowMinimap | CommandResult::HideMinimap |
        CommandResult::ShowLog | CommandResult::HideLog |
        CommandResult::ShowMap | CommandResult::ToggleTypewriter |
        CommandResult::AddToSide(_) | CommandResult::ShowHistory(_) => {
            trigger_output(OutputZone::Log, "此命令僅在終端 UI 模式可用");
            true
        },
        _ => {
            trigger_output(OutputZone::Log, &format!("命令: {} (功能尚未實現)", command));
            true
        }
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

fn handle_look(game_world: &GameWorld, current_id: &str, _target: Option<String>) {
    // 獲取當前角色位置
    let (x, y) = if let Some(me) = game_world.npc_manager.get_npc(current_id) {
        (me.x, me.y)
    } else {
        trigger_output(OutputZone::Status, "找不到當前控制的角色");
        return;
    };
    
    // 顯示當前位置資訊
    if let Some(map) = game_world.get_current_map() {
        trigger_output(OutputZone::Main, &format!("📍 {}", map.name));
        trigger_output(OutputZone::Main, &map.description);
        trigger_output(OutputZone::Main, &format!("你在 ({}, {})", x, y));
        
        // 顯示當前位置的物品
        if let Some(point) = map.get_point(x, y) {
            if !point.objects.is_empty() {
                trigger_output(OutputZone::Main, "\n這裡有：");
                for (item, count) in &point.objects {
                    trigger_output(OutputZone::Main, &format!("  {} x{}", item, count));
                }
            }
        }
    }
}

fn handle_move(game_world: &mut GameWorld, current_id: &str, dx: i32, dy: i32) {
    // 獲取當前位置
    let (old_x, old_y) = if let Some(me) = game_world.npc_manager.get_npc(current_id) {
        (me.x, me.y)
    } else {
        trigger_output(OutputZone::Status, "找不到當前控制的角色");
        return;
    };
    
    let new_x = (old_x as i32 + dx) as usize;
    let new_y = (old_y as i32 + dy) as usize;
    
    // 檢查是否可行走
    let can_walk = if let Some(map) = game_world.get_current_map() {
        if let Some(point) = map.get_point(new_x, new_y) {
            point.walkable
        } else {
            false
        }
    } else {
        false
    };
    
    if can_walk {
        // 更新位置
        if let Some(me) = game_world.npc_manager.get_npc_mut(current_id) {
            me.x = new_x;
            me.y = new_y;
            
            let direction = match (dx, dy) {
                (0, -1) => "北",
                (0, 1) => "南",
                (1, 0) => "東",
                (-1, 0) => "西",
                _ => "未知方向",
            };
            trigger_output(OutputZone::Main, &format!("你向{}移動到 ({}, {})", direction, new_x, new_y));
            
            // 保存角色位置
            let person_dir = format!("{}/persons", game_world.world_dir);
            let _ = me.save(&person_dir, &format!("{}.json", current_id));
        }
    } else {
        trigger_output(OutputZone::Status, "那個方向無法通行");
    }
}

fn handle_get(game_world: &mut GameWorld, current_id: &str, item_name: Option<String>, quantity: u32) {
    let me = match game_world.npc_manager.get_npc_mut(current_id) {
        Some(npc) => npc,
        None => {
            trigger_output(OutputZone::Status, "找不到當前控制的角色");
            return;
        }
    };
    
    let (x, y) = (me.x, me.y);
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
            *me.items.entry(item.clone()).or_insert(0) += count;
            if let Some(map) = game_world.get_current_map_mut() {
                if let Some(point) = map.get_point_mut(x, y) {
                    point.objects.remove(&item);
                }
            }
            trigger_output(OutputZone::Main, &format!("你撿起了 {} x{}", item, count));
        }
        
        // 保存地圖和角色
        if let Some(map) = game_world.maps.get(&map_name) {
            let _ = game_world.save_map(map);
        }
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = me.save(&person_dir, &format!("{}.json", current_id));
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
                
                *me.items.entry(resolved_item.clone()).or_insert(0) += to_get;
                trigger_output(OutputZone::Main, &format!("你撿起了 {} x{}", resolved_item, to_get));
                
                // 保存地圖和角色
                if let Some(map) = game_world.maps.get(&map_name) {
                    let _ = game_world.save_map(map);
                }
                let person_dir = format!("{}/persons", game_world.world_dir);
                let _ = me.save(&person_dir, &format!("{}.json", current_id));
            } else {
                trigger_output(OutputZone::Status, &format!("這裡沒有 {}", item_name));
            }
        }
    }
}

fn handle_drop(game_world: &mut GameWorld, current_id: &str, item_name: String, quantity: u32) {
    let me = match game_world.npc_manager.get_npc_mut(current_id) {
        Some(npc) => npc,
        None => {
            trigger_output(OutputZone::Status, "找不到當前控制的角色");
            return;
        }
    };
    
    let resolved_item = crate::item_registry::resolve_item_name(&item_name);
    
    if let Some(count) = me.items.get_mut(&resolved_item) {
        let to_drop = quantity.min(*count);
        *count -= to_drop;
        if *count == 0 {
            me.items.remove(&resolved_item);
        }
        
        // 放到地圖上
        let (x, y) = (me.x, me.y);
        let map_name = game_world.current_map_name.clone();
        if let Some(map) = game_world.get_current_map_mut() {
            if let Some(point) = map.get_point_mut(x, y) {
                *point.objects.entry(resolved_item.clone()).or_insert(0) += to_drop;
            }
        }
        
        trigger_output(OutputZone::Main, &format!("你放下了 {} x{}", resolved_item, to_drop));
        
        // 保存地圖和角色
        if let Some(map) = game_world.maps.get(&map_name) {
            let _ = game_world.save_map(map);
        }
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = me.save(&person_dir, &format!("{}.json", current_id));
    } else {
        trigger_output(OutputZone::Status, &format!("你沒有 {}", item_name));
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
            me.heal(heal_amount);
            
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
    let npc_id = game_world.npc_manager.find_npc_by_name(&npc_name);
    if let Some(id) = npc_id {
        if let Some(npc) = game_world.npc_manager.get_npc_mut(&id) {
            npc.x = me_pos.0;
            npc.y = me_pos.1;
            trigger_output(OutputZone::Main, &format!("{} 被召喚到了這裡", npc.name));
            
            // 保存 NPC
            let person_dir = format!("{}/persons", game_world.world_dir);
            let _ = npc.save(&person_dir, &format!("{}.json", id));
        }
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
    let npc_id = game_world.npc_manager.find_npc_by_name(&npc_name);
    if let Some(id) = npc_id {
        if let Some(npc) = game_world.npc_manager.get_npc(&id) {
            trigger_output(OutputZone::Main, &format!("=== {} ===", npc.name));
            trigger_output(OutputZone::Main, &format!("位置: ({}, {})", npc.x, npc.y));
            trigger_output(OutputZone::Main, &format!("HP: {}/{}", npc.hp, npc.max_hp));
            trigger_output(OutputZone::Main, &format!("等級: {}", npc.level));
            trigger_output(OutputZone::Main, &format!("力量: {}", npc.strength));
            trigger_output(OutputZone::Main, &format!("知識: {}", npc.knowledge));
            trigger_output(OutputZone::Main, &format!("社交: {}", npc.sociality));
            
            if !npc.items.is_empty() {
                trigger_output(OutputZone::Main, "\n物品:");
                for (item, count) in &npc.items {
                    trigger_output(OutputZone::Main, &format!("  {} x{}", item, count));
                }
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
    let npc_id = game_world.npc_manager.find_npc_by_name(&npc_name);
    if let Some(id) = npc_id {
        game_world.current_controlled_id = id.clone();
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
            
            // 保存地圖
            if let Some(map) = game_world.maps.get(&map_name) {
                let _ = game_world.save_map(map);
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
            
            // 保存地圖
            if let Some(map) = game_world.maps.get(&map_name) {
                let _ = game_world.save_map(map);
            }
        }
    }
}

fn handle_name(game_world: &mut GameWorld, _current_id: &str, target: String, name: String) {
    // 嘗試作為 NPC 名稱
    let npc_id = game_world.npc_manager.find_npc_by_name(&target);
    if let Some(id) = npc_id {
        if let Some(npc) = game_world.npc_manager.get_npc_mut(&id) {
            let old_name = npc.name.clone();
            npc.name = name.clone();
            trigger_output(OutputZone::Main, &format!("你將 {} 重命名為 {}", old_name, name));
            
            let person_dir = format!("{}/persons", game_world.world_dir);
            let _ = npc.save(&person_dir, &format!("{}.json", id));
            return;
        }
    }
    
    trigger_output(OutputZone::Status, &format!("找不到 {}", target));
}

fn handle_destroy(game_world: &mut GameWorld, current_id: &str, target: String) {
    // 嘗試刪除 NPC
    let npc_id = game_world.npc_manager.find_npc_by_name(&target);
    if let Some(id) = npc_id {
        game_world.npc_manager.remove_npc(&id);
        trigger_output(OutputZone::Main, &format!("你刪除了 NPC {}", target));
        
        // 刪除檔案
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = std::fs::remove_file(format!("{}/{}.json", person_dir, id));
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
                
                // 保存地圖
                if let Some(map) = game_world.maps.get(&map_name) {
                    let _ = game_world.save_map(map);
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
            let mut npc = Person::new(npc_name.clone(), x, y);
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
                    
                    // 保存地圖
                    if let Some(map) = game_world.maps.get(&map_name) {
                        let _ = game_world.save_map(map);
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
    let npc_id = game_world.npc_manager.find_npc_by_name(&target);
    if let Some(id) = npc_id {
        if let Some(npc) = game_world.npc_manager.get_npc_mut(&id) {
            set_person_attribute(npc, &attribute, value);
            
            let person_dir = format!("{}/persons", game_world.world_dir);
            let _ = npc.save(&person_dir, &format!("{}.json", id));
        }
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
    // 查找 NPC
    let npc_id = game_world.npc_manager.find_npc_by_name(&npc_name);
    let npc_id = match npc_id {
        Some(id) => id,
        None => {
            trigger_output(OutputZone::Status, &format!("找不到名為 {} 的 NPC", npc_name));
            return;
        }
    };
    
    let resolved_item = crate::item_registry::resolve_item_name(&item);
    
    // 從 me 移除物品
    let me = match game_world.npc_manager.get_npc_mut(current_id) {
        Some(npc) => npc,
        None => {
            trigger_output(OutputZone::Status, "找不到當前控制的角色");
            return;
        }
    };
    
    if let Some(count) = me.items.get_mut(&resolved_item) {
        let to_give = quantity.min(*count);
        *count -= to_give;
        if *count == 0 {
            me.items.remove(&resolved_item);
        }
        
        // 給 NPC
        if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) {
            *npc.items.entry(resolved_item.clone()).or_insert(0) += to_give;
            trigger_output(OutputZone::Main, &format!("你給了 {} {} x{}", npc_name, resolved_item, to_give));
            
            // 保存兩個角色
            let person_dir = format!("{}/persons", game_world.world_dir);
            let _ = npc.save(&person_dir, &format!("{}.json", npc_id));
        }
        
        // 保存 me
        let person_dir = format!("{}/persons", game_world.world_dir);
        if let Some(me) = game_world.npc_manager.get_npc(current_id) {
            let _ = me.save(&person_dir, &format!("{}.json", current_id));
        }
    } else {
        trigger_output(OutputZone::Status, &format!("你沒有 {}", item));
    }
}
