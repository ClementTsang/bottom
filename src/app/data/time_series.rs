//! Time series data.

use std::{
    cmp::Ordering,
    time::{Duration, Instant},
    vec::Vec,
};

#[cfg(feature = "gpu")]
use hashbrown::{HashMap, HashSet}; // TODO: Try fxhash again.
use timeless::data::ChunkedData;

use crate::collection::Data;

/// Values corresponding to a time slice.
pub type Values = ChunkedData<f64>;

/// Represents time series data in a chunked, deduped manner.
///
/// Properties:
/// - Time in this manner is represented in a reverse-offset fashion from the current time.
/// - All data is stored in SoA fashion.
/// - Values are stored in a chunked format, which facilitates gaps in data collection if needed.
/// - Additional metadata is stored to make data pruning over time easy.
#[derive(Clone, Debug, Default)]
pub struct TimeSeriesData {
    /// Time values.
    ///
    /// TODO: (points_rework_v1) Either store millisecond-level only or offsets only.
    pub time: Vec<Instant>,

    /// Network RX data.
    pub rx: Values,

    /// Network TX data.
    pub tx: Values,

    /// CPU data.
    pub cpu: Vec<Values>,

    /// Average CPU data.
    pub avg_cpu: Values,

    /// RAM memory data.
    pub ram: Values,

    /// Swap data.
    pub swap: Values,

    #[cfg(not(target_os = "windows"))]
    /// Cache data.
    pub cache_mem: Values,

    #[cfg(feature = "zfs")]
    /// Arc data.
    pub arc_mem: Values,

    #[cfg(feature = "gpu")]
    /// GPU memory data.
    pub gpu_mem: HashMap<String, Values>,
}

impl TimeSeriesData {
    /// Add a new data point.
    pub fn add(&mut self, data: &Data) {
        self.time.push(data.collection_time);

        if let Some(network) = &data.network {
            self.rx.push(network.rx as f64);
            self.tx.push(network.tx as f64);
        } else {
            self.rx.insert_break();
            self.tx.insert_break();
        }

        if let Some(cpu_harvest) = &data.cpu {
            let cpus = &cpu_harvest.cpus;

            match self.cpu.len().cmp(&cpus.len()) {
                Ordering::Less => {
                    let diff = cpus.len() - self.cpu.len();
                    self.cpu.reserve_exact(diff);

                    for _ in 0..diff {
                        self.cpu.push(Default::default());
                    }
                }
                Ordering::Greater => {
                    let diff = self.cpu.len() - cpus.len();
                    let offset = self.cpu.len() - diff;

                    for curr in &mut self.cpu[offset..] {
                        curr.insert_break();
                    }
                }
                Ordering::Equal => {}
            }

            for (curr, new_data) in self.cpu.iter_mut().zip(cpus.iter()) {
                curr.push((*new_data).into());
            }

            // If there isn't avg then we never had any to begin with.
            if let Some(avg) = cpu_harvest.avg {
                self.avg_cpu.push(avg.into());
            }
        } else {
            for c in &mut self.cpu {
                c.insert_break();
            }

            self.avg_cpu.insert_break();
        }

        if let Some(memory) = &data.memory {
            self.ram.push(memory.percentage());
        } else {
            self.ram.insert_break();
        }

        if let Some(swap) = &data.swap {
            self.swap.push(swap.percentage());
        } else {
            self.swap.insert_break();
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Some(cache) = &data.cache {
                self.cache_mem.push(cache.percentage());
            } else {
                self.cache_mem.insert_break();
            }
        }

        #[cfg(feature = "zfs")]
        {
            if let Some(arc) = &data.arc {
                self.arc_mem.push(arc.percentage());
            } else {
                self.arc_mem.insert_break();
            }
        }

        #[cfg(feature = "gpu")]
        {
            if let Some(gpu) = &data.gpu {
                let mut not_visited = self
                    .gpu_mem
                    .keys()
                    .map(String::to_owned)
                    .collect::<HashSet<_>>();

                for (name, new_data) in gpu {
                    not_visited.remove(name);

                    if !self.gpu_mem.contains_key(name) {
                        self.gpu_mem
                            .insert(name.to_string(), ChunkedData::default());
                    }

                    let curr = self
                        .gpu_mem
                        .get_mut(name)
                        .expect("entry must exist as it was created above");
                    curr.push(new_data.percentage());
                }

                for nv in not_visited {
                    if let Some(entry) = self.gpu_mem.get_mut(&nv) {
                        entry.insert_break();
                    }
                }
            } else {
                for g in self.gpu_mem.values_mut() {
                    g.insert_break();
                }
            }
        }
    }

    /// Prune any data older than the given duration.
    pub fn prune(&mut self, max_age: Duration) {
        if self.time.is_empty() {
            return;
        }

        let now = Instant::now();
        let end = {
            let partition_point = self
                .time
                .partition_point(|then| now.duration_since(*then) > max_age);

            // Partition point returns the first index that does not match the predicate, so minus one.
            if partition_point > 0 {
                partition_point - 1
            } else {
                // If the partition point was 0, then it means all values are too new to be pruned.
                // crate::info!("Skipping prune.");
                return;
            }
        };

        // crate::info!("Pruning up to index {end}.");

        // Note that end here is _inclusive_.
        self.time.drain(0..=end);
        self.time.shrink_to_fit();

        let _ = self.rx.prune_and_shrink_to_fit(end);
        let _ = self.tx.prune_and_shrink_to_fit(end);

        for cpu in &mut self.cpu {
            let _ = cpu.prune_and_shrink_to_fit(end);
        }

        let _ = self.ram.prune_and_shrink_to_fit(end);
        let _ = self.swap.prune_and_shrink_to_fit(end);

        #[cfg(not(target_os = "windows"))]
        let _ = self.cache_mem.prune_and_shrink_to_fit(end);

        #[cfg(feature = "zfs")]
        let _ = self.arc_mem.prune_and_shrink_to_fit(end);

        #[cfg(feature = "gpu")]
        {
            self.gpu_mem.retain(|_, gpu| {
                let _ = gpu.prune(end);

                // Remove the entry if it is empty. We can always add it again later.
                if gpu.no_elements() {
                    false
                } else {
                    gpu.shrink_to_fit();
                    true
                }
            });
        }
    }
}
