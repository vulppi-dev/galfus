#[cfg(target_os = "linux")]
use super::{parse_status_kib_line, read_process_cpu_ticks, read_system_cpu_ticks};

#[cfg(target_os = "linux")]
#[test]
fn parse_status_kib_line_extracts_numeric_value() {
    assert_eq!(
        parse_status_kib_line("VmRSS:\t  12345 kB", "VmRSS:"),
        Some(12_345)
    );
    assert_eq!(parse_status_kib_line("VmHWM:\t9 kB", "VmHWM:"), Some(9));
    assert_eq!(parse_status_kib_line("Threads:\t2", "VmRSS:"), None);
}

#[cfg(target_os = "linux")]
#[test]
fn process_cpu_tick_sources_are_readable() {
    assert!(read_process_cpu_ticks().is_some());
    assert!(read_system_cpu_ticks().is_some());
}
