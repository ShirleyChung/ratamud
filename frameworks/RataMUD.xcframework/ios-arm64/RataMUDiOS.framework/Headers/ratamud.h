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

/// 輸出回調函數類型（帶類型標記）
/// msg_type: 輸出類型 ("MAIN", "LOG", "STATUS", "SIDE")
/// content: 輸出內容
typedef void (*OutputCallback)(const char* msg_type, const char* content);

/// 狀態回調函數類型
typedef void (*StateCallback)(const char* state_json);

/// 事件回調函數類型
typedef void (*EventCallback)(const char* event_type, const char* event_data);

/// 面板回調函數類型
/// panel_type: "MAP", "MINIMAP", "INVENTORY", "STATUS", "TRADE"
/// content: ASCII 純文字面板內容
typedef void (*PanelCallback)(const char* panel_type, const char* content);

// ============= 回調註冊函數 =============
void ratamud_register_output_callback(OutputCallback callback);
void ratamud_clear_output_callback(void);
void ratamud_register_state_callback(StateCallback callback);
void ratamud_register_event_callback(EventCallback callback);
void ratamud_register_panel_callback(PanelCallback callback);
void ratamud_clear_panel_callback(void);

// ============= 面板請求 / 交易 API =============
/// 請求面板內容（透過 panel callback 推回，回傳 0=成功, -1=失敗）
int ratamud_request_map(void);
int ratamud_request_minimap(void);
int ratamud_request_inventory(void);
int ratamud_request_status(void);
/// npc 可為空字串，引擎會自動偵測當前格的商人
int ratamud_request_trade(const char* npc);
/// 買/賣（回傳 1=成功, 0=交易失敗, -1=錯誤），成功後會自動刷新 TRADE/INVENTORY 面板
int ratamud_trade_buy(const char* npc, const char* item, int qty);
int ratamud_trade_sell(const char* npc, const char* item, int qty);

// ============= 遊戲引擎 API（推薦使用）=============

/// 處理命令（返回 1=繼續, 0=退出, -1=錯誤）
int ratamud_input_command(const char* command);

/// 初始化無 UI 遊戲世界（返回 0=成功, -1=失敗）
int ratamud_init_game(void);

/// 推進無 UI 遊戲循環一次（返回 0=成功, -1=失敗）
int ratamud_tick(void);

int ratamud_start_game(void);

/// 測試輸出回調功能（會生成各種類型的測試輸出）
void ratamud_test_output_callback(void);

#ifdef __cplusplus
}
#endif

#endif // RATAMUD_H
