# RataMUD 程式碼規範

> **所有開發者與 AI 助手在修改程式碼時都必須遵守這些核心準則**

---

## 核心準則

### 1. 程式要有註解

**關鍵邏輯、複雜演算法、公開 API 必須加註解**

```rust
✅ 正確：清楚的註解
/// 更新 NPC 距離並返回靠近/離開的通知
/// 
/// # 參數
/// * `player_just_moved` - true=玩家主動行動, false=NPC移動
/// 
/// # 返回
/// Vec<(npc_id, message, should_greet)>
pub fn update_proximity(...) -> Vec<(String, String, bool)> {
    // 檢查控制角色是否移動（用於決定訊息類型）
    let controlled_moved = prev_x != current_x;
}

❌ 錯誤：無註解或廢話註解
pub fn update_proximity(...) {  // ❌ 沒有說明
    let x = 10;  // 設定 x 為 10  ❌ 廢話
}
```

**規則：**
- ✅ 公開函數使用 `///` 文檔註解
- ✅ 複雜邏輯說明「為什麼」而非「做什麼」
- ✅ 非顯而易見的設計決策要註解
- ❌ 避免廢話註解

---

### 2. 函數避免過於龐大

**單一函數不超過 100 行，超過則拆分**

```rust
❌ 錯誤：函數過長
fn handle_command_result(...) {
    match result {
        // 50+ 個 match arms
    }
    // 再 50 行 proximity 檢測
    // 又 20 行 minimap 更新
}  // 總共 200+ 行 ❌

✅ 正確：拆分成多個函數
fn handle_command_result(...) {
    match result {
        CommandResult::Move(dx, dy) => handle_movement(dx, dy, ...),
        // ... 簡潔的分派
    }
    
    check_and_handle_proximity(...);  // 獨立函數
    update_minimap_if_open(...);      // 獨立函數
}
```

**規則：**
- ✅ 函數 > 100 行時考慮拆分
- ✅ 每個函數只做一件事
- ✅ 提取重複邏輯為獨立函數

---

### 3. 避免重複寫同樣的功能（DRY）

**Don't Repeat Yourself - 相同邏輯出現 2 次以上必須提取**

```rust
❌ 錯誤：重複的邏輯
// 在主循環
let notifications = game_world.npc_manager.update_proximity(...);
for (npc_id, msg, greet) in notifications {
    output_manager.print(msg);
    if greet { /* ... */ }
}

// 在指令處理又重複一次（20+ 行相同程式碼）
let notifications = game_world.npc_manager.update_proximity(...);
for (npc_id, msg, greet) in notifications {
    output_manager.print(msg);
    if greet { /* ... */ }
}

✅ 正確：提取為共用函數
fn check_and_handle_proximity(...) {
    let notifications = game_world.npc_manager.update_proximity(...);
    for (npc_id, msg, greet) in notifications {
        output_manager.print(msg);
        if greet { /* ... */ }
    }
}

// 調用處只需一行
check_and_handle_proximity(&mut output_manager, &mut game_world, &me, false);
```

**規則：**
- ✅ 相同邏輯出現 2+ 次必須提取
- ✅ 使用參數控制差異
- ✅ 優先使用函數而非複製貼上

---

### 4. 能共享記憶體就不 clone

**優先使用引用，只在必要時 clone**

```rust
❌ 錯誤：不必要的 clone
fn process_npc(npc: Person) {  // 取得所有權，強制外部 clone
    println!("{}", npc.name);
}

let npc = game_world.npc_manager.get_npc("merchant").unwrap().clone();  // ❌
process_npc(npc);

✅ 正確：使用引用
fn process_npc(npc: &Person) {  // 借用即可
    println!("{}", npc.name);
}

let npc = game_world.npc_manager.get_npc("merchant").unwrap();
process_npc(npc);  // 無需 clone
```

**允許 clone 的情況：**

```rust
✅ 合理的 clone

// 1. 跨執行緒傳遞
let time = game_world.time.clone();
thread::spawn(move || { /* 使用 time */ });

// 2. 避免借用衝突
let map_name = me.map.clone();  // 先 clone
game_world.do_something(&mut me);  // 可變借用
println!("{}", map_name);  // 使用之前的值

// 3. 保存狀態快照
game_world.original_player = Some(me.clone());
```

**規則：**
- ✅ 優先使用 `&T` 而非 `T`
- ✅ 返回引用而非所有權（在可能的情況下）
- ✅ 只在必要時 `clone()`（跨執行緒、借用衝突、快照）

---

### 5. Warning 要避免

**程式碼必須無任何編譯警告**

```bash
✅ 每次修改後必須執行
cargo build    # 必須無 warning
cargo clippy   # 必須無 warning
```

**處理未使用的程式碼：**

```rust
❌ 錯誤：保留未使用的程式碼產生 warning
fn load_npcs(...) {  // warning: function is never used
    // 空函數或未使用
}

✅ 正確：移除未使用的程式碼
// 直接刪除

✅ 或：標註未來會用（謹慎使用）
#[allow(dead_code)]
fn future_feature() {
    // 確定未來會使用的功能
}
```

**規則：**
- ✅ 未使用的程式碼必須移除
- ✅ 將來會用的使用 `#[allow(dead_code)]` 保留
- ✅ 不可忽略 warning

---

### 6. 要 run cargo clippy 優化風格

**使用 clippy 優化程式碼風格**

```bash
# 檢查 clippy 建議
cargo clippy

# 自動修復
cargo clippy --fix --allow-dirty --allow-staged
```

**常見優化：**

```rust
❌ Clippy 會提示的問題

// 1. 舊式格式化
format!("Hello {}", name)  // ❌

// 2. 不必要的 max().min()
(value).max(0).min(100)  // ❌

// 3. loop with if let
loop {
    if let Ok(data) = rx.recv() { /* ... */ }
    else { break; }
}  // ❌

✅ Clippy 建議的寫法

// 1. 現代格式化
format!("Hello {name}")  // ✅

// 2. 使用 clamp
value.clamp(0, 100)  // ✅

// 3. while let
while let Ok(data) = rx.recv() {
    /* ... */
}  // ✅
```

**規則：**
- ✅ 提交前執行 `cargo clippy`
- ✅ 修復所有 clippy 建議
- ✅ 使用 `cargo clippy --fix` 自動修復

---

## 📌 提交前檢查清單

每次修改後，確認以下項目：

- [ ] 關鍵邏輯有註解
- [ ] 函數長度 < 100 行
- [ ] 無重複程式碼（DRY）
- [ ] 無不必要的 `clone()`
- [ ] `cargo build` 無 warning
- [ ] `cargo clippy` 無 warning

## 修改完後
-  簡易總結就好，不必詳列

---

**最後更新**: 2026-01-01  
**維護者**: RataMUD 開發團隊
