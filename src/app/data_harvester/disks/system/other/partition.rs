use super::bindings;

pub(crate) struct Partition {}

impl Partition {
    fn get_device_name(&self) -> String {
        if let Some(device) = self.device() {
            device
                .to_os_string()
                .into_string()
                .unwrap_or_else(|_| "Name unavailable".to_string())
        } else {
            "Name unavailable".to_string()
        }
    }
}

pub(crate) fn partitions() -> anyhow::Result<Vec<Partition>> {
    let mounts = bindings::mounts()?;

    Ok(())
}
