#[cfg(feature = "nvidia")]
pub mod nvidia;

cfg_select! {
    all(target_os = "linux", feature = "gpu") => {
        pub mod intel;
        pub mod amd;
    }
    all(not(target_os = "linux"), feature = "gpu") => {
        pub mod sysinfo_backend;
    }
    _ => {}
}

use std::{
    borrow::Cow,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

use rustc_hash::FxHashSet as HashSet;

/// A GPU's PCI address, used to identify it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PciAddress {
    pub domain: u32,
    pub bus: u32,
    pub device: u32,
    pub function: u32,
}

impl PciAddress {
    /// Parse a PCI address (e.g. `0000:01:00.0` from sysfs or
    /// `00000000:01:00.0` from NVML) in the form `domain:bus:device.function`.
    pub fn parse(address: &str) -> Option<Self> {
        let mut parts = address.trim().split(':');
        let domain = u32::from_str_radix(parts.next()?, 16).ok()?;
        let bus = u32::from_str_radix(parts.next()?, 16).ok()?;
        let (device, function) = parts.next()?.split_once('.')?;

        if parts.next().is_some() {
            return None;
        }

        Some(Self {
            domain,
            bus,
            device: u32::from_str_radix(device, 16).ok()?,
            function: u32::from_str_radix(function, 16).ok()?,
        })
    }
}

impl fmt::Display for PciAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            domain,
            bus,
            device,
            function,
        } = self;

        if *domain == 0 {
            write!(f, "{bus:02x}:{device:02x}.{function:x}")
        } else {
            write!(f, "{domain:04x}:{bus:02x}:{device:02x}.{function:x}")
        }
    }
}

/// Identifies a GPU. Equality is solely done via PCI address (as
/// that's unique), but we also store the display name for convenience.
#[derive(Clone, Debug)]
pub struct GpuKey {
    pub address: PciAddress,
    pub name: String,
}

impl PartialEq for GpuKey {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl Eq for GpuKey {}

impl PartialOrd for GpuKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GpuKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.cmp(&other.address)
    }
}

impl Hash for GpuKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}

impl GpuKey {
    /// The name to display for this GPU. If `with_address` is set, the PCI
    /// address is included, e.g. to tell apart GPUs that share a name.
    pub fn display_name(&self, with_address: bool) -> Cow<'_, str> {
        if with_address {
            Cow::Owned(format!("{} ({})", self.name, self.address))
        } else {
            Cow::Borrowed(&self.name)
        }
    }
}

/// Returns the names that are shared by more than one of the given GPUs.
pub fn duplicate_gpu_names<'a>(keys: impl IntoIterator<Item = &'a GpuKey>) -> HashSet<&'a str> {
    let mut seen = HashSet::default();
    let mut duplicates = HashSet::default();

    for key in keys {
        if !seen.insert(key.name.as_str()) {
            duplicates.insert(key.name.as_str());
        }
    }

    duplicates
}
