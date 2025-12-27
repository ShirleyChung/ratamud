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

// ============= 回調函數類型定義 =============

/// 輸出回調函數類型
typedef void (*OutputCallback)(const char* message);

/// 狀態回調函數類型
typedef void (*StateCallback)(const char* state_json);

/// 事件回調函數類型
typedef void (*EventCallback)(const char* event_type, const char* event_data);

// ============= 回調註冊函數 =============
void ratamud_register_output_callback(OutputCallback callback);
void ratamud_register_state_callback(StateCallback callback);
void ratamud_register_event_callback(EventCallback callback);

// ============= 遊戲引擎 API（推薦使用）=============

/// 處理命令（返回 1=繼續, 0=退出, -1=錯誤）
int ratamud_input_command(const char* command);

void ratamud_start_game(void);

#ifdef __cplusplus
}
#endif

#endif // RATAMUD_H
