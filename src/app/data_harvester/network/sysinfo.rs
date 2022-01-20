//! Gets network data via sysinfo.

use super::NetworkHarvest;
use std::time::Instant;

pub async fn get_network_data(
    sys: &sysinfo::System, prev_net_access_time: Instant, prev_net_rx: &mut u64,
    prev_net_tx: &mut u64, curr_time: Instant, actually_get: bool,
    filter: &Option<crate::app::Filter>,
) -> crate::utils::error::Result<Option<NetworkHarvest>> {
    use sysinfo::{NetworkExt, SystemExt};

    if !actually_get {
        return Ok(None);
    }

    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;

    let networks = sys.networks();
    for (name, network) in networks {
        let to_keep = if let Some(filter) = filter {
            let mut ret = filter.is_list_ignored;
            for r in &filter.list {
                if r.is_match(name) {
                    ret = !filter.is_list_ignored;
                    break;
                }
            }
            ret
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
    Ok(Some(NetworkHarvest {
        rx,
        tx,
        total_rx,
        total_tx,
    }))
}
