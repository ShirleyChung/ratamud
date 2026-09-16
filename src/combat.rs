//! 回合制戰鬥 —— UI 無關的共用實作
//!
//! 以前整套戰鬥只寫在 `app.rs`，FFI 模式的 `punch` / `kick` / `escape` 只會回
//! 「戰鬥系統需要在終端 UI 模式中使用」，而且 iOS 沒有東西去推進回合，所以
//! NPC 打玩家的傷害也從來不會結算。搬到這裡之後：
//!
//! * `command_executor` 可以直接處理戰鬥指令。
//! * `ffi::ratamud_tick` 可以呼叫 [`execute_combat_round`] 推進自動回合，
//!   對應終端版主迴圈裡「每 3 秒一回合」的邏輯。

use crate::core_output::{trigger_output, OutputZone};
use crate::world::{CombatState, GameWorld};

/// 自動戰鬥回合間隔（秒），與終端版主迴圈相同
pub const COMBAT_ROUND_INTERVAL_SECS: u64 = 3;

/// `punch` / `kick` —— 沒有目標時進入練習模式
pub fn handle_combat_skill(game_world: &mut GameWorld, skill_name: &str, target: Option<String>) {
    let in_combat = !matches!(game_world.combat_state, CombatState::None);

    let target_name = match target {
        Some(t) => t,
        None if in_combat => {
            // 戰鬥中不指定目標就打第一個對手
            let CombatState::InCombat { ref participants, .. } = game_world.combat_state else {
                return;
            };
            match participants.iter().find(|p| p.as_str() != "me") {
                Some(t) => t.clone(),
                None => {
                    trigger_output(OutputZone::Main, "戰鬥中沒有可攻擊的目標");
                    return;
                }
            }
        }
        None => {
            enter_practice_mode(game_world, skill_name);
            return;
        }
    };

    execute_combat(game_world, skill_name, &target_name, in_combat);
}

/// 沒有目標時的技能練習
fn enter_practice_mode(game_world: &mut GameWorld, skill_name: &str) {
    let id = game_world.current_controlled_id.clone();
    let person_dir = format!("{}/persons", game_world.world_dir);

    let Some(me) = game_world.npc_manager.get_npc_mut(&id) else { return };

    match me.practice_skill(skill_name, false) {
        Some(msg) => {
            trigger_output(OutputZone::Main, &msg);
            let _ = me.save(&person_dir, &id);
        }
        None => trigger_output(OutputZone::Main, &format!("未知技能: {skill_name}")),
    }
}

/// 檢查條件 → 開戰 → 玩家出手 → 推進一回合
fn execute_combat(game_world: &mut GameWorld, skill_name: &str, target_name: &str, in_combat: bool) {
    // 玩家體力是否足以開戰
    {
        let Some(me) = game_world.npc_manager.get_npc("me") else { return };
        if !me.can_start_combat() && !in_combat {
            trigger_output(OutputZone::Main, "你沒力氣戰鬥");
            return;
        }
    }

    if !check_target_valid(game_world, target_name, in_combat) {
        return;
    }

    if !in_combat {
        game_world.combat_state = CombatState::InCombat {
            participants: vec!["me".to_string(), target_name.to_string()],
            round: 1,
        };
        trigger_output(OutputZone::Main, &format!("⚔️  戰鬥開始！你 vs {target_name}"));
    }

    execute_attack(game_world, skill_name, "me", target_name);
    execute_combat_round(game_world);
}

/// 目標必須存在、有體力、且在相鄰格內
fn check_target_valid(game_world: &GameWorld, target_name: &str, in_combat: bool) -> bool {
    let Some(target) = game_world.npc_manager.get_npc(target_name) else {
        trigger_output(OutputZone::Main, &format!("找不到 {target_name}"));
        return false;
    };

    if !target.can_start_combat() && !in_combat {
        trigger_output(OutputZone::Main, &format!("{} 沒力氣跟你打", target.name));
        return false;
    }

    let Some(me) = game_world.npc_manager.get_npc("me") else { return false };

    let too_far = target.map != game_world.current_map_name
        || (target.x as i32 - me.x as i32).abs() + (target.y as i32 - me.y as i32).abs() > 1;

    if too_far {
        trigger_output(OutputZone::Main, &format!("{} 距離太遠", target.name));
        return false;
    }

    true
}

/// 執行一次攻擊：結算傷害、設定冷卻、檢查戰鬥是否結束
pub fn execute_attack(
    game_world: &mut GameWorld,
    skill_name: &str,
    attacker_id: &str,
    defender_id: &str,
) {
    let Some(attacker) = game_world.npc_manager.get_npc(attacker_id) else { return };

    // 冷卻中就跳過；玩家要收到提示，NPC 則安靜跳過
    if attacker.get_skill_cooldown(skill_name) > 0 {
        if attacker_id == "me" {
            trigger_output(OutputZone::Main, &format!("你還沒準備好 {skill_name}"));
        }
        return;
    }

    let attacker_name = attacker.name.clone();
    let skill_dialogue = attacker.get_skill_dialogue(skill_name);
    let damage = attacker
        .combat_skills
        .get(skill_name)
        .map(|s| s.damage)
        .unwrap_or(1);

    // 結算傷害
    if let Some(defender) = game_world.npc_manager.get_npc_mut(defender_id) {
        defender.check_hp(-damage);
        let (defender_name, hp, max_hp) = (defender.name.clone(), defender.hp, defender.max_hp);

        let who = if defender_id == "me" { "你".to_string() } else { defender_name };
        trigger_output(
            OutputZone::Main,
            &format!(
                "💥 {attacker_name} 說：「{skill_dialogue}」造成 {damage} 點傷害！{who} 剩餘 HP: {hp}/{max_hp}"
            ),
        );
    }

    // 設定技能冷卻並增加熟練度
    if let Some(attacker) = game_world.npc_manager.get_npc_mut(attacker_id) {
        let _ = attacker.practice_skill(skill_name, true);
    }

    check_combat_end(game_world);
}

