use std::time::Instant;

#[derive(Default, Clone, Debug)]
pub struct NetworkHarvest {
    pub rx: u64,
    pub tx: u64,
    pub total_rx: u64,
    pub total_tx: u64,
}

impl NetworkHarvest {
    pub fn first_run_cleanup(&mut self) {
        self.rx = 0;
        self.tx = 0;
    }
}

/// Meant for Windows and ARM use.
#[cfg(any(target_os = "windows", target_arch = "aarch64", target_arch = "arm"))]
pub async fn get_network_data(
    sys: &sysinfo::System, prev_net_access_time: Instant, prev_net_rx: &mut u64,
    prev_net_tx: &mut u64, curr_time: Instant, actually_get: bool,
) -> Option<NetworkHarvest> {
    use sysinfo::{NetworkExt, SystemExt};

    if !actually_get {
        return None;
    }

    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;

    let networks = sys.get_networks();
    for (_, network) in networks {
        total_rx += network.get_total_received();
        total_tx += network.get_total_transmitted();
    }

    let elapsed_time = curr_time.duration_since(prev_net_access_time).as_secs_f64();

    let (rx, tx) = if elapsed_time == 0.0 {
        (0, 0)
    } else {
        (
            ((total_rx.saturating_sub(*prev_net_rx)) as f64 / elapsed_time) as u64,
            ((total_tx.saturating_sub(*prev_net_tx)) as f64 / elapsed_time) as u64,
        )
    };

    *prev_net_rx = total_rx;
    *prev_net_tx = total_tx;
    Some(NetworkHarvest {
        rx,
        tx,
        total_rx,
        total_tx,
    })
}

#[cfg(not(any(target_os = "windows", target_arch = "aarch64", target_arch = "arm")))]
pub async fn get_network_data(
    prev_net_access_time: Instant, prev_net_rx: &mut u64, prev_net_tx: &mut u64,
    curr_time: Instant, actually_get: bool,
) -> Option<NetworkHarvest> {
    use futures::StreamExt;

    if !actually_get {
        return None;
    }

    let mut io_data = heim::net::io_counters();
    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;

    while let Some(io) = io_data.next().await {
        if let Ok(io) = io {
            total_rx += io.bytes_recv().get::<heim::units::information::byte>();
            total_tx += io.bytes_sent().get::<heim::units::information::byte>();
        }
    }

    let elapsed_time = curr_time.duration_since(prev_net_access_time).as_secs_f64();

    let (rx, tx) = if elapsed_time == 0.0 {
        (0, 0)
    } else {
        (
            ((total_rx.saturating_sub(*prev_net_rx)) as f64 / elapsed_time) as u64,
            ((total_tx.saturating_sub(*prev_net_tx)) as f64 / elapsed_time) as u64,
        )
    };

    *prev_net_rx = total_rx;
    *prev_net_tx = total_tx;
    Some(NetworkHarvest {
        rx,
        tx,
        total_rx,
        total_tx,
    })
}
