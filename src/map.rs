use rand::Rng;
use crate::observable::Observable;
use crate::item::Item;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// 地圖類型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MapType {
    Normal,     // 普通地圖
    Forest,     // 森林地圖
    Cave,       // 洞穴地圖
    Desert,     // 沙漠地圖
    Mountain,   // 山脈地圖
}

impl MapType {
    pub fn walkable_chance(&self) -> f64 {
        match self {
            MapType::Normal => 0.7,
            MapType::Forest => 0.6,
            MapType::Cave => 0.5,
            MapType::Desert => 0.75,
            MapType::Mountain => 0.4,
        }
    }
}

// 描述資料庫
pub struct DescriptionDb {
    descriptions: HashMap<MapType, Vec<String>>,
}

impl DescriptionDb {
    pub fn new() -> Self {
        let mut db = DescriptionDb {
            descriptions: HashMap::new(),
        };
        db.init_default_descriptions();
        db
    }

    fn init_default_descriptions(&mut self) {
        // 預設描述 - 普通地圖
        self.descriptions.insert(
            MapType::Normal,
            vec![
                "寬闊的綠色草地，微風吹過", "鵝卵石舖成的古老石路", "鬱鬱蒼蒼的樹林",
                "遠方的翠綠山丘", "清澈的河流流經此地", "灰色的嶙峋岩石", "密集的灌木叢",
                "盛開的彩色花田", "黃色沙漠風景", "白雪皚皚的地面", "懸崖的邊緣，可以俯瞰遠景",
                "神秘的洞穴入口", "古老的廢墟遺跡", "熱鬧的小村落", "宏偉的石砌城堡",
                "連接兩岸的古老橋樑", "清涼的泉水湧出", "寂靜的墓地", "聳立的懸崖", "深綠色的古老森林",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );
        
        // 森林地圖
        self.descriptions.insert(
            MapType::Forest,
            vec![
                "密集的樹林，陽光透過樹葉灑下", "高大的橡樹，樹幹粗壯", "竹林沙沙作響",
                "布滿樹根和苔蘚的地面", "灌木叢掩蓋的狹窄小徑", "林間寬敞的空地",
                "千年古木，蒼勁挺立", "陰暗潮濕的林地", "清澈溪流邊的石頭",
                "蕨類植物生長茂盛", "松樹林香氣撲鼻", "橡樹叢聚集成林",
                "野生花卉遍佈", "林間寬闊露地", "深不見底的老林", "倒下的樹樁",
                "稀有的臺灣檜木", "山毛櫸樹下", "森林中的大石頭", "腐爛的枯木",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        // 洞穴地圖
        self.descriptions.insert(
            MapType::Cave,
            vec![
                "灰色的石灰岩洞壁", "垂直的鐘乳石發光", "暗黑的地下河流",
                "令人窒息的深淵", "礦脈閃閃發光", "古老的化石印痕", "高聳的石柱",
                "黑暗陰暗的角落", "地下水池平靜如鏡", "岩石峽谷狹窄蜿蜒",
                "洞穴入口外透進微光", "如同地下宮殿般的空間", "岩壁上的裂縫",
                "陡峭的滑坡", "蝙蝠棲息地傳來尖叫聲", "熔岩冷卻成的黑石",
                "地震遺留的痕跡", "隱藏的秘密房間", "寶藏可能埋藏的地點", "複雜的地下迷宮",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        // 沙漠地圖
        self.descriptions.insert(
            MapType::Desert,
            vec![
                "細細的沙粒覆蓋", "金黃色的沙丘堆積", "岩石露頭刺穿沙面",
                "乾枯的灌木稀疏分佈", "遠方的海市蜃樓", "黃色沙塵暴逼近", "古老的廢墟遺跡",
                "綠洲中的棕櫚樹", "駱駝骨骼白骨化", "黑色的火山岩", "沙漠刺骨寒風",
                "地表的流沙危險", "沙漠中的沙棗樹", "鹽湖結晶而成", "陶土色的沙堆",
                "被沙埋沒的建築", "開放的沙漠花卉", "岩石構成的平台", "深色的沙層",
                "被風化的古老石頭",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        // 山脈地圖
        self.descriptions.insert(
            MapType::Mountain,
            vec![
                "白雪覆蓋的山峰", "陡峭的岩壁險峻", "冰川流動的痕跡",
                "高山草甸風景秀麗", "亂石堆積的坡地", "深邃的峽谷", "壯觀的瀑布",
                "懸崖邊沒有防欄", "山洞入口黑暗深邃", "樹林和高山交界處",
                "山脈的脊線清晰", "碎石坡滑落危險", "高山湖水清澈冰冷",
                "雲霧繚繞視線不清", "松樹林密集生長", "石頭砌成的平台",
                "冰凍的溪流結冰", "山頂視野遼闊", "吊橋晃動不穩", "彎曲的山路",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );
    }

    pub fn get_description(&self, map_type: &MapType) -> Option<String> {
        if let Some(descs) = self.descriptions.get(map_type) {
            if !descs.is_empty() {
                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(0..descs.len());
                return Some(descs[idx].clone());
            }
        }
        None
    }

    pub fn load_from_file(&mut self, _map_name: &str, _map_type: &MapType) -> std::io::Result<()> {
        // 預留檔案載入邏輯
        Ok(())
    }
}

// Point 代表地圖上的一個點
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: usize,
    pub y: usize,
    pub walkable: bool,           // 是否可移動
    pub description: String,      // 描述
    pub objects: Vec<String>,     // 該點上的物件（Person也算）
}

impl Point {
    pub fn new(x: usize, y: usize, walkable: bool, description: String) -> Self {
        Point {
            x,
            y,
            walkable,
            description,
            objects: Vec::new(),
        }
    }

    // 隨機生成Point - 使用指定的地圖類型
    pub fn random_for_type(x: usize, y: usize, map_type: &MapType) -> Self {
        let mut rng = rand::thread_rng();
        
        let db = DescriptionDb::new();
        let walkable_chance = map_type.walkable_chance();
        
        let walkable = rng.gen_bool(walkable_chance);
        let description = db.get_description(map_type)
            .unwrap_or_else(|| "未知地點".to_string());
        
        Point {
            x,
            y,
            walkable,
            description,
            objects: Vec::new(),
        }
    }

    // 隨機生成Point - 舊方法保留相容性
    pub fn random(x: usize, y: usize) -> Self {
        Self::random_for_type(x, y, &MapType::Normal)
    }

    pub fn add_object(&mut self, obj: String) {
        self.objects.push(obj);
    }

    pub fn remove_object(&mut self, obj: &str) -> bool {
        if let Some(pos) = self.objects.iter().position(|x| x == obj) {
            self.objects.remove(pos);
            true
        } else {
            false
        }
    }
}

// Map 代表整個遊戲地圖
#[derive(Clone, Serialize, Deserialize)]
pub struct Map {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub map_type: MapType,           // 地圖類型
    pub points: Vec<Vec<Point>>,
}

impl Map {
    pub fn new(name: String, width: usize, height: usize) -> Self {
        Self::new_with_type(name, width, height, MapType::Normal)
    }

    // 根據類型建立地圖
    pub fn new_with_type(name: String, width: usize, height: usize, map_type: MapType) -> Self {
        let mut points = Vec::new();
        
        for y in 0..height {
            let mut row = Vec::new();
            for x in 0..width {
                row.push(Point::random_for_type(x, y, &map_type));
            }
            points.push(row);
        }

        Map {
            name,
            width,
            height,
            map_type,
            points,
        }
    }

    // 獲取所有可移動的點
    pub fn get_walkable_points(&self) -> Vec<(usize, usize)> {
        let mut walkable_points = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(point) = self.get_point(x, y) {
                    if point.walkable {
                        walkable_points.push((x, y));
                    }
                }
            }
        }
        walkable_points
    }

    // 獲取指定位置的Point
    pub fn get_point(&self, x: usize, y: usize) -> Option<&Point> {
        if x < self.width && y < self.height {
            Some(&self.points[y][x])
        } else {
            None
        }
    }

    // 可變地獲取指定位置的Point
    pub fn get_point_mut(&mut self, x: usize, y: usize) -> Option<&mut Point> {
        if x < self.width && y < self.height {
            Some(&mut self.points[y][x])
        } else {
            None
        }
    }

    // 獲取周圍的Point（3x3範圍，包括中心點）
    pub fn get_surrounding_points(&self, x: usize, y: usize, radius: usize) -> Vec<&Point> {
        let mut surrounding = Vec::new();
        
        let x_start = x.saturating_sub(radius);
        let x_end = (x + radius).min(self.width - 1);
        let y_start = y.saturating_sub(radius);
        let y_end = (y + radius).min(self.height - 1);
        
        for py in y_start..=y_end {
            for px in x_start..=x_end {
                if let Some(point) = self.get_point(px, py) {
                    surrounding.push(point);
                }
            }
        }
        
        surrounding
    }

    // 統計可移動和不可移動的Point
    pub fn get_stats(&self) -> (usize, usize) {
        let mut walkable = 0;
        let mut unwalkable = 0;

        for row in &self.points {
            for point in row {
                if point.walkable {
                    walkable += 1;
                } else {
                    unwalkable += 1;
                }
            }
        }

        (walkable, unwalkable)
    }

    // 初始化隨機 item 散落在地圖上，大概占一半的可移動地點
    pub fn initialize_items(&mut self) {
        let mut rng = rand::thread_rng();
        let walkable_points = self.get_walkable_points();
        
        if walkable_points.is_empty() {
            return;
        }

        // 計算要放置的 item 數量（可移動地點的一半）
        let item_count = walkable_points.len() / 2;
        
        // 隨機選擇位置並放置 item
        for _ in 0..item_count {
            let random_idx = rng.gen_range(0..walkable_points.len());
            let (x, y) = walkable_points[random_idx];
            
            if let Some(point) = self.get_point_mut(x, y) {
                let item = Item::generate_random();
                point.add_object(format!("[物品] {}", item.display()));
            }
        }
    }

    // 保存地圖到檔案（JSON格式）
    pub fn save(&self, filename: &str) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mut file = File::create(filename)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    // 從檔案加載地圖
    pub fn load(filename: &str) -> std::io::Result<Self> {
        use std::fs;

        let content = fs::read_to_string(filename)?;
        let map: Map = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(map)
    }
}

impl Observable for Map {
    fn show_title(&self) -> String {
        format!("地圖: {}", self.name)
    }

    fn show_description(&self) -> String {
        let (walkable, unwalkable) = self.get_stats();
        format!(
            "大小: {} x {}\n可行走點: {}\n不可行走點: {}",
            self.width, self.height, walkable, unwalkable
        )
    }

    fn show_list(&self) -> Vec<String> {
        vec![
            format!("總點數: {}", self.width * self.height),
            format!("類型: MUD地圖"),
        ]
    }
}
