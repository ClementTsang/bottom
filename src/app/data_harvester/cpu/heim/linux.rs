//! Linux-specific functions regarding CPU usage.

use heim::cpu::os::linux::CpuTimeExt;
pub fn convert_cpu_times(cpu_time: &heim::cpu::CpuTime) -> (f64, f64) {
    let working_time: f64 = (cpu_time.user()
        + cpu_time.nice()
        + cpu_time.system()
        + cpu_time.irq()
        + cpu_time.soft_irq()
        + cpu_time.steal())
    .get::<heim::units::time::second>();
    (
        working_time,
        working_time + (cpu_time.idle() + cpu_time.io_wait()).get::<heim::units::time::second>(),
    )
}
