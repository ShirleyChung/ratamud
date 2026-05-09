# 隨機技能台詞系統

## 功能
戰鬥技能台詞會從設置的台詞列表中隨機選擇，讓戰鬥更加生動活潑。

## 實作

### 修改前
```rust
pub fn get_skill_dialogue(&self, skill_name: &str) -> String {
    // 只取第一個台詞
    if let Some(option) = dialogue_options.first() {
        return option.text.clone();
    }
}
```

### 修改後
```rust
pub fn get_skill_dialogue(&self, skill_name: &str) -> String {
    // 隨機選擇一個台詞
    if !dialogue_options.is_empty() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..dialogue_options.len());
        return dialogue_options[index].text.clone();
    }
}
```

## 使用方式

### 設置多個技能台詞
```
sdl me punch 吃我一拳
sdl me punch 嘗嘗我的鐵拳
sdl me punch 看拳！
sdl me punch 哼！

sdl me kick 看我的飛踢
sdl me kick 旋風腿！
sdl me kick 踢你一腳
```

### 設置NPC台詞
```
sdl 商人 punch 你敢打我？
sdl 商人 punch 住手！
sdl 商人 punch 救命啊！

sdl 商人 kick 別欺負老實人
sdl 商人 kick 我也會功夫
```

## 戰鬥效果

### 玩家攻擊（隨機台詞）
```
> punch 商人
💥 me 說：「吃我一拳」造成 2 點傷害！

> punch 商人
💥 me 說：「嘗嘗我的鐵拳」造成 2 點傷害！

> punch 商人
💥 me 說：「看拳！」造成 2 點傷害！

> kick 商人
�� me 說：「看我的飛踢」造成 3 點傷害！

> kick 商人
💥 me 說：「旋風腿！」造成 3 點傷害！
```

### NPC反擊（隨機台詞）
```
💥 商人 說：「你敢打我？」造成 2 點傷害！
💥 商人 說：「住手！」造成 2 點傷害！
💥 商人 說：「救命啊！」造成 2 點傷害！
💥 商人 說：「別欺負老實人」造成 3 點傷害！
💥 商人 說：「我也會功夫」造成 3 點傷害！
```

## 特點

### 1. 多樣性
- 可以為同一個技能設置多個台詞
- 每次使用隨機選擇一個
- 讓戰鬥對話更豐富

### 2. 個性化
- 每個角色可以有不同的台詞風格
- 玩家：勇敢、自信的台詞
- NPC：根據角色性格設置

### 3. 累積設置
```
sdl me punch 第一句
sdl me punch 第二句
sdl me punch 第三句
# 三句都會被保存，隨機選用
```

## 應用場景

### 不同情境的台詞
```
# 普通攻擊
sdl 張三 punch 接招！
sdl 張三 punch 看我的

# 配合條件使用（未來可擴展）
sdl 張三 punch add 你惹怒我了！ when 血量<30
sdl 張三 kick add 絕招！ when 戰鬥經驗>100
```

### 角色個性
```
# 暴躁角色
sdl 暴躁王 punch 去死！
sdl 暴躁王 punch 滾開！
sdl 暴躁王 kick 踢死你！

# 溫和角色
sdl 和平使者 punch 對不起
sdl 和平使者 punch 沒辦法
sdl 和平使者 kick 請原諒
```

### 幽默效果
```
sdl 小丑 punch 哈哈哈
sdl 小丑 punch 驚不驚喜
sdl 小丑 punch 意不意外
sdl 小丑 kick 踢你一腳啦
sdl 小丑 kick 嘿嘿
```

## 技術細節

### 隨機演算法
- 使用 `rand::thread_rng()`
- `gen_range(0..len)` 生成隨機索引
- 從台詞列表中取出對應台詞

### 台詞儲存
- 儲存在 `dialogues: HashMap<String, Vec<DialogueOption>>`
- 技能名稱作為 key（如 "punch", "kick"）
- 每次 `sdl` 會添加新台詞到列表

### 選擇邏輯
```rust
if !dialogue_options.is_empty() {
    let index = rng.gen_range(0..dialogue_options.len());
    return dialogue_options[index].text.clone();
}
```

## 測試範例

```bash
cargo run

# 設置多個台詞
> sdl me punch 吃我一拳
> sdl me punch 看拳！
> sdl me punch 哼！

# 測試隨機效果
> flyto 3,3
> punch 商人
💥 me 說：「看拳！」造成 2 點傷害！

> punch 商人  
💥 me 說：「吃我一拳」造成 2 點傷害！

> punch 商人
💥 me 說：「哼！」造成 2 點傷害！

# 每次都可能不同！
```

## 優勢

### 1. 生動活潑
- ✅ 戰鬥不再單調
- ✅ 每次攻擊都有新鮮感
- ✅ 增加遊戲趣味性

### 2. 易於擴展
- ✅ 隨時可以添加新台詞
- ✅ 不影響已有台詞
- ✅ 支援無限數量

### 3. 靈活運用
- ✅ 可以設置情境台詞
- ✅ 可以表現角色個性
- ✅ 可以製造幽默效果

## 相關檔案

- `src/person.rs` - `get_skill_dialogue()` 函數
- `SKILL_DIALOGUE_MERGE.md` - 台詞系統整合說明
