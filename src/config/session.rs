use crate::config::config::Config;
use crate::engine::stack::CalcState;
use crate::engine::units::lookup_unit;
use crate::engine::value::CalcValue;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Returns true if the loaded state contains any TaggedValue whose dim field is
/// all-zeros (old format default) but whose unit abbreviation maps to a non-zero
/// dim in the current registry. This detects sessions saved before the
/// compound-unit-model refactor.
fn has_old_format_tagged_values(state: &CalcState) -> bool {
    let check = |val: &CalcValue| -> bool {
        if let CalcValue::Tagged(t) = val {
            if t.dim.is_dimensionless() {
                if let Some(unit) = lookup_unit(&t.unit) {
                    return !unit.dim.is_dimensionless();
                }
            }
        }
        false
    };
    state.stack.iter().any(check)
        || state.registers.values().any(check)
}

pub fn session_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".rpnpad").join("session.json"))
}

/// Core save — testable with injected path. Atomic: write temp then rename.
pub(crate) fn save_to_path(
    path: &Path,
    state: &CalcState,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_string(state)?;
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, &json)?;
    fs::rename(&temp, path)?; // atomic on same filesystem
    Ok(())
}

/// Core load — testable with injected path.
/// Returns Ok(None) if file not found, Err(msg) if corrupt or IO error.
/// Returns Err("unit format updated") if the session contains old-format
/// TaggedValues (pre-compound-unit-model) — caller should show migration message.
pub(crate) fn load_from_path(path: &Path) -> Result<Option<CalcState>, String> {
    let data = match fs::read_to_string(path) {
        Ok(d) => d,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("IO error reading session: {}", e)),
    };
    let state: CalcState = serde_json::from_str(&data)
        .map_err(|e| format!("Session file corrupt: {}", e))?;
    if has_old_format_tagged_values(&state) {
        return Err("unit format updated".to_string());
    }
    Ok(Some(state))
}

/// Save session to ~/.rpnpad/session.json using atomic write.
/// Respects Config::persist_session — no-op if false.
/// Best-effort: callers use `let _ = session::save(...)`.
pub fn save(state: &CalcState) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();
    if !config.persist_session {
        return Ok(());
    }
    let path = session_path().ok_or("cannot resolve home directory")?;
    save_to_path(&path, state)
}

/// Load session from ~/.rpnpad/session.json.
/// Returns Ok(None) if no session file exists (first launch) or persist_session is false.
/// Returns Err(msg) if the file is present but corrupt — caller shows message.
pub fn load() -> Result<Option<CalcState>, String> {
    let config = Config::load();
    if !config.persist_session {
        return Ok(None); // persist disabled — always start fresh
    }
    let path = match session_path() {
        Some(p) => p,
        None => return Ok(None),
    };
    load_from_path(&path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::value::CalcValue;
    use dashu::integer::IBig;

    fn state_with_value(n: i32) -> CalcState {
        let mut s = CalcState::new();
        s.stack.push(CalcValue::Integer(IBig::from(n)));
        s
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = std::env::temp_dir();
        let path = dir.join("rpnpad_test_session.json");
        let _ = std::fs::remove_file(&path);

        let state = state_with_value(42);
        save_to_path(&path, &state).unwrap();
        let loaded = load_from_path(&path).unwrap().unwrap();
        assert_eq!(loaded.stack.len(), 1);
        assert_eq!(loaded.stack[0].to_f64(), 42.0);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_load_returns_none_when_no_file() {
        let path = std::env::temp_dir().join("rpnpad_nonexistent_session_4321.json");
        let _ = std::fs::remove_file(&path);
        assert!(load_from_path(&path).unwrap().is_none());
    }

    #[test]
    fn test_load_returns_err_on_corrupt_file() {
        let path = std::env::temp_dir().join("rpnpad_corrupt_session_4321.json");
        std::fs::write(&path, b"not valid json at all").unwrap();
        assert!(load_from_path(&path).is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_save_creates_directory() {
        let dir = std::env::temp_dir().join("rpnpad_test_dir_creation_4321");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("session.json");
        let state = CalcState::new();
        save_to_path(&path, &state).unwrap();
        assert!(path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── AC-4: old session format migration ───────────────────────────────────

    #[test]
    fn test_old_format_session_migration() {
        // Build an old-format session by serialising a real TaggedValue and
        // stripping the "dim" field — simulates a session saved before the
        // compound-unit-model refactor added DimensionVector.
        use crate::engine::units::TaggedValue;
        let mut state = CalcState::new();
        state.stack.push(CalcValue::Tagged(TaggedValue::new(1.9, "oz")));
        let new_json = serde_json::to_string(&state).unwrap();
        // Strip "dim" from every Tagged entry to produce old-format JSON.
        let mut state_val: serde_json::Value = serde_json::from_str(&new_json).unwrap();
        if let Some(stack) = state_val["stack"].as_array_mut() {
            for item in stack.iter_mut() {
                if let Some(tagged) = item["Tagged"].as_object_mut() {
                    tagged.remove("dim");
                }
            }
        }
        let old_json = serde_json::to_string(&state_val).unwrap();

        let path = std::env::temp_dir().join("rpnpad_old_format_migration_test.json");
        std::fs::write(&path, &old_json).unwrap();
        let result = load_from_path(&path);
        let _ = std::fs::remove_file(&path);
        assert!(
            matches!(result, Err(ref e) if e == "unit format updated"),
            "expected 'unit format updated' error, got {:?}",
            result
        );
    }

    #[test]
    fn test_new_format_session_loads_normally() {
        use crate::engine::units::TaggedValue;
        let dir = std::env::temp_dir();
        let path = dir.join("rpnpad_new_format_session_test.json");
        let _ = std::fs::remove_file(&path);

        let mut state = CalcState::new();
        state
            .stack
            .push(CalcValue::Tagged(TaggedValue::new(1.9, "oz")));
        save_to_path(&path, &state).unwrap();

        let loaded = load_from_path(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert!(loaded.is_some(), "new format session should load successfully");
    }

    #[test]
    fn test_roundtrip_preserves_registers() {
        let dir = std::env::temp_dir();
        let path = dir.join("rpnpad_test_registers_4321.json");
        let _ = std::fs::remove_file(&path);

        let mut state = CalcState::new();
        state
            .registers
            .insert("x".to_string(), CalcValue::Integer(IBig::from(99)));

        save_to_path(&path, &state).unwrap();
        let loaded = load_from_path(&path).unwrap().unwrap();
        assert!(loaded.registers.contains_key("x"));
        assert_eq!(loaded.registers["x"].to_f64(), 99.0);

        let _ = std::fs::remove_file(&path);
    }
}
