#!/bin/bash
# 構建動態連結函式庫的腳本

set -e

echo "開始構建 RataMUD 動態連結函式庫..."

# 設定顏色
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 構建類型（debug 或 release）
BUILD_TYPE="${1:-release}"

echo -e "${BLUE}構建類型: ${BUILD_TYPE}${NC}"

# 根據平台設定輸出檔案名稱
case "$(uname -s)" in
    Darwin*)
        LIB_PREFIX="lib"
        LIB_EXT="dylib"
        PLATFORM="macOS"
        ;;
    Linux*)
        LIB_PREFIX="lib"
        LIB_EXT="so"
        PLATFORM="Linux"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        LIB_PREFIX=""
        LIB_EXT="dll"
        PLATFORM="Windows"
        ;;
    *)
        echo "未支援的平台: $(uname -s)"
        exit 1
        ;;
esac

echo -e "${BLUE}目標平台: ${PLATFORM}${NC}"

# 執行構建
if [ "$BUILD_TYPE" = "release" ]; then
    echo -e "${YELLOW}執行 cargo build --release --lib${NC}"
    cargo build --release --lib
    BUILD_DIR="target/release"
else
    echo -e "${YELLOW}執行 cargo build --lib${NC}"
    cargo build --lib
    BUILD_DIR="target/debug"
fi

# 檢查構建是否成功
LIB_FILE="${BUILD_DIR}/${LIB_PREFIX}ratamud.${LIB_EXT}"
if [ ! -f "$LIB_FILE" ]; then
    echo "錯誤: 找不到生成的函式庫檔案 $LIB_FILE"
    exit 1
fi

echo -e "${GREEN}✓ 構建成功！${NC}"
echo -e "${GREEN}動態連結函式庫位置: ${LIB_FILE}${NC}"

# 顯示檔案資訊
echo ""
echo "檔案資訊:"
ls -lh "$LIB_FILE"

# 建立輸出目錄
OUTPUT_DIR="dist"
mkdir -p "$OUTPUT_DIR"

# 複製函式庫到輸出目錄
cp "$LIB_FILE" "$OUTPUT_DIR/"
echo -e "${GREEN}✓ 已複製到 ${OUTPUT_DIR}/${NC}"

# 生成 C 標頭檔
HEADER_FILE="${OUTPUT_DIR}/ratamud.h"
echo "生成 C 標頭檔: ${HEADER_FILE}"

cat > "$HEADER_FILE" << 'EOF'
#ifndef RATAMUD_H
#define RATAMUD_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * RataMUD C API
 * 用於跨平台移植的 C ABI 接口
 */

/**
 * 初始化遊戲
 * @return 0 表示成功，非 0 表示失敗
 */
int ratamud_init(void);

/**
 * 清理遊戲資源
 */
void ratamud_cleanup(void);

/**
 * 處理玩家輸入命令
 * @param command UTF-8 編碼的命令字串
 * @return 0 表示成功，非 0 表示失敗
 */
int ratamud_process_command(const char* command);

/**
 * 獲取遊戲輸出訊息
 * @return UTF-8 編碼的字串指針，呼叫者需要用 ratamud_free_string 釋放
 */
char* ratamud_get_output(void);

/**
 * 獲取玩家位置
 * @param x 輸出參數，返回 x 坐標
 * @param y 輸出參數，返回 y 坐標
 * @return 0 表示成功，非 0 表示失敗
 */
int ratamud_get_player_position(int* x, int* y);

/**
 * 獲取當前地圖名稱
 * @return UTF-8 編碼的字串指針，呼叫者需要用 ratamud_free_string 釋放
 */
char* ratamud_get_current_map(void);

/**
 * 獲取玩家資訊（JSON 格式）
 * @return UTF-8 編碼的 JSON 字串指針，呼叫者需要用 ratamud_free_string 釋放
 */
char* ratamud_get_player_info(void);

/**
 * 釋放由 ratamud_* 函數分配的字串
 * @param s 要釋放的字串指針
 */
void ratamud_free_string(char* s);

/**
 * 更新遊戲狀態（每幀調用）
 * @param delta_ms 自上次更新以來的毫秒數
 * @return 0 表示繼續，非 0 表示應該退出
 */
int ratamud_update(int delta_ms);

/**
 * 獲取版本資訊
 * @return 版本字串（靜態字串，不需要釋放）
 */
const char* ratamud_version(void);

