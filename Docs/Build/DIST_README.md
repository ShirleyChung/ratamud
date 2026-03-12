# RataMUD 動態連結函式庫

## 檔案說明

- `libratamud.dylib` - 動態連結函式庫
- `ratamud.h` - C 標頭檔
- `example.c` - 使用範例

## 使用方式

### macOS/Linux

編譯範例程式:
```bash
gcc -o example example.c -L. -lratamud -Wl,-rpath,.
```

執行:
```bash
./example
```

### iOS 開發

1. 將 `libratamud.dylib` 添加到 Xcode 項目
2. 包含 `ratamud.h` 標頭檔
3. 在 Swift 中使用 Bridging Header 調用

Swift 範例:
```swift
import Foundation

class RataMUDWrapper {
    init?() {
        if ratamud_init() != 0 {
            return nil
        }
    }
    
    deinit {
        ratamud_cleanup()
    }
    
    func processCommand(_ command: String) -> Bool {
        return ratamud_process_command(command) == 0
    }
    
    func getOutput() -> String? {
        guard let ptr = ratamud_get_output() else { return nil }
        let str = String(cString: ptr)
        ratamud_free_string(ptr)
        return str
    }
}
```

### Android 開發

使用 JNI 包裝:
```java
public class RataMUD {
    static {
        System.loadLibrary("ratamud");
    }
    
    public native int init();
    public native void cleanup();
    public native int processCommand(String command);
    public native String getOutput();
}
```

## 跨平台編譯

### iOS (使用 cargo-lipo)
```bash
cargo install cargo-lipo
cargo lipo --release
```

### Android (使用 cargo-ndk)
```bash
cargo install cargo-ndk
cargo ndk --target aarch64-linux-android --platform 21 -- build --release
```

### Windows
```bash
cargo build --release --lib --target x86_64-pc-windows-msvc
```

## API 文檔

查看 `ratamud.h` 獲取完整的 API 文檔。

## 授權

與 RataMUD 主項目相同。
