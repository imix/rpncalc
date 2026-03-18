// TODO: Story 4.4 — Full Config struct with config.toml loading

use crate::engine::{angle::AngleMode, base::Base};

#[allow(dead_code)]
pub struct Config {
    pub angle_mode: AngleMode,
    pub base: Base,
    pub precision: usize,
    pub max_undo_history: usize,
    pub persist_session: bool,
}

impl Config {
    pub fn load() -> Self {
        Self::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            angle_mode: AngleMode::Deg,
            base: Base::Dec,
            precision: 15,
            max_undo_history: 1000,
            persist_session: true,
        }
    }
}
