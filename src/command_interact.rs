//! NPC 互動 / 交易 / 任務 —— UI 無關的共用實作
//!
//! 這些規則以前只寫在 `app.rs`（終端 UI）裡，`command_executor` 對它們一律回
//! 「此功能需要在終端 UI 模式中使用」，所以 iOS（FFI）版根本無法跟 NPC 對話、
//! 交易或接任務。搬到這裡之後兩個 client 走同一套邏輯。
//!
//! 輸出一律透過 `core_output::trigger_output`，不依賴 ratatui/crossterm。

use crate::core_output::{trigger_output, OutputZone};
use crate::person::Person;
use crate::world::GameWorld;

// =================================================================
// 對話 / 叫住 / 組隊
// =================================================================

/// 與 NPC 對話（需在同一格）
pub fn handle_talk(game_world: &mut GameWorld, current_id: &str, npc_name: &str, topic: &str) {
    let Some(me) = game_world.npc_manager.get_npc(current_id) else {
        trigger_output(OutputZone::Status, "無法取得當前角色資訊");
        return;
    };
    let (x, y) = (me.x, me.y);

    // 只能跟同一格的 NPC 說話。用 ID 而不是名稱當 key：世界裡有同名 NPC
    // （`merchant` 和 `商人` 的 name 都是「商人」），拿名稱會存到別人的檔案。
    let npc_here = game_world
        .npc_manager
        .get_npcs_with_ids_at_in_map(&game_world.current_map_name, x, y)
        .into_iter()
        .find(|(id, n)| {
            id != current_id
                && (id.to_lowercase() == npc_name.to_lowercase()
                    || n.name.to_lowercase() == npc_name.to_lowercase())
        })
        .map(|(id, npc)| (id, npc.clone()));

    let Some((npc_id, npc)) = npc_here else {
        trigger_output(OutputZone::Status, &format!("此處找不到 {npc_name}"));
        return;
    };

    let Some(me) = game_world.npc_manager.get_npc(current_id) else { return };

    if let Some(dialogue) = npc.try_talk(topic, me) {
        trigger_output(OutputZone::Main, &format!("💬 跟{}開始{topic}...", npc.name));
        trigger_output(OutputZone::Main, &format!("{} 說：「{}」", npc.name, dialogue));
    } else {
        trigger_output(
            OutputZone::Main,
            &format!("{} 對「{}」這個話題似乎不想說話。", npc.name, topic),
        );
    }

    // 對話算一次互動，累計次數後存檔（終端版靠離開遊戲時的 save_all 落地，
    // iOS 隨時可能被系統終止，所以這裡就寫回去）
    let person_dir = format!("{}/persons", game_world.world_dir);
    if let Some(npc_mut) = game_world.npc_manager.get_npc_mut(&npc_id) {
        npc_mut.interaction_count += 1;
        let _ = npc_mut.save(&person_dir, &npc_id);
    }
}

/// 叫住單一 NPC，成功率取決於好感度。回傳是否叫住。
fn try_stop_npc(npc: &mut Person) -> bool {
    use rand::Rng;

    let success_rate = (50 + npc.relationship / 2).clamp(0, 100);
    let roll = rand::thread_rng().gen_range(0..100);

    if roll < success_rate {
        npc.is_interacting = true;
        trigger_output(OutputZone::Main, &format!("你叫住了 {}", npc.name));

        // 與終端版一致：條件評估對象就是 NPC 自己
        let self_copy = npc.clone();
        if let Some(response) = npc.get_weighted_dialogue("被叫住", &self_copy) {
            trigger_output(OutputZone::Main, &format!("{} 說：「{}」", npc.name, response));
        }
        true
    } else {
        trigger_output(OutputZone::Main, &format!("{} 沒有理會你", npc.name));
        false
    }
}

