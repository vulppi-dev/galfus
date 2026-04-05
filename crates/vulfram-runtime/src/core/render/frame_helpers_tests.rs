use super::build_soft_cut_diagnostic;

#[test]
fn build_soft_cut_diagnostic_reports_new_cut_set() {
    let frame_report = crate::core::realm::FrameReport {
        cut_edges: vec![crate::core::realm::FrameCutEdge {
            from: 1,
            to: 2,
            connector_id: Some(9),
        }],
        ..Default::default()
    };

    let diagnostic = build_soft_cut_diagnostic(&frame_report, 0, 42);
    assert_eq!(
        diagnostic.as_deref(),
        Some("frame=42 cut_edges=1 connectors=9")
    );
}
