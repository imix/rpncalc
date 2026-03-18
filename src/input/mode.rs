// TODO: Story 2.1 — Full AppMode state machine integration

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ChordCategory {
    Trig,
    Log,
    Functions,
    Constants,
    AngleMode,
    Base,
    HexStyle,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Alpha(String),
    Chord(ChordCategory),
}
