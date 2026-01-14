# NPC AI Strategy 模式使用指南

## 概述

我們已經使用 **Strategy 設計模式** 重構了 NPC AI 行為系統，讓您可以輕鬆擴充 NPC 的 AI 行為。

## 架構說明

### 核心組件

1. **NpcAiStrategy trait** - 所有 AI 策略都實現這個特徵
2. **NpcAiStrategyComposer** - 策略組合器，按優先級執行策略
3. **NpcAiController** - AI 控制器，使用策略組合器決定行為

### 內建策略（按優先級排序）

1. **InteractingStrategy** (優先級 10) - 互動中返回 Idle
2. **CombatStrategy** (優先級 20) - 戰鬥中使用技能
3. **PartyStrategy** (優先級 30) - 隊伍中不隨意移動
4. **HealingStrategy** (優先級 40) - HP 低於一半時使用食物
5. **RandomBehaviorStrategy** (優先級 1000) - 隨機移動、撿物品或閒置

## 如何擴充 AI 行為

### 步驟 1: 創建新的策略

在 `src/npc_action.rs` 中添加新的策略結構：

```rust
/// 逃跑策略 - HP 低於 20% 時遠離敵人
pub struct FleeStrategy;

impl NpcAiStrategy for FleeStrategy {
    fn decide_action(&self, npc_view: &NpcView) -> Option<NpcAction> {
        // 如果 HP 低於 20% 且在戰鬥中
        if npc_view.in_combat && npc_view.self_hp < npc_view.self_max_hp / 5 {
            // 隨機選擇一個方向逃跑
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
            let direction = directions[rng.gen_range(0..directions.len())].clone();
            
            return Some(NpcAction::Move(direction));
        }
        
        None
    }
    
    fn priority(&self) -> i32 {
        15  // 比戰鬥策略優先（數字越小優先級越高）
    }
}
```

### 步驟 2: 註冊策略到組合器

修改 `NpcAiStrategyComposer` 的 `Default` 實現：

```rust
impl Default for NpcAiStrategyComposer {
    fn default() -> Self {
        Self::new()
            .add_strategy(Box::new(InteractingStrategy))
            .add_strategy(Box::new(FleeStrategy))  // 添加新策略
            .add_strategy(Box::new(CombatStrategy))
            .add_strategy(Box::new(PartyStrategy))
            .add_strategy(Box::new(HealingStrategy))
            .add_strategy(Box::new(RandomBehaviorStrategy))
    }
}
```

### 步驟 3: 編譯並測試

```bash
cargo build
```

## 更多範例

### 範例 1: 商人策略

```rust
/// 商人策略 - 在特定位置等待交易
pub struct MerchantStrategy;

impl NpcAiStrategy for MerchantStrategy {
    fn decide_action(&self, npc_view: &NpcView) -> Option<NpcAction> {
        // 只對特定 NPC 生效（可以通過 npc_view 的資訊判斷）
        // 這裡簡化處理，實際可能需要更多上下文
        
        // 如果有玩家在附近（通過 visible_items 可能需要擴展 NpcView）
        // 可以主動打招呼
        Some(NpcAction::Say("歡迎光臨！需要什麼嗎？".to_string()))
    }
    
    fn priority(&self) -> i32 {
        50
    }
}
```

### 範例 2: 巡邏策略

```rust
/// 巡邏策略 - 按照預定路線移動
pub struct PatrolStrategy {
    patrol_points: Vec<(usize, usize)>,
    current_index: std::sync::Arc<std::sync::Mutex<usize>>,
}

impl NpcAiStrategy for PatrolStrategy {
    fn decide_action(&self, npc_view: &NpcView) -> Option<NpcAction> {
        let mut index = self.current_index.lock().unwrap();
        let target = self.patrol_points[*index];
        
        // 計算移動方向
        let dx = target.0 as i32 - npc_view.self_x as i32;
        let dy = target.1 as i32 - npc_view.self_y as i32;
        
        if dx == 0 && dy == 0 {
            // 到達目標點，切換到下一個點
            *index = (*index + 1) % self.patrol_points.len();
            return Some(NpcAction::Idle);
        }
        
        // 根據距離決定移動方向
        if dx.abs() > dy.abs() {
            if dx > 0 {
                Some(NpcAction::Move(Direction::Right))
            } else {
                Some(NpcAction::Move(Direction::Left))
            }
        } else if dy > 0 {
            Some(NpcAction::Move(Direction::Down))
        } else {
            Some(NpcAction::Move(Direction::Up))
        }
    }
    
    fn priority(&self) -> i32 {
        500
    }
}
```

## 優勢

1. **易於擴展** - 只需創建新的策略類別，實現 `NpcAiStrategy` trait
2. **優先級控制** - 通過 `priority()` 方法控制策略執行順序
3. **低耦合** - 每個策略獨立，互不影響
4. **易於測試** - 每個策略可以單獨測試
5. **動態組合** - 可以在運行時動態添加或移除策略

## 注意事項

1. **優先級** - 數字越小優先級越高（10 > 20 > 30）
2. **返回 None** - 如果策略不處理，返回 `None` 讓下一個策略處理
3. **返回 Some** - 如果策略處理，返回 `Some(NpcAction)`，後續策略不會執行
4. **線程安全** - 策略需要實現 `Send + Sync` trait
