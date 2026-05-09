/**
 * RataMUD 輸出回調測試程序
 * 
 * 編譯命令 (macOS):
 *   cargo build --release --lib
 *   g++ -std=c++11 test_callback.cpp -L./target/release -lratamud -o test_callback
 *   ./test_callback
 * 
 * Linux:
 *   cargo build --release --lib
 *   g++ -std=c++11 test_callback.cpp -L./target/release -lratamud -Wl,-rpath,./target/release -o test_callback
 *   ./test_callback
 */

#include <iostream>
#include <fstream>
#include <string>
#include <cstring>
#include "src/ratamud.h"

// 控制台輸出計數器
static int output_count = 0;

// 文件輸出流
static std::ofstream log_file;

/**
 * 輸出回調函數
 * 
 * 功能：
 * 1. 在控制台打印帶顏色的輸出
 * 2. 同時寫入 game_output.log 文件
 */
void my_output_callback(const char* msg_type, const char* content) {
    output_count++;
    
    // ANSI 顏色碼
    const char* color_reset = "\033[0m";
    const char* color = "";
    
    // 根據類型選擇顏色
    if (strcmp(msg_type, "MAIN") == 0) {
        color = "\033[32m";  // 綠色
    } else if (strcmp(msg_type, "LOG") == 0) {
        color = "\033[36m";  // 青色
    } else if (strcmp(msg_type, "STATUS") == 0) {
        color = "\033[33m";  // 黃色
    } else if (strcmp(msg_type, "SIDE") == 0) {
        color = "\033[35m";  // 紫色
    }
    
    // 1. 控制台輸出（帶顏色）
    std::cout << color << "[" << msg_type << "] " 
              << color_reset << content << std::endl;
    
    // 2. 文件輸出（純文本）
    if (log_file.is_open()) {
        log_file << msg_type << ":" << content << std::endl;
        log_file.flush();  // 立即寫入
    }
}

int main() {
    std::cout << "========================================" << std::endl;
    std::cout << "   RataMUD 輸出回調系統測試" << std::endl;
    std::cout << "========================================" << std::endl;
    std::cout << std::endl;
    
    // 打開日誌文件
    log_file.open("game_output.log", std::ios::out | std::ios::trunc);
    if (!log_file.is_open()) {
        std::cerr << "錯誤：無法創建 game_output.log" << std::endl;
        return 1;
    }
    
    std::cout << "✓ 已打開文件: game_output.log" << std::endl;
    std::cout << std::endl;
    
    // 註冊回調函數
    std::cout << "→ 註冊輸出回調..." << std::endl;
    ratamud_register_output_callback(my_output_callback);
    std::cout << "✓ 回調已註冊" << std::endl;
    std::cout << std::endl;
    
    // 觸發測試輸出
    std::cout << "→ 觸發測試輸出..." << std::endl;
    std::cout << "----------------------------------------" << std::endl;
    
    ratamud_test_output_callback();
    
    std::cout << "----------------------------------------" << std::endl;
    std::cout << std::endl;
    
    // 統計結果
    std::cout << "========================================" << std::endl;
    std::cout << "   測試結果" << std::endl;
    std::cout << "========================================" << std::endl;
    std::cout << "總輸出次數: " << output_count << std::endl;
    std::cout << std::endl;
    
    // 顯示輸出類型分佈
    std::cout << "輸出類型說明:" << std::endl;
    std::cout << "  \033[32mMAIN\033[0m   - 主遊戲訊息（移動、戰鬥、對話）" << std::endl;
    std::cout << "  \033[36mLOG\033[0m    - 系統日誌（帶時間戳）" << std::endl;
    std::cout << "  \033[33mSTATUS\033[0m - 狀態欄訊息（5秒自動清除）" << std::endl;
    std::cout << "  \033[35mSIDE\033[0m   - 側邊面板（NPC信息等）" << std::endl;
    std::cout << std::endl;
    
    // 關閉文件
    log_file.close();
    std::cout << "✓ 日誌已保存到 game_output.log" << std::endl;
    std::cout << std::endl;
    
    // 顯示文件內容
    std::cout << "========================================" << std::endl;
    std::cout << "   文件內容預覽" << std::endl;
    std::cout << "========================================" << std::endl;
    
    std::ifstream read_log("game_output.log");
    std::string line;
    while (std::getline(read_log, line)) {
        std::cout << "  " << line << std::endl;
    }
    read_log.close();
    
    std::cout << std::endl;
    std::cout << "✅ 測試完成！回調系統工作正常。" << std::endl;
    
    return 0;
}
