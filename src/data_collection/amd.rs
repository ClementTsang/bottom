use libamdgpu_top::{
    has_vcn, has_vcn_unified, has_vpe,
    stat::{self, FdInfoSortType, FdInfoStat, ProcInfo, Sensors},
    DevicePath,
    PCI::BUS_INFO,
};

use crate::{
    app::{filter::Filter, layout_manager::UsedWidgets},
    data_collection::{
        memory::MemHarvest,
        temperature::{TempHarvest, TemperatureType},
    },
};
use hashbrown::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

pub struct AMDGPUData {
    pub memory: Option<Vec<(String, MemHarvest)>>,
    pub temperature: Option<Vec<TempHarvest>>,
    pub procs: Option<(u64, Vec<HashMap<u32, (u64, u32)>>)>,
}

// needs previous state
static PROC_DATA: LazyLock<Mutex<HashMap<BUS_INFO, FdInfoStat>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[inline]
pub fn get_amd_vecs(
    temp_type: &TemperatureType, filter: &Option<Filter>, widgets_to_harvest: &UsedWidgets,
) -> Option<AMDGPUData> {
    let device_path_list = DevicePath::get_device_path_list();
    let num_gpu = device_path_list.len();
    let mut temp_vec = Vec::with_capacity(num_gpu as usize);
    let mut mem_vec = Vec::with_capacity(num_gpu as usize);
    let mut proc_vec = Vec::with_capacity(num_gpu as usize);
    let mut total_mem = 0;
    let mut proc_map = PROC_DATA.lock().unwrap();

    for device_path in DevicePath::get_device_path_list() {
        if let Ok(amdgpu_dev) = device_path.init() {
            let pci_bus = device_path.pci;
            let Ok(ext_info) = amdgpu_dev.device_info() else {
                continue;
            };
            let name = amdgpu_dev.get_marketing_name_or_default();

            if widgets_to_harvest.use_temp && Filter::optional_should_keep(filter, &name) {
                let sensors = Sensors::new(&amdgpu_dev, &pci_bus, &ext_info);
                if let Some(ref sensors) = sensors {
                    for temp in [
                        &sensors.edge_temp,
                        &sensors.junction_temp,
                        &sensors.memory_temp,
                    ] {
                        let Some(temp) = temp else { continue };
                        let temperature = temp_type.convert_temp_unit(temp.current as f32);
                        temp_vec.push(TempHarvest {
                            name: format!("{} {}", name, temp.type_),
                            temperature: Some(temperature),
                        });
                    }
                }
            }

            if widgets_to_harvest.use_mem {
                if let Ok(memory_info) = amdgpu_dev.memory_info() {
                    mem_vec.push((
                        name.clone(),
                        MemHarvest {
                            total_bytes: memory_info.vram.total_heap_size,
                            used_bytes: memory_info.vram.heap_usage,
                            use_percent: if memory_info.vram.total_heap_size == 0 {
                                None
                            } else {
                                Some(
                                    memory_info.vram.heap_usage as f64
                                        / memory_info.vram.total_heap_size as f64
                                        * 100.0,
                                )
                            },
                        },
                    ));
                }
            }

            if widgets_to_harvest.use_proc {
                let default_fdinfo = FdInfoStat {
                    has_vcn: has_vcn(&amdgpu_dev),
                    has_vcn_unified: has_vcn_unified(&amdgpu_dev),
                    has_vpe: has_vpe(&amdgpu_dev),
                    interval: Duration::from_secs(1),
                    ..Default::default()
                };
                let _ = proc_map.try_insert(pci_bus, default_fdinfo);
                let fdinfo = proc_map.get_mut(&pci_bus).unwrap();

                let mut proc_index: Vec<ProcInfo> = Vec::new();
                stat::update_index(&mut proc_index, &device_path);
                fdinfo.get_all_proc_usage(&proc_index);
                fdinfo.sort_proc_usage(FdInfoSortType::default(), false);

                let mut procs = HashMap::new();

                for pu in fdinfo.proc_usage.clone() {
                    let usage_vram = pu.usage.vram_usage << 10; // KiB -> B
                    let pid: u32 = pu.pid.try_into().unwrap_or(0);
                    let mut gpu_util_wide = pu.usage.gfx;

                    if fdinfo.has_vcn_unified {
                        gpu_util_wide += pu.usage.media;
                    } else if fdinfo.has_vcn {
                        gpu_util_wide += pu.usage.enc + pu.usage.dec;
                    }

                    if fdinfo.has_vpe {
                        gpu_util_wide += pu.usage.vpe;
                    }

                    let gpu_util: u32 = gpu_util_wide.try_into().unwrap_or(0);

                    procs.insert(pid, (usage_vram, gpu_util));
                }

                if !procs.is_empty() {
                    proc_vec.push(procs);
                }

                if let Ok(memory_info) = amdgpu_dev.memory_info() {
                    total_mem += memory_info.vram.total_heap_size
                }
            }
        }
    }

    Some(AMDGPUData {
        memory: if !mem_vec.is_empty() {
            Some(mem_vec)
        } else {
            None
        },
        temperature: if !temp_vec.is_empty() {
            Some(temp_vec)
        } else {
            None
        },
        procs: if !proc_vec.is_empty() {
            Some((total_mem, proc_vec))
        } else {
            None
        },
    })
}
