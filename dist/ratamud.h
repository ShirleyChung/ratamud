#ifndef RATAMUD_H
#define RATAMUD_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * RataMUD C API
 * 用於跨平台移植的 C ABI 接口
 */

// 不透明類型
typedef struct Person Person;
typedef struct GameWorld GameWorld;

/**
 * 創建新玩家
 * @param name 玩家名稱 (UTF-8)
 * @param description 玩家描述 (UTF-8)
 * @return 玩家指針，失敗則返回 NULL
 */
Person* ratamud_create_player(const char* name, const char* description);

/**
 * 創建遊戲世界
 * @param player 玩家指針
 * @return 世界指針，失敗則返回 NULL
 */
GameWorld* ratamud_create_world(Person* player);

/**
 * 釋放玩家
 * @param player 玩家指針
 */
void ratamud_free_player(Person* player);

/**
 * 釋放遊戲世界
 * @param world 世界指針
 */
void ratamud_free_world(GameWorld* world);

/**
 * 載入地圖
 * @param world 世界指針
 * @param map_name 地圖名稱 (UTF-8)
 * @return 0 表示成功，非 0 表示失敗
 */
int ratamud_load_map(GameWorld* world, const char* map_name);

/**
 * 獲取玩家位置
 * @param player 玩家指針
 * @param x 輸出參數，返回 x 坐標
 * @param y 輸出參數，返回 y 坐標
 * @return 0 表示成功，非 0 表示失敗
 */
int ratamud_get_player_position(const Person* player, int* x, int* y);

/**
 * 設置玩家位置
 * @param player 玩家指針
 * @param x x 坐標
 * @param y y 坐標
 * @return 0 表示成功，非 0 表示失敗
 */
int ratamud_set_player_position(Person* player, int x, int y);

/**
 * 獲取當前地圖名稱
 * @param world 世界指針
 * @return UTF-8 編碼的字串指針，呼叫者需要用 ratamud_free_string 釋放
 */
char* ratamud_get_current_map(const GameWorld* world);

/**
 * 獲取玩家資訊（JSON 格式）
 * @param player 玩家指針
 * @return UTF-8 編碼的 JSON 字串指針，呼叫者需要用 ratamud_free_string 釋放
 */
char* ratamud_get_player_info(const Person* player);

/**
 * 獲取玩家名稱
 * @param player 玩家指針
 * @return UTF-8 編碼的字串指針，呼叫者需要用 ratamud_free_string 釋放
 */
char* ratamud_get_player_name(const Person* player);

/**
 * 獲取玩家 HP
 * @param player 玩家指針
 * @return HP 值，失敗返回 -1
 */
int ratamud_get_player_hp(const Person* player);

/**
 * 設置玩家 HP
 * @param player 玩家指針
 * @param hp HP 值
 * @return 0 表示成功，非 0 表示失敗
 */
int ratamud_set_player_hp(Person* player, int hp);

/**
 * 釋放由 ratamud_* 函數分配的字串
 * @param s 要釋放的字串指針
 */
void ratamud_free_string(char* s);

/**
 * 獲取版本資訊
 * @return 版本字串（靜態字串，不需要釋放）
 */
const char* ratamud_version(void);

#ifdef __cplusplus
}
#endif

#endif /* RATAMUD_H */