/// 任一方 HP 低於一半就結束戰鬥並結算經驗
pub fn check_combat_end(game_world: &mut GameWorld) {
    let CombatState::InCombat { ref participants, round } = game_world.combat_state else {
        return;
    };
    let participants = participants.clone();
    let current_round = round;

    let mut combat_ended = false;

    if let Some(me) = game_world.npc_manager.get_npc("me") {
        if me.hp <= me.max_hp / 2 {
            combat_ended = true;
            trigger_output(OutputZone::Main, "你的HP低於50%，戰鬥結束！");
        }
    }

    for participant in participants.iter().filter(|p| p.as_str() != "me") {
        if let Some(npc) = game_world.npc_manager.get_npc(participant) {
            if npc.hp <= npc.max_hp / 2 {
                combat_ended = true;
                trigger_output(
                    OutputZone::Main,
                    &format!("{} 的HP低於50%，戰鬥結束！", npc.name),
                );
            }
        }
    }

    if !combat_ended {
        return;
    }

    trigger_output(OutputZone::Main, "⚔️  戰鬥結束！");
    if let Some(me) = game_world.npc_manager.get_npc("me") {
        trigger_output(OutputZone::Main, &format!("你的 HP: {}/{}", me.hp, me.max_hp));
    }
    for participant in participants.iter().filter(|p| p.as_str() != "me") {
        if let Some(npc) = game_world.npc_manager.get_npc(participant) {
            trigger_output(
                OutputZone::Main,
                &format!("{} 的 HP: {}/{}", npc.name, npc.hp, npc.max_hp),
            );
        }
    }

    // 全體給經驗
    let exp = current_round * 2;
    for participant in participants.iter() {
        if let Some(npc) = game_world.npc_manager.get_npc_mut(participant) {
            npc.combat_exp += exp;
        }
    }
    trigger_output(OutputZone::Main, &format!("獲得 {exp} 點戰鬥經驗！"));

    game_world.combat_state = CombatState::None;
    save_combatants(game_world, &participants);
}

/// 推進一個戰鬥回合：NPC 行動 + 回合結算（減少冷卻）
///
/// 回傳 true 表示這回合真的有動作發生（host 可據此決定要不要刷新面板）。
pub fn execute_combat_round(game_world: &mut GameWorld) -> bool {
    use rand::Rng;

    let CombatState::InCombat { ref participants, .. } = game_world.combat_state else {
        return false;
    };
    let participants = participants.clone();

    // NPC 行動（各有 50% 機率出手，與終端版相同）
    for participant in participants.iter().filter(|p| p.as_str() != "me") {
        let (should_act, skill_name) = {
            let mut rng = rand::thread_rng();
            let skills = ["punch", "kick"];
            (rng.gen_bool(0.5), skills[rng.gen_range(0..skills.len())])
        };

        if should_act {
            execute_attack(game_world, skill_name, participant, "me");
        }

        // 攻擊可能已經結束戰鬥，結束後就不要再讓其他 NPC 出手
        if matches!(game_world.combat_state, CombatState::None) {
            return true;
        }
    }

    // 回合結算：回合數 +1、所有參與者減少技能冷卻
    if let CombatState::InCombat { ref mut round, .. } = game_world.combat_state {
        *round += 1;
    }
    for participant in participants.iter() {
        if let Some(npc) = game_world.npc_manager.get_npc_mut(participant) {
            npc.reduce_skill_cooldowns();
        }
    }

    save_combatants(game_world, &participants);
    true
}

/// `escape` —— 逃離戰鬥，只拿一半經驗
pub fn handle_escape(game_world: &mut GameWorld) {
    let CombatState::InCombat { ref participants, round } = game_world.combat_state else {
        trigger_output(OutputZone::Main, "你不在戰鬥中");
        return;
    };
    let participants = participants.clone();
    let exp = round / 2;

    for participant in participants.iter() {
        if let Some(npc) = game_world.npc_manager.get_npc_mut(participant) {
            npc.combat_exp += exp;
        }
    }

    trigger_output(OutputZone::Main, &format!("逃跑獲得 {exp} 點戰鬥經驗"));
    game_world.combat_state = CombatState::None;
    trigger_output(OutputZone::Main, "你逃離了戰鬥！");

    save_combatants(game_world, &participants);
}

/// 把參戰者的 HP / 經驗寫回檔案（persons 檔案很小，這裡直接存）
fn save_combatants(game_world: &GameWorld, participants: &[String]) {
    let person_dir = format!("{}/persons", game_world.world_dir);
    for participant in participants {
        if let Some(npc) = game_world.npc_manager.get_npc(participant) {
            let _ = npc.save(&person_dir, participant);
        }
    }
}
