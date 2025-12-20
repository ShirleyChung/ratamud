#include <stdio.h>
#include <stdlib.h>
#include "ratamud.h"

int main() {
    printf("RataMUD C API 使用範例\n");
    printf("版本: %s\n\n", ratamud_version());
    
    // 創建玩家
    Person* player = ratamud_create_player("冒險者", "勇敢的冒險者");
    if (!player) {
        fprintf(stderr, "創建玩家失敗\n");
        return 1;
    }
    printf("✓ 玩家創建成功\n");
    
    // 獲取玩家名稱
    char* name = ratamud_get_player_name(player);
    if (name) {
        printf("玩家名稱: %s\n", name);
        ratamud_free_string(name);
    }
    
    // 創建遊戲世界
    GameWorld* world = ratamud_create_world(player);
    if (!world) {
        fprintf(stderr, "創建世界失敗\n");
        ratamud_free_player(player);
        return 1;
    }
    printf("✓ 世界創建成功\n");
    
    // 獲取玩家資訊
    char* info = ratamud_get_player_info(player);
    if (info) {
        printf("玩家資訊: %s\n", info);
        ratamud_free_string(info);
    }
    
    // 獲取玩家位置
    int x, y;
    if (ratamud_get_player_position(player, &x, &y) == 0) {
        printf("玩家位置: (%d, %d)\n", x, y);
    }
    
    // 移動玩家
    printf("\n移動玩家到 (10, 20)\n");
    ratamud_set_player_position(player, 10, 20);
    if (ratamud_get_player_position(player, &x, &y) == 0) {
        printf("新位置: (%d, %d)\n", x, y);
    }
    
    // 獲取當前地圖
    char* map = ratamud_get_current_map(world);
    if (map) {
        printf("當前地圖: %s\n", map);
        ratamud_free_string(map);
    }
    
    // 獲取玩家 HP
    int hp = ratamud_get_player_hp(player);
    printf("玩家 HP: %d\n", hp);
    
    // 設置玩家 HP
    ratamud_set_player_hp(player, hp - 10);
    printf("受到傷害後 HP: %d\n", ratamud_get_player_hp(player));
    
    // 載入地圖（測試多個地圖）
    const char* maps_to_try[] = {"初始之地", "森林", "洞穴", "沙漠"};
    int maps_count = sizeof(maps_to_try) / sizeof(maps_to_try[0]);
    
    printf("\n嘗試載入地圖...\n");
    int map_loaded = 0;
    for (int i = 0; i < maps_count; i++) {
        printf("嘗試載入: %s...", maps_to_try[i]);
        if (ratamud_load_map(world, maps_to_try[i]) == 0) {
            printf(" ✓ 成功\n");
            char* new_map = ratamud_get_current_map(world);
            if (new_map) {
                printf("當前地圖: %s\n", new_map);
                ratamud_free_string(new_map);
            }
            map_loaded = 1;
            break;
        } else {
            printf(" ✗ 失敗\n");
        }
    }
    
    if (!map_loaded) {
        printf("注意: 所有地圖載入都失敗（可能需要在正確的目錄運行程序）\n");
        printf("請確保從專案根目錄執行程式，以便訪問 worlds/初始世界/maps/ 目錄\n");
    }
    
    // 清理資源
    ratamud_free_world(world);
    ratamud_free_player(player);
    printf("\n✓ 資源已清理\n");
    
    return 0;
}
