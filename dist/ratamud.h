#ifndef RATAMUD_H
#define RATAMUD_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * RataMUD C API
 * 用於跨平台移植的 C ABI 接口
 */

// 前向聲明（用於高級 API）
typedef struct Person Person;
typedef struct GameWorld GameWorld;
typedef struct GameEngine GameEngine;

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

// ===== 高級 API（直接操作 Person 和 GameWorld 對象）=====

/**
 * 創建玩家對象
 * @param name 玩家名稱
 * @param description 玩家描述
 * @return Person 指針，使用完畢後需調用 ratamud_free_player 釋放
 */
Person* ratamud_create_player(const char* name, const char* description);

/**
 * 釋放玩家對象
 * @param player 玩家指針
 */
void ratamud_free_player(Person* player);

/**
 * 創建遊戲世界
 * @param player 玩家指針
 * @return GameWorld 指針，使用完畢後需調用 ratamud_free_world 釋放
 */
GameWorld* ratamud_create_world(Person* player);

/**
 * 釋放遊戲世界
 * @param world 世界指針
 */
void ratamud_free_world(GameWorld* world);

/**
 * 載入地圖
 * @param world 世界指針
 * @param map_name 地圖名稱
 * @return 0 表示成功，非 0 表示失敗
 */
int ratamud_load_map(GameWorld* world, const char* map_name);

/**
 * 獲取玩家名稱
 * @param player 玩家指針
 * @return 玩家名稱字串，需調用 ratamud_free_string 釋放
 */
char* ratamud_get_player_name(Person* player);

/**
 * 獲取玩家資訊（JSON 格式）- 高級 API
 * @param player 玩家指針
 * @return JSON 字串，需調用 ratamud_free_string 釋放
 */
char* ratamud_player_get_info(Person* player);

/**
 * 獲取玩家 HP
 * @param player 玩家指針
 * @return HP 值
 */
int ratamud_get_player_hp(Person* player);

/**
 * 設置玩家 HP
 * @param player 玩家指針
 * @param hp HP 值
 * @return 0 表示成功，非 0 表示失敗
 */
int ratamud_set_player_hp(Person* player, int hp);

/**
 * 獲取玩家位置 - 高級 API
 * @param player 玩家指針
 * @param x 輸出參數，返回 x 坐標
 * @param y 輸出參數，返回 y 坐標
 * @return 0 表示成功，非 0 表示失敗
 */
int ratamud_player_get_position(Person* player, int* x, int* y);

/**
 * 設置玩家位置
 * @param player 玩家指針
 * @param x x 坐標
 * @param y y 坐標
 * @return 0 表示成功，非 0 表示失敗
 */
int ratamud_set_player_position(Person* player, int x, int y);

/**
 * 獲取當前地圖名稱 - 高級 API
 * @param world 世界指針
 * @return 地圖名稱字串，需調用 ratamud_free_string 釋放
 */
char* ratamud_world_get_current_map(GameWorld* world);

#ifdef __cplusplus
}
#endif

#endif /* RATAMUD_H */
