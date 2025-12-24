use ratamud::map::{Map, MapType};
use std::fs;
use std::path::Path;

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
        let filename = format!("{maps_dir}/{name}.json");
        
        println!("Generating map: {name} ({width} x {height})");
        let map = Map::new_with_type(name.to_string(), width, height, map_type);
        
        match map.save(&filename) {
            Ok(_) => {
                let (walkable, unwalkable) = (
                    map.points.iter().flatten().filter(|p| p.walkable).count(),
                    map.points.iter().flatten().filter(|p| !p.walkable).count(),
                );
                println!("✓ Saved {filename} ({walkable} walkable, {unwalkable} unwalkable)");
            }
            Err(e) => println!("✗ Failed to save {filename}: {e}"),
        }
    }

    println!("\n✓ All maps generated successfully!");
}
