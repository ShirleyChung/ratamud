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
