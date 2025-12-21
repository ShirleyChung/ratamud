# iOS Framework 使用說明

## 前置需求

1. **安裝 Xcode**（不僅是 Command Line Tools）
   - 從 App Store 安裝 Xcode
   - 首次打開 Xcode 並接受授權條款

2. **切換到 Xcode 開發工具**
   ```bash
   sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
   ```

3. **驗證設置**
   ```bash
   xcode-select -p
   # 應該輸出: /Applications/Xcode.app/Contents/Developer
   ```

## 編譯 iOS Framework

執行以下指令編譯 XCFramework：

```bash
./build_ios_framework.sh
```

這將會：
1. 安裝必要的 iOS targets (arm64, x86_64, simulator)
2. 安裝 cargo-lipo (如果尚未安裝)
3. 編譯所有架構的靜態庫
4. 創建通用的 XCFramework：`dist/ios/ratamud.xcframework`

## 在 Xcode 中整合

### 1. 添加 Framework

1. 將 `dist/ios/ratamud.xcframework` 拖入您的 Xcode 專案
2. 在 Target 設定中：
   - General → Frameworks, Libraries, and Embedded Content
   - 點擊 `+` 按鈕
   - 選擇 `ratamud.xcframework`
   - 設為 "Do Not Embed"（靜態庫不需要嵌入）

### 2. 配置 Bridging Header（Swift 專案）

如果使用 Swift，需要創建 Bridging Header：

1. File → New → File → Header File
2. 命名為 `YourProject-Bridging-Header.h`
3. 在 Build Settings 中設定 `Objective-C Bridging Header` 路徑
4. 在 Bridging Header 中導入：

```objc
#import <ratamud/ratamud.h>
```

### 3. Swift 使用範例

```swift
import Foundation

class RataMUDGame {
    private var player: OpaquePointer?
    private var world: OpaquePointer?
    
    init(playerName: String, description: String) {
        // 獲取版本
        if let version = ratamud_version() {
            print("RataMUD Version: \(String(cString: version))")
        }
        
        // 創建玩家
        player = ratamud_create_player(playerName, description)
        
        // 創建世界
        if let player = player {
            world = ratamud_create_world(player)
        }
    }
    
    deinit {
        // 清理資源
        if let world = world {
            ratamud_free_world(world)
        }
        if let player = player {
            ratamud_free_player(player)
        }
    }
    
    func loadMap(name: String) -> Bool {
        guard let world = world else { return false }
        return ratamud_load_map(world, name) == 0
    }
    
    func getPlayerInfo() -> String? {
        guard let player = player else { return nil }
        
        if let jsonPtr = ratamud_get_player_info(player) {
            let info = String(cString: jsonPtr)
            ratamud_free_string(jsonPtr)
            return info
        }
        return nil
    }
    
    func getPlayerPosition() -> (x: Int, y: Int)? {
        guard let player = player else { return nil }
        
        var x: Int32 = 0
        var y: Int32 = 0
        
        if ratamud_get_player_position(player, &x, &y) == 0 {
            return (Int(x), Int(y))
        }
        return nil
    }
    
    func setPlayerPosition(x: Int, y: Int) -> Bool {
        guard let player = player else { return false }
        return ratamud_set_player_position(player, Int32(x), Int32(y)) == 0
    }
    
    func getPlayerHP() -> Int {
        guard let player = player else { return -1 }
        return Int(ratamud_get_player_hp(player))
    }
    
    func setPlayerHP(_ hp: Int) -> Bool {
        guard let player = player else { return false }
        return ratamud_set_player_hp(player, Int32(hp)) == 0
    }
}

// 使用範例
let game = RataMUDGame(playerName: "Hero", description: "A brave adventurer")
_ = game.loadMap(name: "town")

if let info = game.getPlayerInfo() {
    print("Player Info: \(info)")
}

if let pos = game.getPlayerPosition() {
    print("Player Position: (\(pos.x), \(pos.y))")
}
```

### 4. Objective-C 使用範例

```objc
#import "ratamud.h"

@interface RataMUDGame : NSObject
@property (nonatomic, assign) Person *player;
@property (nonatomic, assign) GameWorld *world;
@end

@implementation RataMUDGame

- (instancetype)initWithPlayerName:(NSString *)name description:(NSString *)desc {
    self = [super init];
    if (self) {
        const char *version = ratamud_version();
        NSLog(@"RataMUD Version: %s", version);
        
        self.player = ratamud_create_player([name UTF8String], [desc UTF8String]);
        self.world = ratamud_create_world(self.player);
    }
    return self;
}

- (void)dealloc {
    if (self.world) {
        ratamud_free_world(self.world);
    }
    if (self.player) {
        ratamud_free_player(self.player);
    }
}

- (BOOL)loadMap:(NSString *)mapName {
    return ratamud_load_map(self.world, [mapName UTF8String]) == 0;
}

- (NSString *)getPlayerInfo {
    char *json = ratamud_get_player_info(self.player);
    if (json) {
        NSString *info = [NSString stringWithUTF8String:json];
        ratamud_free_string(json);
        return info;
    }
    return nil;
}

@end
```

## 架構支援

- **iOS Device (實機)**: arm64
- **iOS Simulator**: arm64 (Apple Silicon Mac) + x86_64 (Intel Mac)

XCFramework 會自動為不同平台選擇正確的架構。

## 注意事項

1. **記憶體管理**: 
   - 返回的字串需要用 `ratamud_free_string()` 釋放
   - Player 和 World 物件需要用對應的 free 函數釋放

2. **錯誤處理**: 
   - 返回 -1 或 null 表示錯誤
   - 返回 0 表示成功

3. **執行緒安全**: 
   - FFI 函數不保證執行緒安全
   - 建議在主執行緒或使用串行隊列調用

## 疑難排解

### 編譯錯誤
- 確保已安裝最新版本的 Rust 和 Xcode
- 運行 `rustup update`

### 連結錯誤
- 確認 XCFramework 已正確添加到專案
- 檢查 Build Settings 中的 Library Search Paths

### 找不到符號
- 確認 Bridging Header 路徑正確
- 清理並重新建構專案 (Cmd+Shift+K 然後 Cmd+B)
