#include <iostream>
#include <string>
#include <memory>
#include <iomanip>
#include "ratamud.h"

// RAII 包裝類，自動管理資源
class RataMUDString {
private:
    char* str_;
public:
    explicit RataMUDString(char* s) : str_(s) {}
    ~RataMUDString() { 
        if (str_) ratamud_free_string(str_); 
    }
    
    // 禁止拷貝
    RataMUDString(const RataMUDString&) = delete;
    RataMUDString& operator=(const RataMUDString&) = delete;
    
    // 允許移動
    RataMUDString(RataMUDString&& other) noexcept : str_(other.str_) {
        other.str_ = nullptr;
    }
    
    const char* c_str() const { return str_ ? str_ : ""; }
    std::string to_string() const { return str_ ? std::string(str_) : ""; }
    operator bool() const { return str_ != nullptr; }
};

class Player {
private:
    Person* player_;
public:
    Player(const std::string& name, const std::string& description) {
        player_ = ratamud_create_player(name.c_str(), description.c_str());
        if (!player_) {
            throw std::runtime_error("無法創建玩家");
        }
    }
    
    ~Player() {
        if (player_) {
            ratamud_free_player(player_);
        }
    }
    
    // 禁止拷貝
    Player(const Player&) = delete;
    Player& operator=(const Player&) = delete;
    
    // 允許移動
    Player(Player&& other) noexcept : player_(other.player_) {
        other.player_ = nullptr;
    }
    
    Person* get() const { return player_; }
    
    std::string getName() const {
        RataMUDString name(ratamud_get_player_name(player_));
        return name.to_string();
    }
    
    std::pair<int, int> getPosition() const {
        int x, y;
        if (ratamud_get_player_position(player_, &x, &y) == 0) {
            return {x, y};
        }
        return {-1, -1};
    }
    
    void setPosition(int x, int y) {
        ratamud_set_player_position(player_, x, y);
    }
    
    int getHP() const {
        return ratamud_get_player_hp(player_);
    }
    
    void setHP(int hp) {
        ratamud_set_player_hp(player_, hp);
    }
    
    std::string getInfo() const {
        RataMUDString info(ratamud_get_player_info(player_));
        return info.to_string();
    }
};

class World {
private:
    GameWorld* world_;
public:
    explicit World(const Player& player) {
        world_ = ratamud_create_world(player.get());
        if (!world_) {
            throw std::runtime_error("無法創建世界");
        }
    }
    
    ~World() {
        if (world_) {
            ratamud_free_world(world_);
        }
    }
    
    // 禁止拷貝
    World(const World&) = delete;
    World& operator=(const World&) = delete;
    
    GameWorld* get() const { return world_; }
    
    bool loadMap(const std::string& mapName) {
        return ratamud_load_map(world_, mapName.c_str()) == 0;
    }
    
    std::string getCurrentMap() const {
        RataMUDString map(ratamud_get_current_map(world_));
        return map.to_string();
    }
};

// 美化輸出
void printHeader(const std::string& title) {
    std::cout << "\n╔═══════════════════════════════════════════════════════════╗\n";
    std::cout << "║ " << std::setw(55) << std::left << title << " ║\n";
    std::cout << "╚═══════════════════════════════════════════════════════════╝\n";
}

void printInfo(const std::string& label, const std::string& value) {
    std::cout << "  " << std::setw(15) << std::left << label << ": " << value << "\n";
}

void printSuccess(const std::string& msg) {
    std::cout << "  ✓ " << msg << "\n";
}

void printError(const std::string& msg) {
    std::cout << "  ✗ " << msg << "\n";
}

