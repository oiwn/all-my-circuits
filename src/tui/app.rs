use super::components::file_tree::FileNode;
use std::collections::VecDeque;

pub enum PanelFocus {
    Tree,
    Context,
    Log,
}

pub struct AppState {
    pub file_tree: Vec<FileNode>,
    pub context: Vec<FileNode>,
    pub log: VecDeque<String>,
    pub panel: PanelFocus,
    pub show_hidden: bool,
}

impl AppState {
    pub fn new() -> Self {
        let file_tree = vec![
            FileNode::new("main.rs", "src/main.rs", false, 1234, true),
            FileNode::new("lib.rs", "src/lib.rs", false, 4321, false),
            FileNode::new("mod.rs", "src/tui/mod.rs", false, 1200, true),
        ];
        let context = file_tree.iter().cloned().filter(|f| f.selected).collect();
        let log = VecDeque::from(vec!["App started".to_string()]);
        Self {
            file_tree,
            context,
            log,
            panel: PanelFocus::Tree,
            show_hidden: false,
        }
    }
}

// Util for formatting sizes, used by context view
pub fn human_size(size: u64) -> String {
    if size > 1024 * 1024 {
        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
    } else if size > 1024 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else {
        format!("{size} B")
    }
}
