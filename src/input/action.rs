use crate::engine::angle::AngleMode;
use crate::engine::base::{Base, HexStyle};
use crate::engine::ops::Op;
use crate::engine::value::CalcValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Push(CalcValue),
    Execute(Op),
    SetBase(Base),
    SetAngleMode(AngleMode),
    SetHexStyle(HexStyle),
    StoreRegister(String),
    RecallRegister(String),
    DeleteRegister(String),
    Undo,
    Redo,
    Yank,
    EnterAlphaMode,
    Quit,
    Noop,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dashu::integer::IBig;

    #[test]
    fn test_action_constructible() {
        let _ = Action::Push(CalcValue::Integer(IBig::from(42)));
        let _ = Action::Execute(Op::Sin);
        let _ = Action::Undo;
    }
}
