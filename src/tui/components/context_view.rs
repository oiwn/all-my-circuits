use super::file_tree::FileNode;
use crate::tui::app::human_size;
use ratatui::widgets::{List, ListItem};

pub fn context_panel(files: &[FileNode]) -> List {
    let items: Vec<_> = files
        .iter()
        .map(|f| ListItem::new(format!("{} ({})", f.name, human_size(f.size))))
        .collect();
    List::new(items)
}
