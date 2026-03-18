use crate::engine::stack::CalcState;

pub struct UndoHistory {
    past: Vec<CalcState>,
    future: Vec<CalcState>,
    max_depth: usize,
}

impl UndoHistory {
    pub fn new() -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            max_depth: 1000,
        }
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            max_depth,
        }
    }

    /// Save a snapshot of the current state before an operation.
    pub fn snapshot(&mut self, state: &CalcState) {
        self.past.push(state.clone());
        self.future.clear();
        if self.past.len() > self.max_depth {
            self.past.remove(0);
        }
    }

    /// Undo: restore previous state, pushing current into future.
    pub fn undo(&mut self, current: &CalcState) -> Option<CalcState> {
        if let Some(prev) = self.past.pop() {
            self.future.push(current.clone());
            Some(prev)
        } else {
            None
        }
    }

    /// Redo: re-apply a previously undone operation.
    pub fn redo(&mut self, current: &CalcState) -> Option<CalcState> {
        if let Some(next) = self.future.pop() {
            self.past.push(current.clone());
            Some(next)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::value::CalcValue;
    use dashu::integer::IBig;

    fn state_with_value(v: i32) -> CalcState {
        let mut s = CalcState::new();
        s.stack.push(CalcValue::Integer(IBig::from(v)));
        s
    }

    #[test]
    fn test_snapshot_pushes_to_past() {
        let mut hist = UndoHistory::new();
        let s = state_with_value(1);
        hist.snapshot(&s);
        assert!(hist.can_undo());
        assert!(!hist.can_redo());
    }

    #[test]
    fn test_undo_restores_state() {
        let mut hist = UndoHistory::new();
        let before = state_with_value(1);
        let after = state_with_value(2);
        hist.snapshot(&before);
        let restored = hist.undo(&after).unwrap();
        assert_eq!(restored.stack[0].to_f64(), 1.0);
        assert!(hist.can_redo());
    }

    #[test]
    fn test_redo_reapplies_state() {
        let mut hist = UndoHistory::new();
        let before = state_with_value(1);
        let after = state_with_value(2);
        hist.snapshot(&before);
        let _ = hist.undo(&after);
        // redo restores the state we undid FROM (after=2), not the snapshot
        let redone = hist.redo(&before).unwrap();
        assert_eq!(redone.stack[0].to_f64(), 2.0);
    }

    #[test]
    fn test_new_action_after_undo_clears_redo() {
        let mut hist = UndoHistory::new();
        let s1 = state_with_value(1);
        let s2 = state_with_value(2);
        hist.snapshot(&s1);
        let _ = hist.undo(&s2);
        assert!(hist.can_redo());
        hist.snapshot(&s2);
        assert!(!hist.can_redo());
    }

    #[test]
    fn test_depth_limiting_discards_oldest() {
        let mut hist = UndoHistory::with_max_depth(3);
        for i in 0..5 {
            hist.snapshot(&state_with_value(i));
        }
        assert_eq!(hist.past.len(), 3);
    }
}
