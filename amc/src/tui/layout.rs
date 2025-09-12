use crate::tui::app::{AppState, PanelFocus};
use crate::tui::components::Focusable;
use ratatui::{
    Frame,
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &AppState) {
    use ratatui::layout::{Constraint, Direction, Layout};

    // Outer vertical split: top (50%), bottom (50%)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(50), // top half for [tree][context]
            Constraint::Percentage(50), // bottom half for file preview
        ])
        .split(f.area());

    // Top row: split into left and right (file tree, context)
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // left: file tree
            Constraint::Percentage(50), // right: context (swapped from log)
        ])
        .split(main_chunks[0]);

    // Render components with focus information
    app.file_tree
        .render(f, top_chunks[0], matches!(app.focused, PanelFocus::Tree));
    app.context.render(
        f,
        top_chunks[1],
        matches!(app.focused, PanelFocus::Context),
    );

    // File Preview (bottom) - was Log before
    let preview_focused = matches!(app.focused, PanelFocus::Preview);
    let border_style = if preview_focused {
        ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
    } else {
        ratatui::style::Style::default()
    };

    let preview = Paragraph::new(app.file_preview.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("File Preview")
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(preview, main_chunks[1]);
}
