use crate::core::profiling::state::TickProfiling;

pub fn refresh_process_metrics(profiling: &mut TickProfiling) {
    #[cfg(target_os = "linux")]
    {
        let metrics = LinuxProcessMetrics::sample(
            &mut profiling.process_cpu_ticks,
            &mut profiling.system_cpu_ticks,
        );
        profiling.memory.ram_bytes_current = metrics.ram_bytes_current;
        profiling.memory.ram_bytes_peak = profiling
            .memory
            .ram_bytes_peak
            .max(metrics.ram_bytes_peak.max(metrics.ram_bytes_current));
        profiling.utilization.cpu_percent = metrics.cpu_percent;
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = profiling;
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, Default)]
struct LinuxProcessMetrics {
    ram_bytes_current: u64,
    ram_bytes_peak: u64,
    cpu_percent: f32,
}

#[cfg(target_os = "linux")]
impl LinuxProcessMetrics {
    fn sample(process_cpu_ticks: &mut Option<u64>, system_cpu_ticks: &mut Option<u64>) -> Self {
        let (ram_bytes_current, ram_bytes_peak) = read_status_memory_bytes();
        let cpu_percent = sample_process_cpu_percent(process_cpu_ticks, system_cpu_ticks);
        Self {
            ram_bytes_current,
            ram_bytes_peak,
            cpu_percent,
        }
    }
}

#[cfg(target_os = "linux")]
fn read_status_memory_bytes() -> (u64, u64) {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return (0, 0);
    };
    let mut current = 0;
    let mut peak = 0;
    for line in status.lines() {
        if let Some(value) = parse_status_kib_line(line, "VmRSS:") {
            current = value.saturating_mul(1024);
        } else if let Some(value) = parse_status_kib_line(line, "VmHWM:") {
            peak = value.saturating_mul(1024);
        }
    }
    (current, peak)
}

#[cfg(target_os = "linux")]
fn sample_process_cpu_percent(
    process_cpu_ticks: &mut Option<u64>,
    system_cpu_ticks: &mut Option<u64>,
) -> f32 {
    let Some(next_process_ticks) = read_process_cpu_ticks() else {
        return 0.0;
    };
    let Some(next_system_ticks) = read_system_cpu_ticks() else {
        return 0.0;
    };

    let percent = match (*process_cpu_ticks, *system_cpu_ticks) {
        (Some(prev_process), Some(prev_system)) if next_system_ticks > prev_system => {
            let process_delta = next_process_ticks.saturating_sub(prev_process) as f32;
            let system_delta = next_system_ticks.saturating_sub(prev_system) as f32;
            let cpu_count = std::thread::available_parallelism()
                .map(|count| count.get() as f32)
                .unwrap_or(1.0);
            ((process_delta / system_delta) * cpu_count * 100.0).clamp(0.0, 100.0 * cpu_count)
        }
        _ => 0.0,
    };

    *process_cpu_ticks = Some(next_process_ticks);
    *system_cpu_ticks = Some(next_system_ticks);
    percent
}

#[cfg(target_os = "linux")]
fn read_process_cpu_ticks() -> Option<u64> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    let close = stat.rfind(')')?;
    let tail = stat.get(close + 2..)?;
    let fields: Vec<&str> = tail.split_whitespace().collect();
    let utime = fields.get(11)?.parse::<u64>().ok()?;
    let stime = fields.get(12)?.parse::<u64>().ok()?;
    Some(utime.saturating_add(stime))
}

#[cfg(target_os = "linux")]
fn read_system_cpu_ticks() -> Option<u64> {
    let stat = std::fs::read_to_string("/proc/stat").ok()?;
    let line = stat.lines().next()?;
    let mut parts = line.split_whitespace();
    if parts.next()? != "cpu" {
        return None;
    }
    let mut total = 0_u64;
    for value in parts {
        total = total.saturating_add(value.parse::<u64>().ok()?);
    }
    Some(total)
}

#[cfg(target_os = "linux")]
fn parse_status_kib_line(line: &str, key: &str) -> Option<u64> {
    let value = line.strip_prefix(key)?.trim();
    let number = value.split_whitespace().next()?;
    number.parse::<u64>().ok()
}

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;
