/**
 * RataMUD 核心引擎範例 (無 UI 模式)
 * 
 * 編譯方式 (使用 macOS Framework):
 *   ./build_frameworks.sh
 *   gcc -o dist/example dist/example.c -F./frameworks -framework RataMUD -Wl,-rpath,./frameworks
 *   ./dist/example
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "ratamud.h"

// 輸出計數器
static int output_count = 0;

/**
 * 輸出回調函數 - 用 printf 直接印出遊戲輸出
 * 
 * @param msg_type 訊息類型: "MAIN", "LOG", "STATUS", "SIDE"
 * @param content 訊息內容
 */
void game_output_callback(const char* msg_type, const char* content) {
    output_count++;
    
    // ANSI 顏色碼
    const char* color_reset = "\033[0m";
    const char* color = "";
    const char* prefix = "";
    
    // 根據類型選擇顏色和前綴
    if (strcmp(msg_type, "MAIN") == 0) {
        color = "\033[1;32m";  // 亮綠色
        prefix = "💬 ";
    } else if (strcmp(msg_type, "LOG") == 0) {
        color = "\033[0;36m";  // 青色
        prefix = "📝 ";
    } else if (strcmp(msg_type, "STATUS") == 0) {
        color = "\033[1;33m";  // 亮黃色
        prefix = "⚡ ";
    } else if (strcmp(msg_type, "SIDE") == 0) {
        color = "\033[0;35m";  // 紫色
        prefix = "ℹ️  ";
    }
    
    // 印出訊息
    printf("%s%s%s%s\n", color, prefix, content, color_reset);
}

int main() {
    printf("\n");
    printf("╔══════════════════════════════════════╗\n");
    printf("║   RataMUD 遊戲核心範例 (無 UI 模式)  ║\n");
    printf("╚══════════════════════════════════════╝\n");
    printf("\n");
    
    // 註冊輸出回調
    printf("🔧 註冊輸出回調函數...\n");
    ratamud_register_output_callback(game_output_callback);
    printf("✅ 回調已註冊\n");
    printf("\n");
    
    // 初始化遊戲世界
    printf("🎮 初始化遊戲世界...\n");
    printf("─────────────────────────────────────\n");
    printf("\n");
    
    int init_result = ratamud_init_game();
    if (init_result != 0) {
        printf("\033[1;31m❌ 遊戲初始化失敗\033[0m\n");
        return 1;
    }
    
    printf("\n");
    printf("─────────────────────────────────────\n");
    printf("✅ 遊戲世界初始化完成\n");
    printf("\n");
    
    // 進入遊戲互動迴圈
    printf("🎮 進入遊戲模式 (輸入 'quit' 或 'exit' 離開)\n");
    printf("═════════════════════════════════════\n");
    printf("\n");
    
    char input[256];
    while (1) {
        // 顯示提示符
        printf("\033[1;34m> \033[0m");
        fflush(stdout);
        
        // 讀取用戶輸入
        if (fgets(input, sizeof(input), stdin) == NULL) {
            break;
        }
        
        // 移除換行符
        input[strcspn(input, "\n")] = 0;
        
        // 檢查是否要退出
        if (strcmp(input, "quit") == 0 || strcmp(input, "exit") == 0) {
            printf("\n");
            printf("👋 再見！\n");
            break;
        }
        
        // 處理命令
        int result = ratamud_input_command(input);
        
        if (result < 0) {
            printf("\033[1;31m❌ 命令處理錯誤\033[0m\n");
        }
        
        printf("\n");
    }
    
    printf("\n");
    printf("─────────────────────────────────────\n");
    printf("📊 總共收到 %d 條訊息\n", output_count);
    printf("\n");
    
    // 說明訊息類型
    printf("訊息類型說明:\n");
    printf("  💬 MAIN   - 主遊戲訊息 (移動、戰鬥、對話)\n");
    printf("  📝 LOG    - 系統日誌 (帶時間戳)\n");
    printf("  ⚡ STATUS - 狀態欄訊息 (臨時訊息)\n");
    printf("  ℹ️  SIDE   - 側邊面板 (NPC 資訊等)\n");
    printf("\n");
    
    // 清除回調
    ratamud_clear_output_callback();
    printf("🔌 已清除輸出回調\n");
    printf("\n");
    
    return 0;
}
