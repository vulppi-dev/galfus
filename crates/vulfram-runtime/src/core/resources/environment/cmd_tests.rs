use super::*;
use crate::core::resources::{EnvironmentConfig, SkyboxConfig, SkyboxMode};

#[test]
fn create_behaves_as_upsert_for_existing_environment_id() {
    let mut engine = crate::core::state::EngineState::new();
    let id = 100_u32;

    let _first = engine_cmd_environment_create(
        &mut engine,
        &CmdEnvironmentCreateArgs {
            environment_id: id,
            config: EnvironmentConfig {
                skybox: SkyboxConfig {
                    mode: SkyboxMode::None,
                    ..Default::default()
                },
                ..Default::default()
            },
        },
    );
    let second = engine_cmd_environment_create(
        &mut engine,
        &CmdEnvironmentCreateArgs {
            environment_id: id,
            config: EnvironmentConfig {
                skybox: SkyboxConfig {
                    mode: SkyboxMode::Procedural,
                    ..Default::default()
                },
                ..Default::default()
            },
        },
    );

    assert!(second.success);
    assert_eq!(
        engine.universal_state.scene.realm3d.default_environment_id,
        Some(id)
    );
    assert_eq!(
        engine
            .universal_state
            .scene
            .realm3d
            .environment_profiles
            .get(&id)
            .map(|env| env.skybox.mode),
        Some(SkyboxMode::Procedural)
    );
}
