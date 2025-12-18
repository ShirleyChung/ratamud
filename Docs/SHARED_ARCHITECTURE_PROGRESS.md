# 共享变量架构重构 - 进度报告

## 当前状态

✅ **已完成**：
1. `GameWorld` 结构已改为使用 `Arc<Mutex<T>>`
   - `maps: Arc<Mutex<HashMap<String, Map>>>`
   - `current_map_name: Arc<Mutex<String>>`
   - `time: Arc<Mutex<WorldTime>>`
   - `event_manager: Arc<Mutex<EventManager>>`
   - `npc_manager: Arc<Mutex<NpcManager>>`

2. 核心文件已适配：
   - ✅ `src/world.rs` - 已添加辅助方法
   - ✅ `src/app.rs` - 线程启动逻辑已重写
   - ✅ `src/time_thread.rs` - 接受 Arc<Mutex<WorldTime>>

3. 线程已改为真正共享数据：
   - NPC AI 线程直接使用 `Arc::clone(&game_world.npc_manager)`  
   - 事件检查线程直接使用 `Arc::clone(&game_world.time)`
   - **不再需要手动同步！**

## 🔧 待修复文件（84个错误）

主要问题：很多文件直接访问 `Arc<Mutex<T>>` 的方法，需要先 `.lock().unwrap()`

### 需要修复的文件

1. **npc_ai.rs** - NPC AI 逻辑
2. **input.rs** - 用户输入处理（大量命令访问 game_world）
3. **ui.rs** - UI 渲染（访问地图数据）
4. **event_executor.rs** - 事件执行
5. 其他命令处理文件

### 典型错误模式

```rust
// ❌ 错误：直接调用方法
game_world.npc_manager.get_all_npc_ids()

// ✅ 正确：先 lock
game_world.npc_manager.lock().unwrap().get_all_npc_ids()

// ❌ 错误：直接访问字段
game_world.time.hour

// ✅ 正确：先 lock
game_world.time.lock().unwrap().hour

// ❌ 错误：直接访问 HashMap 方法
game_world.maps.get(&map_name)

// ✅ 正确：先 lock，或使用辅助方法
game_world.maps.lock().unwrap().get(&map_name)
game_world.get_current_map()  // 使用辅助方法更好
```

## 🎯 优势（完成后）

### 之前（clone + 同步）
```rust
// 主循环每次都要：
1. clone 整个地图 HashMap
2. 遍历所有 NPC 双向同步
3. 遍历所有地图的所有点位同步物品
```

### 现在（真正共享）
```rust
// 线程直接操作共享数据：
NPC AI 线程 ─┬─> Arc<Mutex<NpcManager>> ←─┐
事件线程 ────┼─> Arc<Mutex<Time>> ←────────┼─ 主线程
时间线程 ────┴─> Arc<Mutex<EventManager>> ←┘

// 无需同步！数据永远一致！
```

## 📋 下一步

### 方案 1：逐个修复（耗时但彻底）
逐个文件修复所有访问点，约需要修改100+处

### 方案 2：添加便利方法（推荐）
在 `GameWorld` 中添加更多辅助方法，减少直接 lock 的需要：

```rust
impl GameWorld {
    // NPC 相关
    pub fn get_npc(&self, id: &str) -> Option<Person> {
        self.npc_manager.lock().unwrap().get_npc(id).cloned()
    }
    
    pub fn with_npc_mut<F, R>(&self, id: &str, f: F) -> Option<R>
    where F: FnOnce(&mut Person) -> R {
        let mut mgr = self.npc_manager.lock().unwrap();
        mgr.get_npc_mut(id).map(f)
    }
    
    // 地图相关
    pub fn get_map(&self, name: &str) -> Option<Map> {
        self.maps.lock().unwrap().get(name).cloned()
    }
    
    // ... 更多辅助方法
}
```

### 方案 3：部分回退
保持核心架构（Arc<Mutex>），但某些不常修改的字段可以用其他方式

## 📊 性能对比

### Clone 方案（之前）
- ❌ 每循环 clone 整个地图（100x100 points）
- ❌ 遍历所有 NPC 同步状态
- ❌ CPU: 高
- ❌ 内存: 大量临时对象

### Arc<Mutex> 方案（现在）
- ✅ 零拷贝，真正共享
- ✅ 数据永远一致
- ✅ CPU: 低（只有 lock 开销）
- ✅ 内存: 最小

## 🚀 启动方式

```rust
// main.rs - 在加载完数据后启动线程
setup_npc_ai_thread(&mut game_world);
setup_event_check_thread(&mut game_world);

// 然后进入主循环
app::run_main_loop(...)?;
```

---

**重构时间**: 2025-12-16  
**当前错误数**: 84  
**主要工作**: 添加辅助方法 + 修复访问点
