use super::*;

#[test]
fn system_build_version_get_returns_pkg_version() {
    let mut engine = EngineState::new();
    let result = engine_cmd_system_build_version_get(&mut engine, &CmdSystemBuildVersionGetArgs {});
    assert!(result.success);
    assert_eq!(result.build_version, env!("CARGO_PKG_VERSION"));
}
