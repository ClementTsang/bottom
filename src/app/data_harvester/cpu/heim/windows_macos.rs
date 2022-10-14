//! Windows and macOS-specific functions regarding CPU usage.

use crate::components::tui_widget::time_chart::Point;

pub fn convert_cpu_times(cpu_time: &heim::cpu::CpuTime) -> Point {
    let working_time: f64 =
        (cpu_time.user() + cpu_time.system()).get::<heim::units::time::second>();
    (
        working_time,
        working_time + cpu_time.idle().get::<heim::units::time::second>(),
    )
}
