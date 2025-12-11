use gfxinfo::active_gpu;
use std::sync::Mutex;
#[cfg(test)]
use std::time::Duration;
use std::time::Instant;

/// Struct to hold vendor-agnostic GPU data.
#[derive(Debug, Clone)]
pub struct AgnosticGpuData {
    pub name: String,
    pub vendor: String,
    pub load_percent: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub temperature: f32,
}

struct IntelGpuState {
    last_rc6: u64,
    last_time: Instant,
}

static INTEL_STATE: Mutex<Option<IntelGpuState>> = Mutex::new(None);

fn get_intel_gpu_data() -> Option<AgnosticGpuData> {
    const RC6_PATH: &str = "/sys/class/drm/card1/gt/gt0/rc6_residency_ms";
    get_intel_gpu_data_from_path(RC6_PATH)
}

fn get_intel_gpu_data_from_path(rc6_path: &str) -> Option<AgnosticGpuData> {
    let current_rc6: u64 = std::fs::read_to_string(rc6_path)
        .ok()?
        .trim()
        .parse()
        .ok()?;
    let now = Instant::now();

    let mut state_lock = INTEL_STATE.lock().ok()?;

    let load_percent = if let Some(last_state) = state_lock.as_mut() {
        let duration = now.duration_since(last_state.last_time).as_millis() as u64;
        let rc6_delta = current_rc6.saturating_sub(last_state.last_rc6);

        // Update state
        last_state.last_rc6 = current_rc6;
        last_state.last_time = now;

        if duration == 0 {
            0.0
        } else if rc6_delta >= duration {
            // rc6 is "sleep time", so if we slept more than duration (or equal), we are 0% busy
            0.0
        } else {
            let busy_ms = duration - rc6_delta;
            (busy_ms as f64 / duration as f64) * 100.0
        }
    } else {
        // First run, initialize state and return 0
        *state_lock = Some(IntelGpuState {
            last_rc6: current_rc6,
            last_time: now,
        });
        0.0
    };

    Some(AgnosticGpuData {
        name: "Intel GPU".to_string(),
        vendor: "Intel".to_string(),
        load_percent,
        memory_used: 0, // Not easily available via sysfs without more digging
        memory_total: 0,
        temperature: 0.0, // Often n/a or different path
    })
}

/// Fetches agnostic GPU data using the gfxinfo crate.
pub fn get_agnostic_gpu_data() -> Option<AgnosticGpuData> {
    match active_gpu() {
        Ok(gpu) => {
            let info = gpu.info();
            let name = gpu.model().to_string(); // Or model/family combination
            let vendor = gpu.vendor().to_string();

            let load_percent = info.load_pct() as f64;
            let memory_used = info.used_vram();
            let memory_total = info.total_vram();
            let temperature = info.temperature() as f32 / 1000.0;

            Some(AgnosticGpuData {
                name,
                vendor,
                load_percent,
                memory_used,
                memory_total,
                temperature,
            })
        }
        Err(_e) => {
            // Fallback to Intel Sysfs
            if let Some(intel_data) = get_intel_gpu_data() {
                return Some(intel_data);
            }

            use std::sync::atomic::{AtomicBool, Ordering};
            static LOGGED_ERROR: AtomicBool = AtomicBool::new(false);

            if !LOGGED_ERROR.swap(true, Ordering::Relaxed) {
                crate::error!("Failed to detect agnostic/Intel GPU: {_e}");
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::thread;

    #[test]
    fn test_intel_gpu_detection() {
        // Create a temporary directory and file
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("rc6_residency_ms");
        let path_str = file_path.to_str().unwrap();

        // 1. Initialize logic (First run)
        // Write initial value 0
        {
            let mut file = File::create(&file_path).unwrap();
            write!(file, "0").unwrap();
        }

        // Reset state for test
        {
            let mut state = INTEL_STATE.lock().unwrap();
            *state = None;
        }

        let data = get_intel_gpu_data_from_path(path_str).expect("Should return data");
        assert_eq!(data.load_percent, 0.0, "First run should be 0.0%");

        // 2. Simulate "Busy" (rc6 doesn't change, time passes)
        // Wait a bit
        thread::sleep(Duration::from_millis(100));
        // RC6 stays 0

        let data = get_intel_gpu_data_from_path(path_str).expect("Should return data");
        // rc6 delta is 0, duration is > 0. usage = (duration - 0) / duration * 100 = 100%
        assert!(
            data.load_percent > 90.0,
            "Should be ~100% busy (actual: {})",
            data.load_percent
        );

        // 3. Simulate "Idle" (rc6 increases by duration)
        // Read the last time from state to verify correctness effectively,
        // but here we just sleep and update file.
        thread::sleep(Duration::from_millis(100));

        // In reality, rc6 is in ms. If we sleep 100ms, rc6 should increase by 100ms for 0% usage.
        // Determining exact previous RC6 is hard without peeking state, but we know it was 0.
        // So let's update it to typical value.
        // Since we did one check at t+100ms (rc6=0), now we are at t+200ms.
        // If we write 100 into rc6, delta will be 100. duration will be 100.
        // usage = (100 - 100) / 100 = 0%.
        {
            let mut file = File::create(&file_path).unwrap();
            write!(file, "100").unwrap();
        }

        let data = get_intel_gpu_data_from_path(path_str).expect("Should return data");
        // It's possible duration is slightly > 100ms due to scheduling, so usage might be slightly > 0,
        // but should be low.
        assert!(
            data.load_percent < 10.0,
            "Should be ~0% busy aka idle (actual: {})",
            data.load_percent
        );
    }
}
