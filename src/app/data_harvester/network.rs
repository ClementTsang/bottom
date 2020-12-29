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

/// Separate Windows implementation required due to https://github.com/heim-rs/heim/issues/26.
#[cfg(target_os = "windows")]
pub async fn get_network_data(
    sys: &sysinfo::System, prev_net_access_time: Instant, prev_net_rx: &mut u64,
    prev_net_tx: &mut u64, curr_time: Instant, actually_get: bool,
) -> crate::utils::error::Result<Option<NetworkHarvest>> {
    use sysinfo::{NetworkExt, SystemExt};

    if !actually_get {
        return Ok(None);
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
    Ok(Some(NetworkHarvest {
        rx,
        tx,
        total_rx,
        total_tx,
    }))
}

#[cfg(not(target_os = "windows"))]
pub async fn get_network_data(
    prev_net_access_time: Instant, prev_net_rx: &mut u64, prev_net_tx: &mut u64,
    curr_time: Instant, actually_get: bool,
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
    Ok(Some(NetworkHarvest {
        rx,
        tx,
        total_rx,
        total_tx,
    }))
}
