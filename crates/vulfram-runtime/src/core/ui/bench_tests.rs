use super::run_ui_bench_scenarios;

#[test]
fn ui_bench_scenarios_smoke() {
    let results = run_ui_bench_scenarios(2);
    assert_eq!(results.len(), 4);
    assert!(results.iter().all(|result| {
        result.samples == 2
            && !result.scenario.is_empty()
            && result.avg_layout_ms.is_finite()
            && result.avg_layout_ms >= 0.0
            && result.avg_tessellation_ms.is_finite()
            && result.avg_tessellation_ms >= 0.0
            && result.max_total_ms.is_finite()
            && result.max_total_ms >= 0.0
    }));
}
