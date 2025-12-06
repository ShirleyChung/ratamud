use std::fs;

fn main() {
    let world_dir = "worlds/初始世界";
    match fs::create_dir_all(&world_dir) {
        Ok(_) => println!("Created: {}", world_dir),
        Err(e) => println!("Error: {}", e),
    }
    
    // Check if exists
    if std::path::Path::new(&world_dir).exists() {
        println!("Directory exists!");
        // List contents
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("  {}", entry.path().display());
                }
            }
        }
    }
}
