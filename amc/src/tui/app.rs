use super::components::context_view::ContextComponent;
use super::components::file_tree::FileTreeComponent;
use crossterm::event::KeyCode;
use ego_tree::Tree;
use std::collections::HashSet;
use std::path::Path;

pub enum PanelFocus {
    Tree,
    Context,
    Preview,
}

pub struct AppState {
    pub file_tree: FileTreeComponent,
    pub context: ContextComponent,
    pub file_preview: String, // File content preview
    pub focused: PanelFocus,
    // pub show_hidden: bool,
}

#[derive(Debug)]
pub enum PreviewResult {
    TextFile,
    BinaryFile,
    TooLarge(u64),
    Error(String),
}

impl AppState {
    pub fn new() -> Self {
        let file_tree = FileTreeComponent::new().unwrap_or_else(|_| {
            // Create a fallback empty component if tree building fails
            use super::components::file_tree::FileNode;
            use std::path::PathBuf;
            let tree = Tree::new(FileNode::new(
                "error",
                PathBuf::from("."),
                true,
                0,
                false,
            ));
            FileTreeComponent {
                tree,
                cursor: None,
                expanded_dir: None,
            }
        });

        let context = ContextComponent::new();
        let file_preview = "Select a file to preview...".to_string();

        Self {
            file_tree,
            context,
            file_preview,
            focused: PanelFocus::Tree,
            // show_hidden: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        // Handle global keys first
        match key {
            KeyCode::Char('q') => return false, // Signal to quit
            KeyCode::Tab => {
                self.cycle_focus();
                return true;
            }
            KeyCode::BackTab => {
                // Shift+Tab
                self.cycle_focus_reverse();
                return true;
            }
            _ => {}
        }

        // Store previous cursor position to detect changes
        let previous_cursor = self.file_tree.cursor;

        if self.focused == PanelFocus::Tree
            && self.file_tree.cursor != previous_cursor
        {
            self.update_file_preview();
        }

        // Delegate to focused component
        use crate::tui::components::Focusable;
        match self.focused {
            PanelFocus::Tree => self.file_tree.handle_key(key),
            PanelFocus::Context => self.context.handle_key(key),
            PanelFocus::Preview => false, // Log doesn't handle keys yet
        }
    }

    fn cycle_focus(&mut self) {
        self.focused = match self.focused {
            PanelFocus::Tree => PanelFocus::Context,
            PanelFocus::Context => PanelFocus::Preview,
            PanelFocus::Preview => PanelFocus::Tree,
        };
    }

    fn cycle_focus_reverse(&mut self) {
        self.focused = match self.focused {
            PanelFocus::Tree => PanelFocus::Preview,
            PanelFocus::Context => PanelFocus::Tree,
            PanelFocus::Preview => PanelFocus::Context,
        };
    }

    fn update_file_preview(&mut self) {
        if let Some(file_path) = self.file_tree.get_selected_file_path() {
            match self.should_preview_file(&file_path) {
                PreviewResult::TextFile => {
                    match self.load_file_chunk(&file_path, 1000) {
                        Ok(content) => self.file_preview = content,
                        Err(e) => {
                            self.file_preview = format!("Error reading file: {}", e)
                        }
                    }
                }
                PreviewResult::BinaryFile => {
                    self.file_preview =
                        "📦 Binary file - no preview available".to_string();
                }
                PreviewResult::TooLarge(size) => {
                    self.file_preview = format!(
                        "📄 File too large ({}) - no preview available",
                        human_size(size)
                    );
                }
                PreviewResult::Error(msg) => {
                    self.file_preview = format!("❌ Error: {}", msg);
                }
            }
        } else {
            self.file_preview = "Select a file to preview...".to_string();
        }
    }

    fn should_preview_file(&self, path: &Path) -> PreviewResult {
        // Check if file exists and get metadata
        let metadata = match std::fs::metadata(path) {
            Ok(meta) => meta,
            Err(e) => {
                return PreviewResult::Error(format!(
                    "Cannot read metadata: {}",
                    e
                ));
            }
        };

        // Skip directories
        if metadata.is_dir() {
            return PreviewResult::Error("Directory selected".to_string());
        }

        // Check file size (limit to 10MB)
        const MAX_SIZE: u64 = 10 * 1024 * 1024;
        if metadata.len() > MAX_SIZE {
            return PreviewResult::TooLarge(metadata.len());
        }

        // Check if file is likely text based on extension
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            let text_extensions = [
                "rs", "toml", "txt", "md", "json", "yaml", "yml", "xml", "html",
                "css", "js", "ts", "py", "go", "java", "c", "cpp", "h", "hpp",
                "sh", "bash", "zsh", "fish", "vim", "lua", "rb", "php", "sql",
                "log", "cfg", "conf", "ini",
            ];

            if text_extensions.contains(&extension.to_lowercase().as_str()) {
                return PreviewResult::TextFile;
            }
        }

        // For files without known text extensions, try to detect if it's text
        // by reading first few bytes
        match std::fs::read(path) {
            Ok(bytes) => {
                if bytes.len() > 1000 {
                    // Check first 1000 bytes for null bytes (binary indicator)
                    if bytes[..1000].contains(&0) {
                        PreviewResult::BinaryFile
                    } else {
                        PreviewResult::TextFile
                    }
                } else {
                    // Small file, check all bytes
                    if bytes.contains(&0) {
                        PreviewResult::BinaryFile
                    } else {
                        PreviewResult::TextFile
                    }
                }
            }
            Err(e) => PreviewResult::Error(format!("Cannot read file: {}", e)),
        }
    }

    fn load_file_chunk(
        &self,
        path: &Path,
        max_lines: usize,
    ) -> anyhow::Result<String> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut lines = Vec::new();
        let mut line_count = 0;

        for line in reader.lines() {
            if line_count >= max_lines {
                lines
                    .push(format!("\n... (showing first {} lines) ...", max_lines));
                break;
            }

            lines.push(line?);
            line_count += 1;
        }

        Ok(lines.join("\n"))
    }
}

impl PartialEq for PanelFocus {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (PanelFocus::Tree, PanelFocus::Tree)
                | (PanelFocus::Context, PanelFocus::Context)
                | (PanelFocus::Preview, PanelFocus::Preview)
        )
    }
}

// Keep the utility function
pub fn human_size(size: u64) -> String {
    if size > 1024 * 1024 {
        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
    } else if size > 1024 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else {
        format!("{size} B")
    }
}
