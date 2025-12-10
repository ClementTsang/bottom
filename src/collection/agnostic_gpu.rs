use gfxinfo::active_gpu;

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

/// Fetches agnostic GPU data using the gfxinfo crate.
pub fn get_agnostic_gpu_data() -> Option<AgnosticGpuData> {
    if let Ok(gpu) = active_gpu() {
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
    } else {
        None
    }
}
