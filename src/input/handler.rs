// TODO: Story 2.4 — handle_key pure function implementation

use crate::input::{action::Action, mode::AppMode};

#[allow(dead_code)]
pub fn handle_key(_mode: &AppMode, _event: crossterm::event::KeyEvent) -> Action {
    Action::Noop
}
