# RataMUD Library 模式使用指南

## ✅ 验证结果

### 所有输出已连接到回调系统

✅ **主要输出方法**（4个，全部已添加回调）：
- `print()` → 触发 `MAIN` 标记
- `log()` → 触发 `LOG` 标记  
- `set_status()` → 触发 `STATUS` 标记
- `set_side_content()` → 触发 `SIDE` 标记

⚠️ **次要输出**：
- `event_loader.rs:52` 有一个 `eprintln!` (仅用于加载错误)
- UI 控制方法（`show_minimap`, `toggle_log` 等）不产生内容输出

✅ **测试结果**：C++ 程序成功接收到所有 7 种输出（3个MAIN + 2个LOG + 1个STATUS + 1个SIDE）

---

## 📦 编译为库

### 自动编译（推荐）

```bash
./build_and_test.sh
```

### 手动编译

```bash
# 1. 编译 Rust 库
cargo build --release --lib

# 2. 生成的文件：
# - target/release/libratamud.dylib (macOS 动态库)
# - target/release/libratamud.so (Linux 动态库)
# - target/release/libratamud.a (静态库)

# 3. 编译 C++ 测试程序
# macOS:
g++ -std=c++11 test_callback.cpp -L./target/release -lratamud -o test_callback

# Linux:
g++ -std=c++11 test_callback.cpp -L./target/release -lratamud -Wl,-rpath,./target/release -o test_callback

# 4. 运行测试
./test_callback
```

---

## 🔌 C/C++ 集成示例

### 头文件

```c
#include "src/ratamud.h"
```

### 注册回调

```cpp
void my_callback(const char* msg_type, const char* content) {
    // msg_type 可能的值: "MAIN", "LOG", "STATUS", "SIDE"
    printf("[%s] %s\n", msg_type, content);
}

int main() {
    // 注册回调
    ratamud_register_output_callback(my_callback);
    
    // 触发测试输出
    ratamud_test_output_callback();
    
    return 0;
}
```

### 实际应用示例

#### 1. 文件输出

```cpp
#include <fstream>

std::ofstream log_file("game.log");

void file_logger(const char* msg_type, const char* content) {
    log_file << msg_type << ":" << content << std::endl;
    log_file.flush();
}

ratamud_register_output_callback(file_logger);
```

#### 2. 分类输出

```cpp
void categorized_output(const char* msg_type, const char* content) {
    if (strcmp(msg_type, "MAIN") == 0) {
        // 主游戏内容 -> GUI 主窗口
        update_main_window(content);
    } else if (strcmp(msg_type, "LOG") == 0) {
        // 系统日志 -> 日志面板
        append_to_log_panel(content);
    } else if (strcmp(msg_type, "STATUS") == 0) {
        // 状态消息 -> 状态栏
        show_status_bar(content, 5000); // 5秒后消失
    } else if (strcmp(msg_type, "SIDE") == 0) {
        // 侧边信息 -> 侧边栏
        update_sidebar(content);
    }
}
```

#### 3. 网络传输

```cpp
#include <curl/curl.h>

void network_sender(const char* msg_type, const char* content) {
    // 发送到远程服务器
    std::string json = std::string("{\"type\":\"") + msg_type + 
                       "\",\"content\":\"" + content + "\"}";
    send_to_server(json);
}
```

---

## 📊 输出类型说明

| 标记 | 说明 | 典型内容 | 使用场景 |
|------|------|----------|----------|
| `MAIN` | 主游戏消息 | "你向北移动"<br>"你攻击了野猪" | 游戏主窗口 |
| `LOG` | 系统日志 | "遊戲初始化完成"<br>"載入地圖: town_square" | 日志面板 |
| `STATUS` | 状态栏消息 | "保存成功"<br>"命令无效" | 状态栏（5秒） |
| `SIDE` | 侧边面板 | "NPC: 商人\n等級: 10" | NPC 信息面板 |

---

## 🧪 测试程序输出示例

运行 `./test_callback` 会产生：

### 控制台输出（带颜色）
```
[MAIN] 歡迎來到 RataMUD！
[MAIN] 你站在一個廣場中央。
[LOG] 遊戲初始化完成
[LOG] 載入地圖: town_square
[STATUS] 保存成功
[SIDE] NPC: 商人
等級: 10
生命: 100/100
[MAIN] 一隻野豬向你衝來！
```

### 文件输出 (game_output.log)
```
MAIN:歡迎來到 RataMUD！
MAIN:你站在一個廣場中央。
LOG:遊戲初始化完成
LOG:載入地圖: town_square
STATUS:保存成功
SIDE:NPC: 商人
等級: 10
生命: 100/100
MAIN:一隻野豬向你衝來！
```

---

## 🏗️ 架构说明

### 双层回调系统

```
游戏代码
    ↓
OutputManager.print/log/set_status/set_side_content
    ↓
OutputManager.trigger_callback()
    ├─→ Rust 内部回调 (可选)
    └─→ FFI 回调 (C/C++/iOS/Android)
```

### 线程安全

- 使用 `Mutex` 保护全局回调
- 回调函数必须是 `Send` trait
- 可在多线程环境中安全调用

---

## 📝 API 参考

### Rust 端

```rust
// 设置 Rust 内部回调
output_manager.set_output_callback(|msg_type, content| {
    println!("[{}] {}", msg_type, content);
});
```

### C/C++ 端

```c
// 注册 C 回调
void ratamud_register_output_callback(OutputCallback callback);

// 测试函数
void ratamud_test_output_callback(void);
```

---

## 🔜 未来扩展

可能的改进方向：

1. **更多输出类型**
   - `COMBAT` - 战斗详情
   - `DIALOGUE` - 对话内容
   - `ERROR` - 错误消息
   - `DEBUG` - 调试信息

2. **异步回调**
   - 使用消息队列避免阻塞
   - 批量输出优化

3. **过滤器**
   - 只接收特定类型的输出
   - 根据优先级过滤

4. **多回调支持**
   - 观察者模式
   - 同时注册多个回调

---

## 📄 相关文件

- `src/output.rs` - OutputManager 核心实现
- `src/ffi.rs` - FFI 接口定义
- `src/ratamud.h` - C 头文件
- `test_callback.cpp` - C++ 测试程序
- `build_and_test.sh` - 自动编译脚本
- `OUTPUT_CALLBACK_USAGE.md` - Rust 回调使用指南

---

## ✅ 总结

✓ **所有主要输出都已连接到回调系统**  
✓ **项目已配置为 library (cdylib + staticlib + rlib)**  
✓ **C++ 测试程序验证回调正常工作**  
✓ **支持同时输出到控制台和文件**  
✓ **使用类型标记清晰区分不同输出来源**

RataMUD 现在可以作为库被其他系统使用，并通过回调接收所有游戏输出！
