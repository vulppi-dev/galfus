use super::UniversalState;
pub use galfus_realm_core::{RealmGraphEdge, RealmGraphPlan};

#[derive(Debug, Default)]
pub struct RealmGraphPlanner;

impl RealmGraphPlanner {
    pub fn build_plan(&self, universal: &UniversalState) -> RealmGraphPlan {
        let realm_output_surfaces = universal
            .composition
            .realms
            .entries
            .iter()
            .map(|(realm_id, entry)| (*realm_id, entry.value.output_surface))
            .collect();
        let presents = universal
            .composition
            .presents
            .entries
            .values()
            .map(|present| (present.value.window_id, present.value.surface))
            .collect::<Vec<_>>();
        let connectors = universal
            .composition
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

        galfus_realm_core::RealmGraphPlanner.build_plan(
            &realm_output_surfaces,
            &presents,
            &connectors,
        )
    }
}

#[cfg(test)]
#[path = "graph_tests.rs"]
mod tests;
