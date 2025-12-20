#ifndef RATAMUD_H
#define RATAMUD_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// 不透明類型
typedef struct Person Person;
typedef struct GameWorld GameWorld;
typedef struct GameEngine GameEngine;  // 新增：遊戲引擎

// ============= 回調函數類型定義 =============

/// 輸出回調函數類型
typedef void (*OutputCallback)(const char* message);

/// 狀態回調函數類型
typedef void (*StateCallback)(const char* state_json);

/// 事件回調函數類型
typedef void (*EventCallback)(const char* event_type, const char* event_data);

// ============= 回調註冊函數 =============

void ratamud_register_output_callback(OutputCallback callback);
void ratamud_unregister_output_callback(void);
void ratamud_register_state_callback(StateCallback callback);
void ratamud_unregister_state_callback(void);
void ratamud_register_event_callback(EventCallback callback);
void ratamud_unregister_event_callback(void);

// ============= 遊戲引擎 API（推薦使用）=============

/// 創建遊戲引擎（無頭模式）
GameEngine* ratamud_create_engine(const char* player_name);

/// 釋放遊戲引擎
void ratamud_free_engine(GameEngine* engine);

/// 處理命令（返回 1=繼續, 0=退出, -1=錯誤）
int ratamud_engine_process_command(GameEngine* engine, const char* command);

/// 獲取輸出（清空緩衝區）
char* ratamud_engine_get_output(GameEngine* engine);

/// 獲取遊戲狀態（JSON）
char* ratamud_engine_get_state(GameEngine* engine);

/// 更新遊戲邏輯
void ratamud_engine_update(GameEngine* engine, uint32_t delta_ms);

// ============= 版本資訊 =============

const char* ratamud_version(void);

// ============= 玩家管理（低階 API）=============

Person* ratamud_create_player(const char* name, const char* description);
void ratamud_free_player(Person* player);
char* ratamud_get_player_name(const Person* player);
char* ratamud_get_player_info(const Person* player);

// ============= 玩家屬性（低階 API）=============

int ratamud_get_player_hp(const Person* player);
int ratamud_set_player_hp(Person* player, int hp);
int ratamud_get_player_position(const Person* player, int* x, int* y);
int ratamud_set_player_position(Person* player, int x, int y);

// ============= 世界管理（低階 API）=============

GameWorld* ratamud_create_world(Person* player);
void ratamud_free_world(GameWorld* world);
int ratamud_load_map(GameWorld* world, const char* map_name);
char* ratamud_get_current_map(const GameWorld* world);

// ============= 字串記憶體管理 =============

void ratamud_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif // RATAMUD_H
