// Session roundtrip serialization is tested as unit tests in src/config/session.rs
// (test_save_and_load_roundtrip, test_roundtrip_preserves_registers).
//
// This binary crate does not expose a lib target, so integration tests cannot
// import internal types like CalcState directly. The unit tests in session.rs
// cover the full save_to_path/load_from_path round-trip with temp files.

#[test]
fn session_integration_placeholder() {
    // Covered by src/config/session.rs::tests::test_save_and_load_roundtrip
    // and test_roundtrip_preserves_registers
}
