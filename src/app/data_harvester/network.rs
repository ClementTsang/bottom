use std::time::Instant;

use futures::StreamExt;
use heim::net;
use heim::units::information::byte;
use sysinfo::{NetworkExt, System, SystemExt};

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

pub async fn get_network_data(
    sys: &System, prev_net_access_time: Instant, prev_net_rx: &mut u64, prev_net_tx: &mut u64,
    curr_time: Instant,
) -> NetworkHarvest {
    let mut io_data = net::io_counters();
    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;

    if cfg!(target_os = "windows") {
        let networks = sys.get_networks();
        for (_, network) in networks {
            total_rx += network.get_total_income();
            total_tx += network.get_total_outcome();
        }
    } else {
        while let Some(io) = io_data.next().await {
            if let Ok(io) = io {
                total_rx += io.bytes_recv().get::<byte>();
                total_tx += io.bytes_sent().get::<byte>();
            }
        }
    }

    let elapsed_time = curr_time.duration_since(prev_net_access_time).as_secs_f64();

    let (rx, tx) = if elapsed_time == 0.0 {
        (0, 0)
    } else {
        (
            ((total_rx - *prev_net_rx) as f64 / elapsed_time) as u64,
            ((total_tx - *prev_net_tx) as f64 / elapsed_time) as u64,
        )
    };

    *prev_net_rx = total_rx;
    *prev_net_tx = total_tx;
    NetworkHarvest {
        rx,
        tx,
        total_rx,
        total_tx,
    }
}