/// `wait [npc]` —— 叫住指定 NPC；名稱留空則叫住同格所有 NPC
pub fn handle_wait(game_world: &mut GameWorld, current_id: &str, npc_name: &str) {
    let Some(me) = game_world.npc_manager.get_npc(current_id) else {
        trigger_output(OutputZone::Status, "無法取得當前角色資訊");
        return;
    };
    let (x, y) = (me.x, me.y);
    let person_dir = format!("{}/persons", game_world.world_dir);

    if npc_name.is_empty() {
        let ids: Vec<String> = game_world
            .npc_manager
            .get_npcs_with_ids_at_in_map(&game_world.current_map_name, x, y)
            .into_iter()
            .filter(|(id, _)| id != current_id)
            .map(|(id, _)| id)
            .collect();

        if ids.is_empty() {
            trigger_output(OutputZone::Main, "此處沒有 NPC");
            return;
        }

        let total = ids.len();
        let mut success = 0;
        for id in ids {
            if let Some(npc) = game_world.npc_manager.get_npc_mut(&id) {
                if try_stop_npc(npc) {
                    success += 1;
                }
                let _ = npc.save(&person_dir, &id);
            }
        }
        trigger_output(OutputZone::Status, &format!("叫住了 {success}/{total} 個 NPC"));
        return;
    }

    // 指定 NPC：必須同地圖且距離 <= 1
    let current_map = game_world.current_map_name.clone();
    let Some(npc_id) = game_world.npc_manager.resolve_id(npc_name) else {
        trigger_output(OutputZone::Status, &format!("找不到 {npc_name}"));
        return;
    };
    let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) else { return };

    if npc.map != current_map {
        trigger_output(OutputZone::Main, &format!("{} 不在這個地圖", npc.name));
        return;
    }
    if (npc.x as i32 - x as i32).abs() + (npc.y as i32 - y as i32).abs() > 1 {
        trigger_output(OutputZone::Main, &format!("{} 距離太遠，無法叫住", npc.name));
        return;
    }

    try_stop_npc(npc);
    let _ = npc.save(&person_dir, &npc_id);
}

/// `party <npc>` —— 邀請 NPC 組隊
pub fn handle_party(game_world: &mut GameWorld, current_id: &str, npc_name: &str) {
    let Some(me) = game_world.npc_manager.get_npc(current_id) else {
        trigger_output(OutputZone::Status, "無法取得當前角色資訊");
        return;
    };
    let (x, y) = (me.x, me.y);
    let current_map = game_world.current_map_name.clone();
    let person_dir = format!("{}/persons", game_world.world_dir);

    let Some(npc_id) = game_world.npc_manager.resolve_id(npc_name) else {
        trigger_output(OutputZone::Status, &format!("找不到 {npc_name}"));
        return;
    };
    let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) else { return };

    if npc.map != current_map {
        trigger_output(OutputZone::Main, &format!("{} 不在這個地圖", npc.name));
        return;
    }
    if (npc.x as i32 - x as i32).abs() + (npc.y as i32 - y as i32).abs() > 1 {
        trigger_output(OutputZone::Main, &format!("{} 距離太遠，無法組隊", npc.name));
        return;
    }
    if npc.party_leader.is_some() {
        trigger_output(OutputZone::Main, &format!("{} 已經在隊伍中了", npc.name));
        return;
    }

    npc.party_leader = Some("me".to_string());
    trigger_output(OutputZone::Main, &format!("{} 加入了你的隊伍", npc.name));
    let _ = npc.save(&person_dir, &npc_id);
}

/// `disband` —— 解散隊伍
pub fn handle_disband(game_world: &mut GameWorld) {
    let person_dir = format!("{}/persons", game_world.world_dir);
    let mut disbanded = 0;

    for npc_id in game_world.npc_manager.get_all_npc_ids() {
        let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) else { continue };
        if npc.party_leader.as_deref() == Some("me") {
            npc.party_leader = None;
            disbanded += 1;
            trigger_output(OutputZone::Main, &format!("{} 離開了隊伍", npc.name));
            let _ = npc.save(&person_dir, &npc_id);
        }
    }

    if disbanded == 0 {
        trigger_output(OutputZone::Main, "當前沒有隊員");
    } else {
        trigger_output(OutputZone::Main, &format!("已解散隊伍，共 {disbanded} 名隊員離隊"));
    }
}

