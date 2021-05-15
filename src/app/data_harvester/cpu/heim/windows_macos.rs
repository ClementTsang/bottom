//! Windows and macOS-specific functions regarding CPU usage.

pub fn convert_cpu_times(cpu_time: &heim::cpu::CpuTime) -> (f64, f64) {
    let working_time: f64 =
        (cpu_time.user() + cpu_time.system()).get::<heim::units::time::second>();
    (
        working_time,
        working_time + cpu_time.idle().get::<heim::units::time::second>(),
    )
}
