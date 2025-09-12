use super::file_tree::FileNode;
use crate::tui::app::human_size;
use crate::tui::components::Focusable;
use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
};

pub struct ContextComponent {
    pub items: Vec<FileNode>,
    pub selected_index: usize,
}

impl ContextComponent {
    pub fn new() -> Self {
        Self {
            items: vec![],
            selected_index: 0,
        }
    }

    pub fn navigate_up(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.items.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn navigate_down(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = if self.selected_index >= self.items.len() - 1 {
                0
            } else {
                self.selected_index + 1
            };
        }
    }
}

impl Focusable for ContextComponent {
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
            _ => false,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let items: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let marker = if i == self.selected_index && focused {
                    "► "
                } else {
                    "  "
                };
                ListItem::new(format!(
                    "{}{} ({})",
                    marker,
                    f.name,
                    human_size(f.size)
                ))
            })
            .collect();

        let border_style = if focused {
            ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
        } else {
            ratatui::style::Style::default()
        };

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Context")
                .border_style(border_style),
        );
        frame.render_widget(list, area);
    }
}