/// `use <item> on <npc>` —— 對同格 NPC 使用物品
pub fn handle_use_item_on(
    game_world: &mut GameWorld,
    current_id: &str,
    item_name: &str,
    npc_name: &str,
) {
    let resolved_item = crate::item_registry::resolve_item_name(item_name);

    let Some(me) = game_world.npc_manager.get_npc(current_id) else {
        trigger_output(OutputZone::Status, "找不到當前控制的角色");
        return;
    };
    let (x, y) = (me.x, me.y);

    // 目標必須在同一格
    let target_id = game_world
        .npc_manager
        .get_npcs_with_ids_at_in_map(&game_world.current_map_name, x, y)
        .into_iter()
        .find(|(id, npc)| {
            id.to_lowercase() == npc_name.to_lowercase()
                || npc.name.to_lowercase() == npc_name.to_lowercase()
        })
        .map(|(id, _)| id);

    let Some(target_id) = target_id else {
        trigger_output(OutputZone::Status, &format!("此處找不到 {npc_name}"));
        return;
    };

    // 先扣玩家的物品；扣不到就整個動作取消
    {
        let Some(me) = game_world.npc_manager.get_npc_mut(current_id) else { return };
        match me.items.get_mut(&resolved_item) {
            Some(count) if *count > 0 => {
                *count -= 1;
                if *count == 0 {
                    me.items.remove(&resolved_item);
                }
            }
            _ => {
                trigger_output(OutputZone::Status, &format!("你沒有 {item_name}"));
                return;
            }
        }
    }

    let display_name = crate::item_registry::get_item_display_name(&resolved_item);
    let person_dir = format!("{}/persons", game_world.world_dir);

    if let Some(target) = game_world.npc_manager.get_npc_mut(&target_id) {
        let heal = crate::item_registry::get_food_hp(&resolved_item).unwrap_or(50);
        target.check_hp(heal);
        target.change_relationship(2);
        trigger_output(
            OutputZone::Main,
            &format!(
                "你對 {} 使用了 {}，{} 回復到 {}/{} HP",
                target.name, display_name, target.name, target.hp, target.max_hp
            ),
        );
        let _ = target.save(&person_dir, &target_id);
    }

    if let Some(me) = game_world.npc_manager.get_npc(current_id) {
        let _ = me.save(&person_dir, current_id);
    }
}

// =================================================================
// NPC 對話 / 好感度設定
// =================================================================

/// 設定 NPC 台詞；`conditions` 格式與終端版相同，例如 `hp>50,顏值>=60`
pub fn handle_set_dialogue(
    game_world: &mut GameWorld,
    npc_name: &str,
    topic: &str,
    text: &str,
    conditions: Option<&str>,
) {
    let person_dir = format!("{}/persons", game_world.world_dir);
    let Some(npc_id) = game_world.npc_manager.resolve_id(npc_name) else {
        trigger_output(OutputZone::Status, &format!("找不到 {npc_name}"));
        return;
    };
    let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) else { return };

    match conditions {
        Some(cond_str) => {
            let parsed = parse_dialogue_conditions(cond_str);
            let count = parsed.len();
            let option = crate::person::DialogueOption::with_conditions(text.to_string(), parsed);
            npc.add_dialogue_option(topic.to_string(), option);
            trigger_output(
                OutputZone::Main,
                &format!("已為 {npc_name} 設定「{topic}」台詞（{count} 個條件）：{text}"),
            );
        }
        None => {
            npc.set_dialogue(topic.to_string(), text.to_string());
            trigger_output(
                OutputZone::Main,
                &format!("已為 {npc_name} 設定「{topic}」台詞：{text}"),
            );
        }
    }

    let _ = npc.save(&person_dir, &npc_id);
}

