use gfxinfo::active_gpu;

fn main() {
    println!("Initializing GPU info...");
    match active_gpu() {
        Ok(gpu) => {
            println!("Detected GPU:");
            println!("  Vendor: {}", gpu.vendor());
            println!("  Model: {}", gpu.model());
            println!("  Family: {}", gpu.family());
            println!("  Device ID: {}", gpu.device_id());

            println!("\nFetching dynamic metrics...");
            let info = gpu.info();
            println!(
                "  VRAM: {} / {} MB",
                info.used_vram() / 1024 / 1024,
                info.total_vram() / 1024 / 1024
            );
            println!("  Load: {}%", info.load_pct());
            println!("  Temp: {} C", info.temperature() as f32 / 1000.0);
        }
        Err(e) => {
            eprintln!("Failed to detect GPU: {}", e);
        }
    }
}
