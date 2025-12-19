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

// 版本資訊
const char* ratamud_version(void);

// 玩家管理
Person* ratamud_create_player(const char* name, const char* description);
void ratamud_free_player(Person* player);
char* ratamud_get_player_name(const Person* player);
char* ratamud_get_player_info(const Person* player);

// 玩家屬性
int ratamud_get_player_hp(const Person* player);
int ratamud_set_player_hp(Person* player, int hp);
int ratamud_get_player_position(const Person* player, int* x, int* y);
int ratamud_set_player_position(Person* player, int x, int y);

// 世界管理
GameWorld* ratamud_create_world(Person* player);
void ratamud_free_world(GameWorld* world);
int ratamud_load_map(GameWorld* world, const char* map_name);
char* ratamud_get_current_map(const GameWorld* world);

// 字串記憶體管理
void ratamud_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif // RATAMUD_H