/// 解析條件字串成 `DialogueCondition` 列表，例如 `hp>50,顏值>=60`
fn parse_dialogue_conditions(conditions_str: &str) -> Vec<crate::person::DialogueCondition> {
    let mut conditions = Vec::new();

    for part in conditions_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // 長的運算子要先比對，否則 ">=" 會被 ">" 先吃掉
        for op in [">=", "<=", "!=", ">", "<", "="] {
            if let Some(pos) = part.find(op) {
                let attribute = part[..pos].trim().to_string();
                let value = part[pos + op.len()..].trim().to_string();
                if !attribute.is_empty() && !value.is_empty() {
                    conditions.push(crate::person::DialogueCondition {
                        attribute,
                        operator: op.to_string(),
                        value,
                    });
                }
                break;
            }
        }
    }

    conditions
}

/// 設定 NPC 說話積極度
pub fn handle_set_eagerness(game_world: &mut GameWorld, npc_name: &str, eagerness: u8) {
    let person_dir = format!("{}/persons", game_world.world_dir);
    let Some(npc_id) = game_world.npc_manager.resolve_id(npc_name) else {
        trigger_output(OutputZone::Status, &format!("找不到 {npc_name}"));
        return;
    };
    let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) else { return };

    npc.set_talk_eagerness(eagerness);
    trigger_output(
        OutputZone::Main,
        &format!("{} 的說話積極度設為 {}", npc.name, npc.talk_eagerness),
    );
    let _ = npc.save(&person_dir, &npc_id);
}

/// 設定／調整 NPC 好感度（`is_delta` 為 true 時是增減量）
pub fn handle_set_relationship(
    game_world: &mut GameWorld,
    npc_name: &str,
    value: i32,
    is_delta: bool,
) {
    let person_dir = format!("{}/persons", game_world.world_dir);
    let Some(npc_id) = game_world.npc_manager.resolve_id(npc_name) else {
        trigger_output(OutputZone::Status, &format!("找不到 {npc_name}"));
        return;
    };
    let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) else { return };

    if is_delta {
        npc.change_relationship(value);
    } else {
        npc.relationship = value.clamp(-100, 100);
    }

    trigger_output(
        OutputZone::Main,
        &format!(
            "{} 的好感度：{}（{}）",
            npc.name,
            npc.relationship,
            npc.get_relationship_description()
        ),
    );
    let _ = npc.save(&person_dir, &npc_id);
}

// =================================================================
// 交易系統
// =================================================================

/// `trade <npc>` —— 顯示交易面板（NPC 商品 + 玩家可賣物品）
pub fn handle_trade(game_world: &mut GameWorld, npc_name: &str) {
    use crate::world::InteractionState;

    let Some(resolved) = crate::panel_render::resolve_trade_npc(game_world, npc_name) else {
        trigger_output(OutputZone::Status, &format!("此處找不到 {npc_name}"));
        return;
    };

    // 記錄互動狀態，host 可據此決定要不要開交易畫面
    game_world.interaction_state = InteractionState::Trading { npc_name: resolved.clone() };
    trigger_output(
        OutputZone::Main,
        &crate::panel_render::render_trade(game_world, &resolved),
    );
}

