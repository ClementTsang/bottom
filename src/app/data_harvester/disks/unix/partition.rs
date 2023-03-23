use super::bindings;

pub(crate) struct Partition {}

#[allow(dead_code)]
fn get_device_name(partition: &Partition) -> String {
    if let Some(device) = partition.device() {
        device
            .to_os_string()
            .into_string()
            .unwrap_or_else(|_| "Name unavailable".to_string())
    } else {
        "Name unavailable".to_string()
    }
}

pub(crate) fn partitions() -> anyhow::Result<Vec<Partition>> {
    let mounts = bindings::mounts()?;

    Ok(())
}
