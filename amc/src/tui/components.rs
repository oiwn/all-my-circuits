pub mod context_view;
pub mod file_tree;

use crossterm::event::KeyCode;
use ratatui::{Frame, layout::Rect};

pub trait Focusable {
    fn handle_key(&mut self, key: KeyCode) -> bool;
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool);
}
