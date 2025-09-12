// In src/tui/components/file_tree.rs
use crate::tui::components::Focusable;
use crossterm::event::KeyCode;
use ego_tree::{NodeId, Tree};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
};
use std::{collections::HashSet, path::PathBuf};

pub struct FileTreeComponent {
    pub tree: Tree<FileNode>,
    pub cursor: Option<NodeId>,
    pub expanded_dir: Option<NodeId>,
}

impl FileTreeComponent {
    pub fn new() -> anyhow::Result<Self> {
        let tree = FileNode::build_tree()?;
        let cursor = Some(tree.root().id());

        Ok(Self {
            tree,
            cursor,
            expanded_dir: None,
        })
    }

    /// Calculate which nodes should be visible based on current cursor position
    /// Returns: Vec<(NodeId, FileNode, display_prefix, depth)>
    pub fn calculate_display_state(
        &self,
    ) -> Vec<(NodeId, FileNode, String, usize)> {
        let mut result = Vec::new();

        if let Some(cursor) = self.cursor {
            let path_to_cursor = self.get_path_to_node(cursor);
            self.traverse_for_display(
                self.tree.root().id(),
                &path_to_cursor,
                0,
                String::new(),
                true,
                &mut result,
            );
        }

        result
    }

    fn traverse_for_display(
        &self,
        node_id: NodeId,
        path_to_cursor: &HashSet<NodeId>,
        depth: usize,
        prefix: String,
        is_last: bool,
        result: &mut Vec<(NodeId, FileNode, String, usize)>,
    ) {
        let node = self.tree.get(node_id).unwrap();
        let node_data = node.value();

        // Create display prefix
        let display_prefix = if depth == 0 {
            String::new()
        } else {
            format!("{}{} ", prefix, if is_last { "└──" } else { "├──" })
        };

        // Clone node data for result
        let mut display_node = node_data.clone();
        display_node.depth = depth;

        result.push((node_id, display_node, display_prefix, depth));

        // Show children if this node is on the path to cursor (expanded)
        let should_show_children = node_data.is_dir && node_data.expanded;

        if should_show_children {
            let children: Vec<_> =
                node.children().map(|child| child.id()).collect();

            for (i, child_id) in children.iter().enumerate() {
                let is_last_child = i == children.len() - 1;
                let new_prefix = if depth == 0 {
                    String::new()
                } else {
                    format!("{}{}", prefix, if is_last { "    " } else { "│   " })
                };

                self.traverse_for_display(
                    *child_id,
                    path_to_cursor,
                    depth + 1,
                    new_prefix,
                    is_last_child,
                    result,
                );
            }
        }
    }

    fn get_path_to_node(&self, target: NodeId) -> HashSet<NodeId> {
        let mut path = HashSet::new();
        let mut current = Some(target);

        while let Some(node_id) = current {
            path.insert(node_id);
            current = self.tree.get(node_id).unwrap().parent().map(|p| p.id());
        }
        path
    }

    fn get_visible_nodes(&self) -> Vec<NodeId> {
        self.calculate_display_state()
            .into_iter()
            .map(|(node_id, _, _, _)| node_id)
            .collect()
    }

    pub fn navigate_up(&mut self) {
        let visible_nodes = self.get_visible_nodes();
        if let Some(current_cursor) = self.cursor {
            if let Some(current_pos) =
                visible_nodes.iter().position(|&id| id == current_cursor)
            {
                let new_pos = if current_pos == 0 {
                    visible_nodes.len() - 1 // Wrap to bottom
                } else {
                    current_pos - 1
                };
                self.cursor = visible_nodes.get(new_pos).copied();
            }
        }
    }

