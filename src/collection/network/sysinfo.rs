//! Gets network data via sysinfo.

use std::time::Instant;

use sysinfo::Networks;

use super::NetworkHarvest;
use crate::app::filter::Filter;

// TODO: Eventually make it so that this thing also takes individual usage into
// account, so we can show per-interface!
pub fn get_network_data(
    networks: &Networks, prev_net_access_time: Instant, prev_net_rx: &mut u64,
    prev_net_tx: &mut u64, curr_time: Instant, filter: &Option<Filter>,
) -> NetworkHarvest {
    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;

    for (name, network) in networks {
        let to_keep = if let Some(filter) = filter {
            filter.should_keep(name)
        } else {
            true
        };

        if to_keep {
            total_rx += network.total_received() * 8;
            total_tx += network.total_transmitted() * 8;
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
    NetworkHarvest {
        rx,
        tx,
        total_rx,
        total_tx,
    }
}
