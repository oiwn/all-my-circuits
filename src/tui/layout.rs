use crate::tui::app::AppState;
use crate::tui::components::{context_view, file_tree};
use ratatui::{
    Frame,
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn draw(f: &mut Frame, app: &AppState) {
    use ratatui::layout::{Constraint, Direction, Layout};

    // Outer vertical split: top (50%), bottom (50%)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(50), // top half for [tree][context]
            Constraint::Percentage(50), // bottom half for log
        ])
        .split(f.area());

    // Top row: split into left and right (file tree, context)
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // left: file tree
            Constraint::Percentage(50), // right: context
        ])
        .split(main_chunks[0]);

    // File Tree (top left)
    let tree_items: Vec<_> = app
        .file_tree
        .iter()
        .map(|f| {
            let prefix = if f.selected { "[x] " } else { "[ ] " };
            ListItem::new(format!("{prefix}{} ({})", f.name, human_size(f.size)))
        })
        .collect();
    let tree = List::new(tree_items)
        .block(Block::default().borders(Borders::ALL).title("File Tree"));
    f.render_widget(tree, top_chunks[0]);

    // Context (top right)
    let ctx = context_view::context_panel(&app.context)
        .block(Block::default().borders(Borders::ALL).title("Context"));
    f.render_widget(ctx, top_chunks[1]);

    // Log (bottom)
    let log =
        Paragraph::new(app.log.iter().cloned().collect::<Vec<_>>().join("\n"))
            .block(Block::default().borders(Borders::ALL).title("Log"));
    f.render_widget(log, main_chunks[1]);
}

/* pub fn draw(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(5),
            Constraint::Min(5),
        ])
        .split(f.area());

    // File Tree
    let tree_items: Vec<_> = app
        .file_tree
        .iter()
        .map(|f| {
            let prefix = if f.selected { "[x] " } else { "[ ] " };
            ListItem::new(format!("{prefix}{} ({})", f.name, human_size(f.size)))
        })
        .collect();
    let tree = List::new(tree_items)
        .block(Block::default().borders(Borders::ALL).title("File Tree"));
    f.render_widget(tree, chunks[0]);

    // Context
    let ctx = context_view::context_panel(&app.context)
        .block(Block::default().borders(Borders::ALL).title("Context"));
    f.render_widget(ctx, chunks[1]);

    // Log
    let log =
        Paragraph::new(app.log.iter().cloned().collect::<Vec<_>>().join("\n"))
            .block(Block::default().borders(Borders::ALL).title("Log"));
    f.render_widget(log, chunks[2]);
} */

// You can re-export this from app if you want
fn human_size(size: u64) -> String {
    crate::tui::app::human_size(size)
}
