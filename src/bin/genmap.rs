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
                "綠色的草地，迎風搖晃",
                "鵝卵石鋪成的道路，踩上去很舒服",
                "一片小樹林，透著青翠的光",
                "緩緩升起的草製山丘，可以看到遠方",
                "清澈的河流，能聽到水流聲",
                "灰色的巨大岩石，布滿青苔",
                "密集的灌木叢，某處傳來鳥鳴聲",
                "開滿野花的草田，蝴蝶翩翩起舞",
                "黃褐色的沙漠邊緣，風吹起沙粒",
                "白雪皚皚的地面，寒風刺骨",
                "懸崖邊緣，俯瞰廣闊的平原",
                "洞穴的黑色入口，散發著冷空氣",
                "破敗的廢墟，曾經輝煌的遺跡",
                "遠處可見村落的煙火",
                "宏偉的城堡矗立在遠方",
                "跨越河流的古老橋樑",
                "清澈的泉水源頭，水質甘甜",
                "墓地靜謐而莊嚴，立著無數墓碑",
                "陡峭的懸崖，風在耳邊呼嘯",
                "茂密的森林，陽光難以透射",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );
        
        self.descriptions.insert(
            MapType::Forest,
            vec![
                "密集的樹林，陽光透過枝葉灑落",
                "高大的橡樹聳立，樹齡已有百年",
                "靜謐的竹林，竹葉沙沙作響",
                "樹根盤根錯節，苔蘚覆蓋地面",
                "灌木掩蓋的小徑，光線昏暗幽深",
                "林間的空地，陽光灑滿草地",
                "古老的樹木，樹皮深紋斑駁",
                "陰暗的林地，空氣潮濕森冷",
                "溪流邊的林地，水聲潺潺",
                "蕨類植物蔓生，綠意盎然",
                "松樹林立，散發松香味道",
                "橡樹叢生，秋日遍地金黃",
                "野生花卉盛開，蜜蜂嗡嗡飛舞",
                "林間的露地，鹿群在此棲息",
                "深林深處，神秘而古老",
                "樹樁遍佈，舊林正在更新",
                "臺灣檜木群，香氣撲鼻而來",
                "山毛櫸林，季節變化展現四季美景",
                "森林中的石頭，被苔蘚完全覆蓋",
                "枯木橫臥，正在被大地吸收",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        self.descriptions.insert(
            MapType::Cave,
            vec![
                "石灰岩洞穴，洞壁質地堅硬",
                "垂直的鐘乳石，形狀奇特怪異",
                "地下河流在洞穴中緩緩流淌",
                "深不見底的深淵，散發著冷氣",
                "露出的礦脈，閃閃發光",
                "化石被嵌入岩壁，記錄上古",
                "巨大的石柱從頂端延伸到地面",
                "昏暗的角落，可能隱藏著危險",
                "地下水池，水面黑如墨",
                "崎嶇的岩石峽谷，行走困難",
                "洞穴入口，透著微弱的外界光線",
                "地下宮殿般的空間，壯觀非凡",
                "岩石裂縫，深不可測",
                "陡峭的滑坡，容易失足",
                "蝙蝠棲息地，黑影飛舞",
                "熔岩冷卻形成的怪異地形",
                "古老地震留下的痕跡",
                "隱藏在深處的神秘房間",
                "傳說中寶藏埋藏之處",
                "複雜的迷宮般洞穴系統",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        self.descriptions.insert(
            MapType::Desert,
            vec![
                "細膩的沙粒，踏上去陷入腳踝",
                "金黃色的沙丘起伏如波浪",
                "露出的岩石頭，頂著烈日",
                "乾枯的灌木叢，早已失去水分",
                "天際線上出現蜃景，虛幻迷人",
                "沙塵暴迫近，天色漸暗",
                "古老的廢墟，埋沒於黃沙",
                "沙漠中的綠洲，生命的奇跡",
                "駱駝的骨骼，述說著滄桑",
                "黑色的熔岩岩石，烈日炙烤",
                "沙漠風呼嘯而過，卷起黃沙",
                "流沙區域，危險重重",
                "沙棗樹屹立，頑強地生存",
                "鹽湖晶瑩，鹽結晶覆蓋地面",
                "陶土堆積，古文明的遺留",
                "被沙埋沒的古代建築",
                "沙漠花卉盛開，顏色鮮豔",
                "高聳的岩石平台，俯瞰沙漠",
                "暗色沙土，深暗難見",
                "風化的石頭，被歲月雕刻",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        self.descriptions.insert(
            MapType::Mountain,
            vec![
                "白雪皚皚的山峰，直刺蒼穹",
                "陡峭的岩壁，幾乎垂直於地面",
                "廣闊的冰川，蒼藍色冰層",
                "高山草甸，遍地山花競放",
                "一堆普通的小石堆，堆積而成",
                "深不見底的峽谷，風聲颯颯",
                "瀑布飛流直下，水花四濺",
                "懸崖邊緣，往下是萬丈深淵",
                "山洞深藏其中，黑暗神秘",
                "樹林與山地的分界線，景色優美",
                "在山的脊線上，登高望去可以看到更遠的山脈",
                "碎石坡陡峭滑落，危險之地",
                "高山湖水色湛藍，清澈見底",
                "雲霧繚繞於山峰，如夢如幻",
                "松樹林立於山腰，迎風搖曳",
                "岩石平台堅實，適合駐足眺望",
                "冰凍的溪流，凝結成琉璃",
                "陡峭山頂，一覽眾山小",
                "懸橋搖晃，連接兩側懸崖",
                "蜿蜒的山路，引領上升之途",
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
