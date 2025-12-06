use rand::Rng;
use crate::observable::Observable;
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
        // 預設描述
        self.descriptions.insert(
            MapType::Normal,
            vec![
                "綠色草地", "石子路", "樹林", "山丘", "河流", "岩石", "灌木叢", "花田", "沙漠", "雪地",
                "崖邊", "洞穴入口", "廢墟", "村落", "城堡", "橋樑", "泉水", "墓地", "懸崖", "森林",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );
        
        self.descriptions.insert(
            MapType::Forest,
            vec![
                "密集樹林", "高大橡樹", "竹林", "樹根和苔蘚", "灌木掩蓋的小徑", "林間空地", "古老樹木", 
                "陰暗林地", "溪流邊", "蕨類植物", "松樹林", "橡樹叢", "野生花卉", "林間露地", "深林",
                "樹樁", "臺灣檜木", "山毛櫸", "森林石頭", "枯木",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        self.descriptions.insert(
            MapType::Cave,
            vec![
                "石灰岩洞", "鐘乳石", "地下河", "深淵", "礦脈", "化石", "石柱", "暗角", "水池", "岩石峽谷",
                "洞穴入口", "地下宮殿", "裂縫", "滑坡", "蝙蝠棲息地", "熔岩冷卻區", "地震遺跡", "隱藏房間", "寶藏點", "迷宮",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        self.descriptions.insert(
            MapType::Desert,
            vec![
                "細沙", "金黃沙丘", "岩石露頭", "乾枯灌木", "蜃景", "沙塵暴", "古廢墟", "綠洲", "駱駝骨", "黑岩",
                "沙漠風", "流沙", "沙棗樹", "鹽湖", "陶土堆", "被埋建築", "沙漠花卉", "岩石平台", "暗色沙", "風化石",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        self.descriptions.insert(
            MapType::Mountain,
            vec![
                "雪峰", "陡峭岩壁", "冰川", "高山草甸", "石堆", "峽谷", "瀑布", "懸崖邊", "山洞", "樹林邊界",
                "山脈脊線", "碎石坡", "高山湖", "雲霧繚繞", "松樹林", "岩石平台", "冰凍溪流", "山頂", "懸橋", "山路",
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

    pub fn load_from_file(&mut self, _map_name: &str, map_type: &MapType) -> std::io::Result<()> {
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
