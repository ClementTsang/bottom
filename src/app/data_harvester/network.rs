//! Data collection for network usage/IO. This is handled by sysinfo.

pub mod sysinfo;
pub use self::sysinfo::*;

#[derive(Default, Clone, Debug)]
/// Harvested network data. Note that all units in bits.
pub struct NetworkHarvest {
    /// Current incoming bits/s.
    pub rx: u64,

    /// Current outgoing bits/s.
    pub tx: u64,

    /// Total number of incoming bits.
    pub total_rx: u64,

    /// Total number of outgoing bits.
    pub total_tx: u64,
}

impl NetworkHarvest {
    pub fn first_run_cleanup(&mut self) {
        self.rx = 0;
        self.tx = 0;
    }
}

#[cfg(test)]
mod test {
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };

    use super::*;
    use ::sysinfo::{System, SystemExt};

    #[test]
    fn test_getting_network() {
        let sys = System::new_all();
        let prev = Instant::now();

        sleep(Duration::from_secs(2));
        let mut prev_rx = 0;
        let mut prev_tx = 0;

        get_network_data(
            &sys,
            prev,
            &mut prev_rx,
            &mut prev_tx,
            Instant::now(),
            &None,
        )
        .unwrap();
    }
}
