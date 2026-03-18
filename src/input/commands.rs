use crate::engine::CalcError;
use crate::input::action::Action;

pub fn parse_command(input: &str) -> Result<Action, CalcError> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    match parts.as_slice() {
        [name, "STORE"] => Ok(Action::StoreRegister(name.to_string())),
        [name, "RCL"] => Ok(Action::RecallRegister(name.to_string())),
        _ => Err(CalcError::InvalidInput(format!(
            "Unknown command: {}",
            input
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::CalcError;
    use crate::input::action::Action;

    // ── STORE ────────────────────────────────────────────────────────────────

    #[test]
    fn test_store_command_simple() {
        assert_eq!(
            parse_command("myvar STORE"),
            Ok(Action::StoreRegister("myvar".to_string()))
        );
    }

    #[test]
    fn test_store_command_alphanumeric() {
        assert_eq!(
            parse_command("r1 STORE"),
            Ok(Action::StoreRegister("r1".to_string()))
        );
    }

    // ── RCL ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_rcl_command_simple() {
        assert_eq!(
            parse_command("myvar RCL"),
            Ok(Action::RecallRegister("myvar".to_string()))
        );
    }

    #[test]
    fn test_rcl_command_alphanumeric() {
        assert_eq!(
            parse_command("r1 RCL"),
            Ok(Action::RecallRegister("r1".to_string()))
        );
    }

    // ── errors ───────────────────────────────────────────────────────────────

    #[test]
    fn test_unknown_command_single_word() {
        // "STORE" alone has no register name — must fail
        assert!(matches!(
            parse_command("STORE"),
            Err(CalcError::InvalidInput(_))
        ));
    }

    #[test]
    fn test_unknown_command_garbage() {
        assert!(matches!(
            parse_command("not a command"),
            Err(CalcError::InvalidInput(_))
        ));
    }

    #[test]
    fn test_unknown_command_empty() {
        assert!(matches!(parse_command(""), Err(CalcError::InvalidInput(_))));
    }

    #[test]
    fn test_unknown_command_wrong_verb() {
        assert!(matches!(
            parse_command("myvar SAVE"),
            Err(CalcError::InvalidInput(_))
        ));
    }
}
