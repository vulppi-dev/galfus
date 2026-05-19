use super::*;
use crate::core::test_support::test_engine;

#[test]
fn system_build_version_get_returns_pkg_version() {
    let mut engine = test_engine();
    let result = engine_cmd_system_build_version_get(&mut engine, &CmdSystemBuildVersionGetArgs {});
    assert!(result.success);
    assert_eq!(result.build_version, env!("CARGO_PKG_VERSION"));
}
