// TODO: Story 2.1 — App struct with full CalcState + UndoHistory + AppMode + error

#[allow(dead_code)]
pub struct App {
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self { should_quit: false }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
