use std::fs;

#[derive(Debug, Clone, Copy)]
pub struct MemInfo {
    pub mem_total_kb: u64,
    pub mem_available_kb: u64,
}

pub fn read_meminfo() -> std::io::Result<MemInfo> {
    let s = fs::read_to_string("/proc/meminfo")?;
    let mut total = 0u64;
    let mut avail = 0u64;

    for line in s.lines() {
        if line.starts_with("MemTotal:") {
            total = line
                .split_whitespace()
                .nth(1)
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            avail = line
                .split_whitespace()
                .nth(1)
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
        }
    }
    Ok(MemInfo {
        mem_total_kb: total,
        mem_available_kb: avail,
    })
}

pub fn pressure_bp(mi: MemInfo) -> u16 {
    if mi.mem_total_kb == 0 {
        return 0;
    }
    let avail = mi.mem_available_kb as f64;
    let total = mi.mem_total_kb as f64;
    let p = 1.0 - (avail / total);
    let bp = (p * 10000.0).clamp(0.0, 10000.0) as u16;
    bp
}

pub fn pressure(mi: MemInfo) -> f64 {
    if mi.mem_total_kb == 0 {
        return 0.0;
    }
    1.0 - (mi.mem_available_kb as f64 / mi.mem_total_kb as f64)
}