#ifdef __cplusplus
}
#endif

#endif /* RATAMUD_H */
EOF

echo -e "${GREEN}✓ C 標頭檔已生成${NC}"

# 生成使用範例
EXAMPLE_FILE="${OUTPUT_DIR}/example.c"
echo "生成使用範例: ${EXAMPLE_FILE}"

cat > "$EXAMPLE_FILE" << 'EOF'
#include <stdio.h>
#include <stdlib.h>
#include "ratamud.h"

int main() {
    printf("RataMUD C API 使用範例\n");
    printf("版本: %s\n\n", ratamud_version());
    
    // 初始化遊戲
    if (ratamud_init() != 0) {
        fprintf(stderr, "遊戲初始化失敗\n");
        return 1;
    }
    printf("✓ 遊戲初始化成功\n");
    
    // 獲取玩家資訊
    char* info = ratamud_get_player_info();
    if (info) {
        printf("玩家資訊: %s\n", info);
        ratamud_free_string(info);
    }
    
    // 獲取玩家位置
    int x, y;
    if (ratamud_get_player_position(&x, &y) == 0) {
        printf("玩家位置: (%d, %d)\n", x, y);
    }
    
    // 獲取當前地圖
    char* map = ratamud_get_current_map();
    if (map) {
        printf("當前地圖: %s\n", map);
        ratamud_free_string(map);
    }
    
    // 處理命令
    printf("\n執行命令: look\n");
    ratamud_process_command("look");
    
    char* output = ratamud_get_output();
    if (output) {
        printf("輸出: %s\n", output);
        ratamud_free_string(output);
    }
    
    // 清理資源
    ratamud_cleanup();
    printf("\n✓ 遊戲已清理\n");
    
    return 0;
}
EOF

echo -e "${GREEN}✓ 使用範例已生成${NC}"

# 生成編譯說明
README_FILE="${OUTPUT_DIR}/README.md"
cat > "$README_FILE" << EOF
# RataMUD 動態連結函式庫

## 檔案說明

- \`${LIB_PREFIX}ratamud.${LIB_EXT}\` - 動態連結函式庫
- \`ratamud.h\` - C 標頭檔
- \`example.c\` - 使用範例

## 使用方式

### macOS/Linux

編譯範例程式:
\`\`\`bash
gcc -o example example.c -L. -lratamud -Wl,-rpath,.
\`\`\`

執行:
\`\`\`bash
./example
\`\`\`

### iOS 開發

1. 將 \`${LIB_PREFIX}ratamud.${LIB_EXT}\` 添加到 Xcode 項目
2. 包含 \`ratamud.h\` 標頭檔
3. 在 Swift 中使用 Bridging Header 調用

Swift 範例:
\`\`\`swift
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
\`\`\`

### Android 開發

使用 JNI 包裝:
\`\`\`java
public class RataMUD {
    static {
        System.loadLibrary("ratamud");
    }
    
    public native int init();
    public native void cleanup();
    public native int processCommand(String command);
    public native String getOutput();
}
\`\`\`

## 跨平台編譯

### iOS (使用 cargo-lipo)
\`\`\`bash
cargo install cargo-lipo
cargo lipo --release
\`\`\`

### Android (使用 cargo-ndk)
\`\`\`bash
cargo install cargo-ndk
cargo ndk --target aarch64-linux-android --platform 21 -- build --release
\`\`\`

### Windows
\`\`\`bash
cargo build --release --lib --target x86_64-pc-windows-msvc
\`\`\`

## API 文檔

查看 \`ratamud.h\` 獲取完整的 API 文檔。

## 授權

與 RataMUD 主項目相同。
EOF

echo -e "${GREEN}✓ README 已生成${NC}"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}構建完成！${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "輸出目錄: ${OUTPUT_DIR}/"
echo -e "包含檔案:"
echo -e "  - ${LIB_PREFIX}ratamud.${LIB_EXT} (動態連結函式庫)"
echo -e "  - ratamud.h (C 標頭檔)"
echo -e "  - example.c (使用範例)"
echo -e "  - README.md (使用說明)"
echo ""
echo -e "${YELLOW}提示: 可使用以下命令測試範例程式${NC}"
echo -e "  cd ${OUTPUT_DIR}"
echo -e "  gcc -o example example.c -L. -lratamud"
echo -e "  ${PLATFORM_RUN_CMD}./example"