/// 買賣共用流程。回傳是否成功。
pub fn execute_trade(
    game_world: &mut GameWorld,
    npc_name: &str,
    item_name: &str,
    quantity: u32,
    is_buy: bool,
) -> bool {
    use crate::trade::{TradeResult, TradeSystem};

    if quantity == 0 {
        trigger_output(OutputZone::Status, "數量必須大於 0");
        return false;
    }

    let Some(npc_id) = crate::panel_render::resolve_trade_npc(game_world, npc_name) else {
        trigger_output(OutputZone::Status, "這裡沒有可以交易的對象");
        return false;
    };

    let resolved_item = crate::item_registry::resolve_item_name(item_name);
    let result = if is_buy {
        let price = TradeSystem::calculate_buy_price(&resolved_item, quantity);
        TradeSystem::buy_from_npc(game_world, &npc_id, &resolved_item, quantity, price)
    } else {
        let price = TradeSystem::calculate_sell_price(&resolved_item, quantity);
        TradeSystem::sell_to_npc(game_world, &npc_id, &resolved_item, quantity, price)
    };

    let success = match result {
        TradeResult::Success(msg) => {
            trigger_output(OutputZone::Main, &msg);
            true
        }
        TradeResult::Failed(reason) => {
            trigger_output(OutputZone::Status, &reason);
            false
        }
    };

    if success {
        // 交易改動雙方背包，立刻落地（persons 檔案都很小，寫入成本可忽略）
        let person_dir = format!("{}/persons", game_world.world_dir);
        let _ = game_world.npc_manager.save_all(&person_dir);
    }

    success
}

// =================================================================
// 任務系統
// =================================================================

pub fn handle_quest_list(game_world: &GameWorld) {
    trigger_output(OutputZone::Main, "═══ 所有任務 ═══");
    if game_world.quest_manager.quests.is_empty() {
        trigger_output(OutputZone::Main, "  （沒有任何任務）");
        return;
    }
    for quest in game_world.quest_manager.quests.values() {
        trigger_output(
            OutputZone::Main,
            &format!("  [{}]{} - {}", quest.get_status_char(), quest.id, quest.name),
        );
    }
}

/// 列出某個分類的任務（進行中／可接取／已完成共用）
fn print_quest_group(title: &str, quests: Vec<&crate::quest::Quest>, empty_hint: &str) {
    trigger_output(OutputZone::Main, title);
    if quests.is_empty() {
        trigger_output(OutputZone::Main, empty_hint);
        return;
    }
    for quest in quests {
        trigger_output(OutputZone::Main, &format!("  • {} - {}", quest.id, quest.name));
    }
}

pub fn handle_quest_active(game_world: &GameWorld) {
    print_quest_group(
        "═══ 進行中的任務 ═══",
        game_world.quest_manager.get_active_quests(),
        "  沒有進行中的任務。",
    );
}

pub fn handle_quest_available(game_world: &GameWorld) {
    print_quest_group(
        "═══ 可接取的任務 ═══",
        game_world.quest_manager.get_available_quests(),
        "  沒有可接取的任務。",
    );
}

pub fn handle_quest_completed(game_world: &GameWorld) {
    print_quest_group(
        "═══ 已完成的任務 ═══",
        game_world.quest_manager.get_completed_quests(),
        "  尚未完成任何任務。",
    );
}

pub fn handle_quest_info(game_world: &GameWorld, quest_id: &str) {
    match game_world.quest_manager.get_quest(quest_id) {
        Some(quest) => {
            trigger_output(OutputZone::Main, &format!("═══ {} ═══", quest.name));
            trigger_output(OutputZone::Main, &format!("ID: {}", quest.id));
            trigger_output(OutputZone::Main, &format!("狀態: {:?}", quest.status));
            trigger_output(OutputZone::Main, &format!("目標:\n  {}", quest.description));
        }
        None => trigger_output(OutputZone::Status, &format!("找不到任務: {quest_id}")),
    }
}

pub fn handle_quest_start(game_world: &mut GameWorld, quest_id: &str) {
    match game_world.quest_manager.start_quest(quest_id) {
        Ok(msg) => {
            trigger_output(OutputZone::Main, &msg);
            save_quests(game_world);
        }
        Err(e) => trigger_output(OutputZone::Status, &e),
    }
}

