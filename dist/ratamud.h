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
