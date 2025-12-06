use rand::Rng;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
}

// Point 代表地圖上的一個點
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: usize,
    pub y: usize,
    pub walkable: bool,
    pub description: String,
    pub objects: Vec<String>,
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
}

// Map 代表整個遊戲地圖
#[derive(Clone, Serialize, Deserialize)]
pub struct Map {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub map_type: MapType,
    pub points: Vec<Vec<Point>>,
}

impl Map {
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

    pub fn save(&self, filename: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mut file = std::fs::File::create(filename)?;
        use std::io::Write;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

fn main() {
    let maps_dir = "maps";
    
    // 建立 maps 目錄（如果不存在）
    if !Path::new(maps_dir).exists() {
        fs::create_dir_all(maps_dir).expect("Failed to create maps directory");
        println!("✓ Created maps directory");
    }

    // 定義要生成的地圖
    let maps_to_generate = vec![
        ("森林", MapType::Forest, 100, 100),
        ("洞穴", MapType::Cave, 80, 80),
        ("沙漠", MapType::Desert, 120, 120),
        ("山脈", MapType::Mountain, 100, 100),
    ];

    for (name, map_type, width, height) in maps_to_generate {
        let filename = format!("{}/{}.json", maps_dir, name);
        
        println!("Generating map: {} ({} x {})", name, width, height);
        let map = Map::new_with_type(name.to_string(), width, height, map_type);
        
        match map.save(&filename) {
            Ok(_) => {
                let (walkable, unwalkable) = (
                    map.points.iter().flatten().filter(|p| p.walkable).count(),
                    map.points.iter().flatten().filter(|p| !p.walkable).count(),
                );
                println!("✓ Saved {} ({} walkable, {} unwalkable)", filename, walkable, unwalkable);
            }
            Err(e) => println!("✗ Failed to save {}: {}", filename, e),
        }
    }

    println!("\n✓ All maps generated successfully!");
}