pub fn handle_quest_complete(game_world: &mut GameWorld, quest_id: &str) {
    match game_world.quest_manager.complete_quest(quest_id) {
        Ok(rewards) => {
            let name = game_world
                .quest_manager
                .get_quest(quest_id)
                .map(|q| q.name.clone())
                .unwrap_or_else(|| quest_id.to_string());
            trigger_output(OutputZone::Main, &format!("任務完成: {name}"));
            trigger_output(OutputZone::Main, "獲得獎勵:");
            apply_quest_rewards(game_world, rewards);
            save_quests(game_world);
        }
        Err(e) => trigger_output(OutputZone::Status, &e),
    }
}

pub fn handle_quest_abandon(game_world: &mut GameWorld, quest_id: &str) {
    match game_world.quest_manager.abandon_quest(quest_id) {
        Ok(msg) => {
            trigger_output(OutputZone::Main, &msg);
            save_quests(game_world);
        }
        Err(e) => trigger_output(OutputZone::Status, &e),
    }
}

fn save_quests(game_world: &GameWorld) {
    let quest_dir = format!("{}/quests", game_world.world_dir);
    if let Err(e) = game_world.quest_manager.save_to_directory(&quest_dir) {
        trigger_output(OutputZone::Log, &format!("⚠️  任務存檔失敗: {e}"));
    }
}

/// 發放任務獎勵
fn apply_quest_rewards(game_world: &mut GameWorld, rewards: Vec<crate::quest::QuestReward>) {
    use crate::quest::QuestReward;

    let person_dir = format!("{}/persons", game_world.world_dir);

    for reward in rewards {
        match reward {
            QuestReward::Item { item, count } => {
                let display_name = crate::item_registry::get_item_display_name(&item);
                trigger_output(OutputZone::Main, &format!("  - 物品: {display_name} x{count}"));
                if let Some(me) = game_world.npc_manager.get_npc_mut("me") {
                    me.add_items(item, count);
                    let _ = me.save(&person_dir, "me");
                }
            }
            QuestReward::Experience { amount } => {
                trigger_output(OutputZone::Main, &format!("  - 經驗值: {amount}"));
                if let Some(me) = game_world.npc_manager.get_npc_mut("me") {
                    me.combat_exp += amount as i32;
                    let _ = me.save(&person_dir, "me");
                }
            }
            QuestReward::Relationship { npc_id, change } => {
                if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) {
                    npc.change_relationship(change);
                    trigger_output(
                        OutputZone::Main,
                        &format!("  - {npc_id} 對你的好感度變化: {change}"),
                    );
                    let _ = npc.save(&person_dir, &npc_id);
                }
            }
            QuestReward::UnlockDialogue { npc_id, scene, text } => {
                if let Some(npc) = game_world.npc_manager.get_npc_mut(&npc_id) {
                    npc.set_dialogue(scene.clone(), text);
                    trigger_output(OutputZone::Main, &format!("  - 解鎖 {npc_id} 的 {scene} 對話"));
                    let _ = npc.save(&person_dir, &npc_id);
                }
            }
            QuestReward::StatBoost { stat, amount } => {
                trigger_output(OutputZone::Main, &format!("  - 屬性提升: {stat} +{amount}"));
                if let Some(me) = game_world.npc_manager.get_npc_mut("me") {
                    boost_stat(me, &stat, amount);
                    let _ = me.save(&person_dir, "me");
                }
            }
        }
    }
}

/// 任務獎勵的屬性加成
fn boost_stat(person: &mut Person, stat: &str, amount: i32) {
    match stat.to_lowercase().as_str() {
        "hp" | "體力" => {
            person.max_hp += amount;
            person.hp += amount;
        }
        "mp" | "精神" => {
            person.max_mp += amount;
            person.mp += amount;
        }
        "strength" | "力量" => person.strength += amount,
        "knowledge" | "知識" => person.knowledge += amount,
        "sociality" | "交誼" => person.sociality += amount,
        other => trigger_output(OutputZone::Log, &format!("未知的屬性加成: {other}")),
    }
}
