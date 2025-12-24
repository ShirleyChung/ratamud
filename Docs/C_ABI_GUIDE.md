# RataMUD C ABI 接口與跨平台移植指南

## 概述

RataMUD 現在提供了 C ABI 接口，可以方便地移植到 iOS、Android 等其他平台。

## 快速開始

### 構建動態連結函式庫

```bash
./build_dylib.sh release
```

這會在 `dist/` 目錄下生成：
- `libratamud.dylib` (macOS) / `libratamud.so` (Linux) / `ratamud.dll` (Windows)
- `ratamud.h` - C 標頭檔
- `example.c` - 使用範例
- `README.md` - 詳細說明

### 測試範例程式

```bash
cd dist
gcc -o example example.c -L. -lratamud -Wl,-rpath,.
./example
```

## C API 函數列表

### 初始化與清理

- `int ratamud_init(void)` - 初始化遊戲引擎
- `void ratamud_cleanup(void)` - 清理資源

### 遊戲交互

- `int ratamud_process_command(const char* command)` - 處理遊戲命令
- `char* ratamud_get_output(void)` - 獲取遊戲輸出
- `int ratamud_update(int delta_ms)` - 更新遊戲狀態

### 查詢函數

- `int ratamud_get_player_position(int* x, int* y)` - 獲取玩家位置
- `char* ratamud_get_current_map(void)` - 獲取當前地圖名稱
- `char* ratamud_get_player_info(void)` - 獲取玩家資訊 (JSON)
- `const char* ratamud_version(void)` - 獲取版本資訊

### 記憶體管理

- `void ratamud_free_string(char* s)` - 釋放字串記憶體

## 平台移植指南

### iOS 移植

#### 1. 構建 iOS 函式庫

```bash
# 安裝 cargo-lipo
cargo install cargo-lipo

# 添加 iOS 目標
rustup target add aarch64-apple-ios x86_64-apple-ios

# 構建通用函式庫
cargo lipo --release
```

#### 2. Xcode 整合

1. 將生成的 `.a` 檔案添加到 Xcode 項目
2. 創建 Bridging Header 包含 `ratamud.h`
3. 在 Swift 中調用

Swift 範例:
```swift
class GameEngine {
    init?() {
        guard ratamud_init() == 0 else { return nil }
    }
    
    deinit {
        ratamud_cleanup()
    }
    
    func execute(_ command: String) -> String? {
        _ = ratamud_process_command(command)
        guard let ptr = ratamud_get_output() else { return nil }
        let output = String(cString: ptr)
        ratamud_free_string(ptr)
        return output
    }
}
```

### Android 移植

#### 1. 構建 Android 函式庫

```bash
# 安裝 cargo-ndk
cargo install cargo-ndk

# 添加 Android 目標
rustup target add aarch64-linux-android armv7-linux-androideabi

# 構建
cargo ndk --target aarch64-linux-android --platform 21 -- build --release
```

#### 2. JNI 包裝

```java
public class RataMUD {
    static {
        System.loadLibrary("ratamud");
    }
    
    public native int init();
    public native void cleanup();
    public native int processCommand(String command);
    public native String getOutput();
    public native String getPlayerInfo();
}
```

### Unity 整合

```csharp
using System;
using System.Runtime.InteropServices;

public class RataMUDWrapper : IDisposable
{
    [DllImport("ratamud")]
    private static extern int ratamud_init();
    
    [DllImport("ratamud")]
    private static extern void ratamud_cleanup();
    
    [DllImport("ratamud")]
    private static extern int ratamud_process_command(string command);
    
    [DllImport("ratamud")]
    private static extern IntPtr ratamud_get_output();
    
    [DllImport("ratamud")]
    private static extern void ratamud_free_string(IntPtr ptr);
    
    public bool Init()
    {
        return ratamud_init() == 0;
    }
    
    public string ProcessCommand(string command)
    {
        ratamud_process_command(command);
        IntPtr ptr = ratamud_get_output();
        string output = Marshal.PtrToStringUTF8(ptr);
        ratamud_free_string(ptr);
        return output;
    }
    
    public void Dispose()
    {
        ratamud_cleanup();
    }
}
```

### Web Assembly (WASM)

```bash
# 添加 wasm 目標
rustup target add wasm32-unknown-unknown

# 構建
cargo build --release --target wasm32-unknown-unknown --lib
```

## 架構說明

### FFI 層設計

`src/ffi.rs` 提供了線程安全的 C ABI 接口：

1. **全局狀態管理**: 使用 `Lazy<Mutex<Option<GameState>>>` 管理遊戲狀態
2. **記憶體安全**: 所有字串使用 `CString` 管理，呼叫者負責釋放
3. **錯誤處理**: 返回 0 表示成功，-1 表示失敗
4. **JSON 交互**: 複雜數據使用 JSON 格式傳遞

### 注意事項

1. **線程安全**: FFI 層使用 Mutex 保護，但建議在單線程環境使用
2. **記憶體管理**: 所有 `get_*` 函數返回的字串必須用 `ratamud_free_string` 釋放
3. **初始化順序**: 必須先調用 `ratamud_init()`，結束時調用 `ratamud_cleanup()`
4. **錯誤檢查**: 每次調用後檢查返回值

## 性能考量

- 動態連結函式庫大小: ~5-10MB (release 模式)
- 初始化時間: ~100-500ms
- 命令處理: <10ms
- 記憶體佔用: ~10-50MB

## 未來擴展

計劃添加的功能：

- [ ] 異步命令處理
- [ ] 事件回調機制
- [ ] 圖形渲染接口
- [ ] 音效接口
- [ ] 網絡多人遊戲支持

## 疑難排解

### 找不到動態函式庫

macOS:
```bash
export DYLD_LIBRARY_PATH=./dist:$DYLD_LIBRARY_PATH
```

Linux:
```bash
export LD_LIBRARY_PATH=./dist:$LD_LIBRARY_PATH
```

### 符號未定義

檢查是否正確鏈接:
```bash
# macOS
otool -L libratamud.dylib

# Linux  
ldd libratamud.so

# 查看導出符號
nm -g libratamud.dylib | grep ratamud
```

## 貢獻

歡迎提交 PR 改進跨平台支持！

## 授權

與 RataMUD 主項目相同。
