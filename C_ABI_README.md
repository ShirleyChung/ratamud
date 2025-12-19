# RataMUD C ABI 接口 - 快速入門

## ✅ 已完成

RataMUD 現在支持 C ABI 接口，可以輕鬆移植到 iOS、Android、Unity 等多個平台！

## 🚀 快速開始

### 1. 構建動態連結函式庫

```bash
./build_dylib.sh release
```

這會在 `dist/` 目錄生成：
- **libratamud.dylib** (macOS) / libratamud.so (Linux) / ratamud.dll (Windows)
- **ratamud.h** - C 標頭檔
- **example.c** - 使用範例
- **README.md** - 使用說明

### 2. 測試範例

```bash
cd dist
gcc -o example example.c -L. -lratamud -Wl,-rpath,.
./example
```

輸出：
```
RataMUD C API 使用範例
版本: RataMUD v0.1.0

✓ 玩家創建成功
玩家名稱: 冒險者
✓ 世界創建成功
玩家資訊: {"hp":100000,"map":"初始之地",...}
玩家位置: (50, 50)
...
```

## 📦 動態函式庫大小

- Release 版本: ~655KB
- 包含完整遊戲引擎邏輯

## 🔌 C API 函數

### 創建與釋放

```c
Person* ratamud_create_player(const char* name, const char* description);
GameWorld* ratamud_create_world(Person* player);
void ratamud_free_player(Person* player);
void ratamud_free_world(GameWorld* world);
```

### 遊戲操作

```c
int ratamud_load_map(GameWorld* world, const char* map_name);
int ratamud_get_player_position(const Person* player, int* x, int* y);
int ratamud_set_player_position(Person* player, int x, int y);
char* ratamud_get_current_map(const GameWorld* world);
```

### 玩家屬性

```c
char* ratamud_get_player_name(const Person* player);
char* ratamud_get_player_info(const Person* player);  // JSON 格式
int ratamud_get_player_hp(const Person* player);
int ratamud_set_player_hp(Person* player, int hp);
```

### 記憶體管理

```c
void ratamud_free_string(char* s);  // 釋放 C 字串
```

## 🎯 使用範例

```c
#include "ratamud.h"

int main() {
    // 創建玩家和世界
    Person* player = ratamud_create_player("冒險者", "勇敢的探險家");
    GameWorld* world = ratamud_create_world(player);
    
    // 獲取玩家資訊
    char* info = ratamud_get_player_info(player);
    printf("%s\n", info);
    ratamud_free_string(info);
    
    // 移動玩家
    ratamud_set_player_position(player, 10, 20);
    
    // 載入地圖
    ratamud_load_map(world, "新手村");
    
    // 清理
    ratamud_free_world(world);
    ratamud_free_player(player);
}
```

## 📱 平台移植

### iOS

```bash
# 安裝工具
cargo install cargo-lipo
rustup target add aarch64-apple-ios x86_64-apple-ios

# 構建通用函式庫
cargo lipo --release
```

在 Swift 中使用：

```swift
class GameEngine {
    let player: OpaquePointer
    let world: OpaquePointer
    
    init?() {
        guard let p = ratamud_create_player("玩家", "描述") else { return nil }
        guard let w = ratamud_create_world(p) else { return nil }
        player = p
        world = w
    }
    
    deinit {
        ratamud_free_world(world)
        ratamud_free_player(player)
    }
}
```

### Android

```bash
# 安裝工具
cargo install cargo-ndk
rustup target add aarch64-linux-android

# 構建
cargo ndk --target aarch64-linux-android --platform 21 -- build --release
```

### Unity (C#)

```csharp
[DllImport("ratamud")]
private static extern IntPtr ratamud_create_player(string name, string desc);

[DllImport("ratamud")]
private static extern void ratamud_free_player(IntPtr player);
```

## 📚 詳細文檔

查看以下文檔獲取更多資訊：

- **dist/README.md** - 動態函式庫使用說明
- **dist/ratamud.h** - 完整 C API 文檔
- **Docs/C_ABI_GUIDE.md** - 詳細跨平台移植指南

## 🔧 技術細節

- **語言**: Rust with C FFI
- **線程安全**: 使用不透明指針，避免全局狀態
- **記憶體管理**: 明確的創建/釋放函數
- **編碼**: 所有字串使用 UTF-8
- **數據交換**: 複雜數據使用 JSON 格式

## 🎮 支持平台

- ✅ macOS (已測試)
- ✅ Linux
- ✅ Windows
- 🚧 iOS (需要 cargo-lipo)
- 🚧 Android (需要 cargo-ndk)
- 🚧 WebAssembly
- 🚧 Unity

## 📝 注意事項

1. **記憶體管理**: 所有返回的字串必須用 `ratamud_free_string()` 釋放
2. **不透明指針**: Person 和 GameWorld 是不透明類型，不要直接訪問
3. **錯誤處理**: 檢查函數返回值，NULL 或 -1 表示失敗
4. **UTF-8 編碼**: 所有字串參數和返回值都是 UTF-8

## 🤝 貢獻

歡迎提交 PR 改進 C ABI 接口或添加新功能！

## 📄 授權

與 RataMUD 主項目相同
