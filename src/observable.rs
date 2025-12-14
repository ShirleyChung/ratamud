use ratatui::text::Line;

// Observable trait 定义侧边面板的显示接口
pub trait Observable {
    // 获取标题
    fn show_title(&self) -> String;
    
    // 获取描述
    fn show_description(&self) -> String;
    
    // 获取列表项
    fn show_list(&self) -> Vec<String>;
}

// 空对象实现，用于默认显示
pub struct Empty;

impl Observable for Empty {
    fn show_title(&self) -> String {
        "無資料".to_string()
    }
    
    fn show_description(&self) -> String {
        String::new()
    }
    
    fn show_list(&self) -> Vec<String> {
        Vec::new()
    }
}

// 世界信息可观察对象
pub struct WorldInfo {
    pub name: String,
    pub description: String,
    pub maps: Vec<String>,
}

impl WorldInfo {
    pub fn new(name: String, description: String, maps: Vec<String>) -> Self {
        WorldInfo {
            name,
            description,
            maps,
        }
    }
}

impl Observable for WorldInfo {
    fn show_title(&self) -> String {
        format!("【{}】", self.name)
    }
    
    fn show_description(&self) -> String {
        self.description.clone()
    }
    
    fn show_list(&self) -> Vec<String> {
        if self.maps.is_empty() {
            vec!["無地圖".to_string()]
        } else {
            self.maps.clone()
        }
    }
}

// 通用观察对象，用于显示自定义内容
pub struct CustomObservable {
    pub title: String,
    pub description: String,
    pub items: Vec<String>,
}

impl CustomObservable {
    #[allow(dead_code)]
    pub fn new(title: String, description: String, items: Vec<String>) -> Self {
        CustomObservable {
            title,
            description,
            items,
        }
    }
}

impl Observable for CustomObservable {
    fn show_title(&self) -> String {
        self.title.clone()
    }
    
    fn show_description(&self) -> String {
        self.description.clone()
    }
    
    fn show_list(&self) -> Vec<String> {
        self.items.clone()
    }
}

// 将 Observable 对象转换为可显示的行
pub fn observable_to_lines(obs: &dyn Observable) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    
    // 添加标题
    lines.push(Line::from(obs.show_title()));
    
    // 如果有描述，添加描述
    let description = obs.show_description();
    if !description.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(description));
    }
    
    // 如果有列表项，添加列表
    let list = obs.show_list();
    if !list.is_empty() {
        lines.push(Line::from(""));
        for item in list {
            lines.push(Line::from(format!("• {item}")));
        }
    }
    
    lines
}
