pub fn memory_limit_mb() -> Option<u64> {
    let mem_kb = std::fs::read_to_string("/proc/meminfo")
        .ok()?
        .lines()
        .find(|l| l.starts_with("MemTotal:"))?
        .split_whitespace()
        .nth(1)?
        .parse::<u64>().ok()?;
    Some((mem_kb as f64 * 0.30 / 1024.0) as u64)
}

pub fn has_nvidia_gpu() -> bool {
    std::process::Command::new("lspci")
        .output()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            s.contains("NVIDIA") || s.contains("nVidia")
        })
        .unwrap_or(false)
}
