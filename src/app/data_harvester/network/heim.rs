//! Gets network data via heim.

use super::NetworkHarvest;
use std::time::Instant;

// FIXME: Eventually make it so that this thing also takes individual usage into account, so we can allow for showing per-interface!
pub async fn get_network_data(
    prev_net_access_time: Instant, prev_net_rx: &mut u64, prev_net_tx: &mut u64,
    curr_time: Instant, actually_get: bool, filter: &Option<crate::app::Filter>,
) -> crate::utils::error::Result<Option<NetworkHarvest>> {
    use futures::StreamExt;

    if !actually_get {
        return Ok(None);
    }

    let io_data = heim::net::io_counters().await?;
    futures::pin_mut!(io_data);
    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;

    while let Some(io) = io_data.next().await {
        if let Ok(io) = io {
            let to_keep = if let Some(filter) = filter {
                if filter.is_list_ignored {
                    let mut ret = true;
                    for r in &filter.list {
                        if r.is_match(io.interface()) {
                            ret = false;
                            break;
                        }
                    }
                    ret
                } else {
                    true
                }
            } else {
                true
            };

            if to_keep {
                // TODO: Use bytes as the default instead, perhaps?
                // Since you might have to do a double conversion (bytes -> bits -> bytes) in some cases;
                // but if you stick to bytes, then in the bytes, case, you do no conversion, and in the bits case,
                // you only do one conversion...
                total_rx += io.bytes_recv().get::<heim::units::information::bit>();
                total_tx += io.bytes_sent().get::<heim::units::information::bit>();
            }
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
