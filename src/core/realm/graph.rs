use super::UniversalState;
pub use vulfram_realm_core::{RealmGraphEdge, RealmGraphPlan};

#[derive(Debug, Default)]
pub struct RealmGraphPlanner;

impl RealmGraphPlanner {
    pub fn build_plan(&self, universal: &UniversalState) -> RealmGraphPlan {
        let realm_output_surfaces = universal
            .realms
            .entries
            .iter()
            .map(|(realm_id, entry)| (*realm_id, entry.value.output_surface))
            .collect();
        let presents = universal
            .presents
            .entries
            .values()
            .map(|present| (present.value.window_id, present.value.surface))
            .collect::<Vec<_>>();
        let connectors = universal
            .connectors
            .entries
            .iter()
            .map(|(connector_id, connector)| {
                (
                    *connector_id,
                    connector.value.source_surface,
                    connector.value.target_realm,
                )
            })
            .collect::<Vec<_>>();

        vulfram_realm_core::RealmGraphPlanner.build_plan(
            &realm_output_surfaces,
            &presents,
            &connectors,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::realm::{
        ConnectorState, PresentState, RealmKind, RealmState, SurfaceKind, SurfaceState,
    };

    fn realm_state(output_surface: Option<crate::core::realm::SurfaceId>) -> RealmState {
        RealmState {
            kind: RealmKind::TwoD,
            output_surface,
            render_graph_id: None,
            importance: 1,
            cache_policy: 0,
            last_render_frame: 0,
        }
    }

    #[test]
    fn planner_orders_linear_dependency() {
        let mut universal = UniversalState::default();
        let surface_a = universal.surfaces.alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: glam::UVec2::new(32, 32),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
        let surface_b = universal.surfaces.alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: glam::UVec2::new(32, 32),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
        let realm_a = universal.realms.alloc(realm_state(Some(surface_a)));
        let realm_b = universal.realms.alloc(realm_state(Some(surface_b)));

        let _connector = universal.connectors.alloc(ConnectorState {
            target_realm: realm_b,
            source_surface: surface_a,
            rect: glam::Vec4::ZERO,
            z_index: 0,
            blend_mode: 0,
            clip: None,
            input_flags: 0,
        });
        let _present = universal.presents.alloc(PresentState {
            window_id: 1,
            surface: surface_b,
        });

        let plan = RealmGraphPlanner.build_plan(&universal);
        assert_eq!(plan.order, vec![realm_a, realm_b]);
        assert!(plan.cut_edges.is_empty());
    }

    #[test]
    fn planner_cuts_cycle_and_keeps_deterministic_order() {
        let mut universal = UniversalState::default();
        let surface_a = universal.surfaces.alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: glam::UVec2::new(32, 32),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
        let surface_b = universal.surfaces.alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: glam::UVec2::new(32, 32),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
        let realm_a = universal.realms.alloc(realm_state(Some(surface_a)));
        let realm_b = universal.realms.alloc(realm_state(Some(surface_b)));

        let _ab = universal.connectors.alloc(ConnectorState {
            target_realm: realm_b,
            source_surface: surface_a,
            rect: glam::Vec4::ZERO,
            z_index: 0,
            blend_mode: 0,
            clip: None,
            input_flags: 0,
        });
        let _ba = universal.connectors.alloc(ConnectorState {
            target_realm: realm_a,
            source_surface: surface_b,
            rect: glam::Vec4::ZERO,
            z_index: 0,
            blend_mode: 0,
            clip: None,
            input_flags: 0,
        });

        let plan = RealmGraphPlanner.build_plan(&universal);
        assert_eq!(plan.order, vec![realm_a, realm_b]);
        assert_eq!(plan.cut_edges.len(), 2);
    }

    #[test]
    fn planner_includes_disconnected_realms() {
        let mut universal = UniversalState::default();
        let surface_a = universal.surfaces.alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: glam::UVec2::new(16, 16),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
        let realm_a = universal.realms.alloc(realm_state(Some(surface_a)));
        let realm_b = universal.realms.alloc(realm_state(None));

        let plan = RealmGraphPlanner.build_plan(&universal);
        assert_eq!(plan.order, vec![realm_a, realm_b]);
        assert!(plan.cut_edges.is_empty());
    }
}