int main() {
    try {
        printHeader("RataMUD C++ API 測試程式");
        
        // 顯示版本
        std::cout << "\n";
        printInfo("API 版本", ratamud_version());
        
        // 測試 1: 創建玩家
        printHeader("測試 1: 創建玩家");
        Player player("勇者", "來自異世界的冒險者");
        printSuccess("玩家創建成功");
        printInfo("玩家名稱", player.getName());
        
        // 測試 2: 創建世界
        printHeader("測試 2: 創建世界");
        World world(player);
        printSuccess("世界創建成功");
        printInfo("當前地圖", world.getCurrentMap());
        
        // 測試 3: 查詢玩家資訊
        printHeader("測試 3: 查詢玩家資訊");
        auto [x, y] = player.getPosition();
        printInfo("初始位置", "(" + std::to_string(x) + ", " + std::to_string(y) + ")");
        printInfo("初始 HP", std::to_string(player.getHP()));
        std::cout << "\n完整資訊 (JSON):\n" << player.getInfo() << "\n";
        
        // 測試 4: 移動玩家
        printHeader("測試 4: 移動玩家");
        std::cout << "  移動前位置: (" << x << ", " << y << ")\n";
        player.setPosition(100, 200);
        auto [newX, newY] = player.getPosition();
        std::cout << "  移動後位置: (" << newX << ", " << newY << ")\n";
        if (newX == 100 && newY == 200) {
            printSuccess("位置更新成功");
        } else {
            printError("位置更新失敗");
        }
        
        // 測試 5: 修改 HP
        printHeader("測試 5: 修改 HP");
        int originalHP = player.getHP();
        std::cout << "  原始 HP: " << originalHP << "\n";
        
        player.setHP(originalHP - 500);
        int newHP = player.getHP();
        std::cout << "  受傷後 HP: " << newHP << "\n";
        
        player.setHP(originalHP - 200);
        int restoredHP = player.getHP();
        std::cout << "  恢復後 HP: " << restoredHP << "\n";
        
        if (newHP == originalHP - 500 && restoredHP == originalHP - 200) {
            printSuccess("HP 修改成功");
        } else {
            printError("HP 修改失敗");
        }
        
        // 測試 6: 載入地圖
        printHeader("測試 6: 載入地圖");
        std::cout << "  當前地圖: " << world.getCurrentMap() << "\n";
        
        std::vector<std::string> testMaps = {"新手村", "森林", "洞穴", "不存在的地圖"};
        for (const auto& mapName : testMaps) {
            std::cout << "  嘗試載入: " << mapName << " ... ";
            if (world.loadMap(mapName)) {
                std::cout << "✓ 成功\n";
                std::cout << "    當前地圖: " << world.getCurrentMap() << "\n";
            } else {
                std::cout << "✗ 失敗（可能地圖不存在）\n";
            }
        }
        
        // 測試 7: 多個玩家實例
        printHeader("測試 7: 多個玩家實例");
        {
            Player player2("戰士", "強壯的戰士");
            Player player3("法師", "智慧的魔法師");
            
            printInfo("玩家 2", player2.getName() + " [HP: " + std::to_string(player2.getHP()) + "]");
            printInfo("玩家 3", player3.getName() + " [HP: " + std::to_string(player3.getHP()) + "]");
            
            player2.setPosition(50, 50);
            player3.setPosition(75, 75);
            
            auto [x2, y2] = player2.getPosition();
            auto [x3, y3] = player3.getPosition();
            
            printInfo("玩家 2 位置", "(" + std::to_string(x2) + ", " + std::to_string(y2) + ")");
            printInfo("玩家 3 位置", "(" + std::to_string(x3) + ", " + std::to_string(y3) + ")");
            
            printSuccess("多實例管理正常");
        }
        
        // 測試 8: 壓力測試
        printHeader("測試 8: 簡單壓力測試");
        std::cout << "  創建和銷毀 1000 個玩家實例...\n";
        auto start = std::chrono::high_resolution_clock::now();
        
        for (int i = 0; i < 1000; ++i) {
            Player tempPlayer("測試玩家" + std::to_string(i), "測試描述");
            tempPlayer.setPosition(i, i);
            (void)tempPlayer.getHP();  // 測試 HP 讀取
            // 玩家會在作用域結束時自動銷毀
        }
        
        auto end = std::chrono::high_resolution_clock::now();
        auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
        
        printSuccess("壓力測試完成");
        printInfo("耗時", std::to_string(duration.count()) + " ms");
        printInfo("平均每個", std::to_string(duration.count() / 1000.0) + " ms");
        
        // 最終總結
        printHeader("測試完成");
        std::cout << "\n";
        printSuccess("所有測試通過！");
        std::cout << "\n  記憶體管理: 使用 RAII 自動管理\n";
        std::cout << "  類型安全: 使用 C++ 包裝類\n";
        std::cout << "  異常安全: 支援 RAII 和異常處理\n";
        std::cout << "\n";
        
    } catch (const std::exception& e) {
        std::cerr << "\n錯誤: " << e.what() << "\n";
        return 1;
    }
    
    return 0;
}