    pub fn navigate_down(&mut self) {
        let visible_nodes = self.get_visible_nodes();
        if let Some(current_cursor) = self.cursor {
            if let Some(current_pos) =
                visible_nodes.iter().position(|&id| id == current_cursor)
            {
                let new_pos = if current_pos >= visible_nodes.len() - 1 {
                    0 // Wrap to top
                } else {
                    current_pos + 1
                };
                self.cursor = visible_nodes.get(new_pos).copied();
            }
        }
    }
    pub fn handle_right_arrow(&mut self) -> bool {
        if let Some(cursor) = self.cursor {
            let node = self.tree.get(cursor).unwrap();
            if node.value().is_dir {
                if !node.value().expanded {
                    // Directory is collapsed, expand it
                    self.expand_node(cursor);
                    true
                } else {
                    // Directory is expanded, move to first child
                    if let Some(first_child) = node.children().next() {
                        self.cursor = Some(first_child.id());
                        true
                    } else {
                        false
                    }
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn handle_left_arrow(&mut self) -> bool {
        if let Some(cursor) = self.cursor {
            let node = self.tree.get(cursor).unwrap();
            if node.value().is_dir && node.value().expanded {
                // Directory is expanded, collapse it
                self.collapse_node(cursor);
                true
            } else {
                // Move to parent directory
                if let Some(parent) = node.parent() {
                    self.cursor = Some(parent.id());
                    true
                } else {
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn get_selected_file_path(&self) -> Option<PathBuf> {
        self.cursor.and_then(|cursor| {
            let node = self.tree.get(cursor)?;
            Some(node.value().path.clone())
        })
    }

    fn toggle_expand(&mut self, node_id: NodeId) -> bool {
        if let Some(node) = self.tree.get(node_id) {
            if node.value().is_dir {
                return true;
            }
        }
        false
    }

    fn expand_node(&mut self, node_id: NodeId) -> bool {
        if let Some(node) = self.tree.get(node_id) {
            if node.value().is_dir {
                self.expanded_dir = Some(node_id);
                return true;
            }
        }
        false
    }

    fn collapse_node(&mut self, node_id: NodeId) -> bool {
        if let Some(node) = self.tree.get(node_id) {
            if node.value().is_dir {
                return true;
            }
        }
        false
    }
}

impl Focusable for FileTreeComponent {
    fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Up => {
                self.navigate_up();
                true
            }
            KeyCode::Down => {
                self.navigate_down();
                true
            }
            KeyCode::Right => self.handle_right_arrow(),
            KeyCode::Left => self.handle_left_arrow(),
            _ => false,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let display_items = self.calculate_display_state();

        let tree_items: Vec<_> = display_items
            .iter()
            .map(|(node_id, file_node, prefix, _depth)| {
                let selected_marker = if Some(*node_id) == self.cursor && focused {
                    "► "
                } else {
                    "  "
                };

                let icon = if file_node.is_dir {
                    if file_node.expanded {
                        "📁▼"
                    } else {
                        "📁▶"
                    }
                } else {
                    "📄"
                };

                let size_str = if file_node.is_dir {
                    String::new()
                } else {
                    format!(" ({})", crate::tui::app::human_size(file_node.size))
                };

                ListItem::new(format!(
                    "{}{}{}{}{}{}",
                    selected_marker,
                    prefix,
                    icon,
                    file_node.name,
                    size_str,
                    if file_node.selected { " ✓" } else { "" }
                ))
            })
            .collect();

        let border_style = if focused {
            ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
        } else {
            ratatui::style::Style::default()
        };

        let tree = List::new(tree_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("File Tree")
                .border_style(border_style),
        );
        frame.render_widget(tree, area);
    }
}

#[derive(Clone, Debug)]
pub struct FileNode {
    // pub id: usize,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub selected: bool,
    pub expanded: bool,
    pub depth: usize,
}

impl FileNode {
    pub fn new(
        // id: usize,
        name: &str,
        path: PathBuf,
        is_dir: bool,
        size: u64,
        selected: bool,
    ) -> Self {
        Self {
            // id,
            name: name.to_string(),
            path,
            is_dir,
            size,
            selected,
            expanded: true, // start collapsed
            depth: 0,
        }
    }

    // Build tree from current directory (recursively)
    pub fn build_tree() -> anyhow::Result<Tree<FileNode>> {
        let current_dir = std::env::current_dir()?;
        let mut tree = Tree::new(FileNode::new(
            current_dir.file_name().unwrap().to_str().unwrap(),
            current_dir.clone(),
            true,
            0,
            false,
        ));

        let root_id = tree.root().id();
        Self::build_directory_recursive(&mut tree, root_id, &current_dir, 1)?;

        Ok(tree)
    }

    fn build_directory_recursive(
        tree: &mut Tree<FileNode>,
        parent_id: NodeId,
        dir_path: &PathBuf,
        mut next_id: usize,
    ) -> anyhow::Result<usize> {
        let entries = std::fs::read_dir(dir_path)?;
        let mut file_entries: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                !entry.file_name().to_str().unwrap_or("").starts_with('.')
            })
            .collect();

        // Sort: directories first, then files
        file_entries.sort_by(|a, b| {
            let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for entry in file_entries {
            let path = entry.path();
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let size = if is_dir {
                0
            } else {
                entry.metadata().map(|m| m.len()).unwrap_or(0)
            };

            let node = FileNode::new(
                entry.file_name().to_str().unwrap(),
                path.clone(),
                is_dir,
                size,
                false,
            );

            let mut parent = tree.get_mut(parent_id).unwrap();
            let child_id = parent.append(node).id();
            next_id += 1;

            // Recursively build subdirectories
            if is_dir {
                next_id = Self::build_directory_recursive(
                    tree, child_id, &path, next_id,
                )?;
            }
        }

        Ok(next_id)
    }
}

// Helper function to flatten tree for display
pub fn flatten_tree_for_display(
    tree: &Tree<FileNode>,
) -> Vec<(NodeId, FileNode, String)> {
    let mut result = Vec::new();

    fn traverse(
        tree: &Tree<FileNode>,
        node_id: NodeId,
        prefix: String,
        is_last: bool,
        depth: usize,
        result: &mut Vec<(NodeId, FileNode, String)>,
    ) {
        let node = tree.get(node_id).unwrap();
        let node_data = node.value();

        let display_prefix = if depth == 0 {
            String::new()
        } else {
            format!("{}{} ", prefix, if is_last { "└──" } else { "├──" })
        };

        let mut display_node = node_data.clone();
        display_node.depth = depth;

        result.push((node_id, display_node, display_prefix));

        if node_data.is_dir && node_data.expanded {
            let children: Vec<_> =
                node.children().map(|child| child.id()).collect();
            // let children: Vec<_> = node.children().collect();
            for (i, child_id) in children.iter().enumerate() {
                let is_last_child = i == children.len() - 1;
                let new_prefix = if depth == 0 {
                    String::new()
                } else {
                    format!("{}{}", prefix, if is_last { "    " } else { "│   " })
                };

                traverse(
                    tree,
                    *child_id,
                    new_prefix,
                    is_last_child,
                    depth + 1,
                    result,
                );
            }
        }
    }

    traverse(tree, tree.root().id(), String::new(), true, 0, &mut result);
    result
}
