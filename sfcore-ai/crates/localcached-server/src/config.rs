use std::env;
use std::sync::atomic::{AtomicU16, Ordering};

#[derive(Clone, Debug)]
pub struct Config {
    pub socket_path: String,
    pub max_frame_bytes: usize,
    pub pressure_hot: f64,
    pub pressure_cool: f64,
    pub pubsub_capacity: usize,
    pub pressure_poll_ms: u64,
    pub max_concurrent_ops: usize, // Backpressure: max concurrent operations
    pub pid_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            socket_path: "/run/localcached.sock".to_string(),
            pid_path: "/run/localcached.pid".to_string(),
            max_frame_bytes: 8 * 1024 * 1024,
            pressure_hot: 0.85,
            pressure_cool: 0.80,
            pubsub_capacity: 256,
            pressure_poll_ms: 150,
            max_concurrent_ops: 10_000, // 10k concurrent ops before backpressure
        }
    }
}

/// Maximum allowed pressure limit (85%)
pub const MAX_PRESSURE_HOT: f64 = 0.85;

impl Config {
    pub fn from_env() -> Self {
        let mut c = Self::default();
        if let Ok(v) = env::var("LOCALCACHED_SOCKET") {
            c.socket_path = v;
            // Auto derive PID path from socket path by default (replace extension with .pid)
            // But only if PID env is not set (checked below)
            let path = std::path::Path::new(&c.socket_path);
            if let Some(parent) = path.parent() {
                let stem = path.file_stem().unwrap_or_default();
                let mut pid_name = stem.to_os_string();
                pid_name.push(".pid");
                c.pid_path = parent.join(pid_name).to_string_lossy().to_string();
            }
        }
        if let Ok(v) = env::var("LOCALCACHED_PID_FILE") {
            c.pid_path = v;
        }

        if let Ok(v) = env::var("LOCALCACHED_MAX_FRAME") {
            c.max_frame_bytes = v.parse().unwrap_or(c.max_frame_bytes);
        }
        if let Ok(v) = env::var("LOCALCACHED_PRESSURE_HOT") {
            let parsed: f64 = v.parse().unwrap_or(c.pressure_hot);
            // Validate: max 85%
            if parsed > MAX_PRESSURE_HOT {
                eprintln!(
                    "WARNING: LOCALCACHED_PRESSURE_HOT={:.0}% exceeds maximum 85%. Clamping to 85%.",
                    parsed * 100.0
                );
                c.pressure_hot = MAX_PRESSURE_HOT;
            } else if parsed < 0.01 {
                eprintln!(
                    "WARNING: LOCALCACHED_PRESSURE_HOT={:.0}% is too low. Using minimum 1%.",
                    parsed * 100.0
                );
                c.pressure_hot = 0.01;
            } else {
                c.pressure_hot = parsed;
            }
        }
        if let Ok(v) = env::var("LOCALCACHED_PRESSURE_COOL") {
            c.pressure_cool = v.parse().unwrap_or(c.pressure_cool);
        }
        if let Ok(v) = env::var("LOCALCACHED_PUBSUB_CAP") {
            c.pubsub_capacity = v.parse().unwrap_or(c.pubsub_capacity);
        }
        if let Ok(v) = env::var("LOCALCACHED_PRESSURE_POLL_MS") {
            c.pressure_poll_ms = v.parse().unwrap_or(c.pressure_poll_ms);
        }
        if let Ok(v) = env::var("LOCALCACHED_MAX_CONCURRENT_OPS") {
            c.max_concurrent_ops = v.parse().unwrap_or(c.max_concurrent_ops);
        }
        c
    }
}

/// Runtime-modifiable configuration (atomic for thread safety)
pub struct RuntimeConfig {
    /// Memory pressure threshold in basis points (0-10000, e.g., 8500 = 85%)
    pub pressure_hot_bp: AtomicU16,
}

impl RuntimeConfig {
    pub fn new(initial_hot: f64) -> Self {
        let bp = (initial_hot * 10000.0) as u16;
        Self {
            pressure_hot_bp: AtomicU16::new(bp),
        }
    }

    /// Get current threshold as float (0.0-1.0)
    pub fn get_pressure_hot(&self) -> f64 {
        self.pressure_hot_bp.load(Ordering::Relaxed) as f64 / 10000.0
    }

    /// Get current threshold in basis points
    pub fn get_pressure_hot_bp(&self) -> u16 {
        self.pressure_hot_bp.load(Ordering::Relaxed)
    }

    /// Set new threshold (basis points, 0-10000)
    /// Returns the previous value
    pub fn set_pressure_hot_bp(&self, new_bp: u16) -> u16 {
        let clamped = new_bp.min(10000);
        self.pressure_hot_bp.swap(clamped, Ordering::SeqCst)
    }
}
