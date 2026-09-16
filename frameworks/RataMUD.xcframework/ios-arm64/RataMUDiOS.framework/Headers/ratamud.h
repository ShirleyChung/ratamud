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
/// panel_type: "MAP", "MAP_JSON", "MINIMAP", "INVENTORY", "STATUS", "TRADE"
/// content: 面板內容（"MAP_JSON" 是 JSON，其餘是 ASCII 純文字）
typedef void (*PanelCallback)(const char* panel_type, const char* content);

// ============= 回調註冊函數 =============
void ratamud_register_output_callback(OutputCallback callback);
void ratamud_clear_output_callback(void);
void ratamud_register_state_callback(StateCallback callback);
void ratamud_register_event_callback(EventCallback callback);
void ratamud_register_panel_callback(PanelCallback callback);
void ratamud_clear_panel_callback(void);

// 設定 host 目前開啟的面板（空字串 = 沒有開）。世界變動時引擎會自動重渲染並推回。
void ratamud_set_active_panel(const char* panel);

// ============= 面板請求 / 交易 API =============

/// 設定大地圖視窗大小（以玩家為中心的格數），傳 0 使用引擎預設值 (41x21)。
/// 整張地圖是 100x100，不建議一次全部取回。
void ratamud_set_map_view_size(int width, int height);

/// 請求面板內容（透過 panel callback 推回，回傳 0=成功, -1=失敗）
int ratamud_request_map(void);
/// 結構化地圖資料（JSON），以 "MAP_JSON" 推回，適合自己畫格子的 host
int ratamud_request_map_json(void);
int ratamud_request_minimap(void);
int ratamud_request_inventory(void);
int ratamud_request_status(void);
/// npc 可為空字串，引擎會自動偵測當前格的商人
int ratamud_request_trade(const char* npc);
/// 買/賣（回傳 1=成功, 0=交易失敗, -1=錯誤），成功後會自動刷新 TRADE/INVENTORY 面板
int ratamud_trade_buy(const char* npc, const char* item, int qty);
int ratamud_trade_sell(const char* npc, const char* item, int qty);

// ============= 資料目錄（iOS 沙盒必要）=============
//
// 引擎預設用相對路徑 "worlds/..."，在 iOS 上一定失敗（工作目錄唯讀，也不是
// app bundle 的位置）。host 必須先設定可寫目錄，否則地圖載不進來、狀態也存
// 不起來。典型用法：
//
//   let docs = FileManager.default.urls(for: .documentDirectory,
//                                       in: .userDomainMask)[0].path
//   ratamud_init_game_with_dir(docs, Bundle.main.resourcePath)
//
/// 設定資料根目錄（必須在 ratamud_init_game() 之前）。回傳 0=成功, -1=失敗
int ratamud_set_data_dir(const char* path);
/// 取得目前的資料根目錄；用完請以 ratamud_free_string() 釋放，失敗回傳 NULL
char* ratamud_get_data_dir(void);
/// 釋放本函式庫回傳的字串
void ratamud_free_string(char* ptr);
/// 把 bundle_dir/worlds 複製到資料目錄（不覆蓋既有檔案，可重複呼叫）。
/// 回傳複製的檔案數，失敗回傳 -1
int ratamud_seed_data_dir(const char* bundle_dir);
/// set_data_dir + seed_data_dir + init_game 一次完成（bundle_dir 可為 NULL）
int ratamud_init_game_with_dir(const char* data_dir, const char* bundle_dir);

// ============= 遊戲引擎 API（推薦使用）=============

/// 處理命令（返回 1=繼續, 0=退出, -1=錯誤）
int ratamud_input_command(const char* command);

/// 初始化無 UI 遊戲世界（返回 0=成功, -1=失敗）
/// iOS 請改用 ratamud_init_game_with_dir()
int ratamud_init_game(void);

/// 推進遊戲世界一次：時間、世界事件、NPC AI、戰鬥回合、面板刷新。
/// host 大約每秒呼叫一次。（返回 0=成功, -1=失敗）
int ratamud_tick(void);

/// 立刻把狀態完整寫回磁碟（含有變動的地圖）。
/// iOS 請在 scenePhase 變成 .background / .inactive 時呼叫。
/// 返回 0=成功, -1=失敗
int ratamud_save(void);

int ratamud_start_game(void);

/// 測試輸出回調功能（會生成各種類型的測試輸出）
void ratamud_test_output_callback(void);

#ifdef __cplusplus
}
#endif

#endif // RATAMUD_H
