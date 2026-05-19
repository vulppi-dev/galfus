use super::{percentile_us, rolling_from_samples};
use crate::core::profiling::state::FrameProfilingSample;
use std::collections::VecDeque;

#[test]
fn percentile_uses_sorted_rank() {
    let mut values = [3_000, 1_000, 2_000, 4_000];
    assert_eq!(percentile_us(&mut values, 0.50), 3.0);
    assert_eq!(percentile_us(&mut values, 0.95), 4.0);
}

#[test]
fn rolling_window_reports_percentiles_and_max() {
    let samples = VecDeque::from([
        FrameProfilingSample {
            command_ns: 1_000,
            input_ns: 2_000,
            render_ns: 4_000,
            gpu_ns: 8_000,
            frame_delta_ns: 16_000,
        },
        FrameProfilingSample {
            command_ns: 2_000,
            input_ns: 3_000,
            render_ns: 8_000,
            gpu_ns: 16_000,
            frame_delta_ns: 32_000,
        },
        FrameProfilingSample {
            command_ns: 3_000,
            input_ns: 4_000,
            render_ns: 12_000,
            gpu_ns: 24_000,
            frame_delta_ns: 48_000,
        },
    ]);

    let rolling = rolling_from_samples(&samples);
    assert_eq!(rolling.sample_count, 3);
    assert_eq!(rolling.frame_us_p50, 32.0);
    assert_eq!(rolling.frame_us_p95, 48.0);
    assert_eq!(rolling.frame_us_p99, 48.0);
    assert_eq!(rolling.frame_us_max, 48.0);
    assert_eq!(rolling.render_us_p95, 12.0);
    assert_eq!(rolling.gpu_us_p95, 24.0);
}
