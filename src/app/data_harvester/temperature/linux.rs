//! Gets temperature sensor data for Linux platforms.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use hashbrown::{HashMap, HashSet};

use super::{is_temp_filtered, TempHarvest, TemperatureType};
use crate::app::{
    data_harvester::temperature::{convert_celsius_to_fahrenheit, convert_celsius_to_kelvin},
    Filter,
};

#[derive(Default)]
struct HwmonResults {
    temperatures: Vec<TempHarvest>,
}

/// Parses and reads temperatures that were in millidegree Celsius, and if successful, returns a temperature in Celsius.
fn read_temp(path: &Path) -> Result<f32> {
    Ok(fs::read_to_string(path)?
        .trim_end()
        .parse::<f32>()
        .map_err(|e| crate::utils::error::BottomError::ConversionError(e.to_string()))?
        / 1_000.0)
}

fn convert_temp_unit(temp: f32, temp_type: &TemperatureType) -> f32 {
    match temp_type {
        TemperatureType::Celsius => temp,
        TemperatureType::Kelvin => convert_celsius_to_kelvin(temp),
        TemperatureType::Fahrenheit => convert_celsius_to_fahrenheit(temp),
    }
}

/// Get temperature sensors from the linux sysfs interface `/sys/class/hwmon` and
/// `/sys/devices/platform/coretemp.*`. It returns all found temperature sensors, and the number
/// of checked hwmon directories (not coretemp directories).
///
/// For more details, see the relevant Linux kernel documentation:
/// - [`/sys/class/hwmon`](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-hwmon)
/// - [`/sys/devices/platform/coretemp.*`](https://www.kernel.org/doc/html/v5.14/hwmon/coretemp.html)
///
/// This method will return `0` as the temperature for devices, such as GPUs,
/// that support power management features that have powered themselves off.
/// Specifically, in laptops with iGPUs and dGPUs, if the dGPU is capable of
/// entering ACPI D3cold, reading the temperature sensors will wake it,
/// and keep it awake, wasting power.
///
/// For such devices, this method will only query the sensors *only* if
/// the device is already in ACPI D0. This has the notable issue that
/// once this happens, the device will be *kept* on through the sensor
/// reading, and not be able to re-enter ACPI D3cold.
fn get_from_hwmon(temp_type: &TemperatureType, filter: &Option<Filter>) -> Result<HwmonResults> {
    let mut temperatures: Vec<TempHarvest> = vec![];

    fn add_hwmon(dirs: &mut HashSet<PathBuf>) {
        if let Ok(read_dir) = Path::new("/sys/class/hwmon").read_dir() {
            for entry in read_dir.flatten() {
                let mut path = entry.path();

                // hwmon includes many sensors, we only want ones with at least one temperature sensor
                // Reading this file will wake the device, but we're only checking existence, so it should be fine.
                if !path.join("temp1_input").exists() {
                    // Note we also check for a `device` subdirectory (e.g. `/sys/class/hwmon/hwmon*/device/`).
                    // This is needed for CentOS, which adds this extra `/device` directory. See:
                    // - https://github.com/nicolargo/glances/issues/1060
                    // - https://github.com/giampaolo/psutil/issues/971
                    // - https://github.com/giampaolo/psutil/blob/642438375e685403b4cd60b0c0e25b80dd5a813d/psutil/_pslinux.py#L1316
                    //
                    // If it does match, then add the `device/` directory to the path.
                    if path.join("device/temp1_input").exists() {
                        path.push("device");
                    }
                }

                dirs.insert(path);
            }
        }
    }

    fn add_coretemp(dirs: &mut HashSet<PathBuf>) {
        if let Ok(read_dir) = Path::new("/sys/devices/platform").read_dir() {
            for entry in read_dir.flatten() {
                if entry.file_name().to_string_lossy().starts_with("coretemp.") {
                    if let Ok(read_dir) = entry.path().join("hwmon").read_dir() {
                        for entry in read_dir.flatten() {
                            let path = entry.path();

                            if path.join("temp1_input").exists() {
                                // It's possible that there are dupes (represented by symlinks) - the easy
                                // way is to just substitute the parent directory and check if the hwmon
                                // variant exists already in a set.
                                if let Some(child) = path.file_name() {
                                    let to_check_path = Path::new("/sys/class/hwmon").join(child);

                                    if !dirs.contains(&to_check_path) {
                                        dirs.insert(path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut dirs = HashSet::default();
    add_hwmon(&mut dirs);
    add_coretemp(&mut dirs);

    // Note that none of this is async if we ever go back to it, but sysfs is in
    // memory, so in theory none of this should block if we're slightly careful.
    // Of note is that reading the temperature sensors of a device that has
    // `/sys/class/hwmon/hwmon*/device/power_state` == `D3cold` will
    // wake the device up, and will block until it initializes.
    //
    // Reading the `hwmon*/device/power_state` or `hwmon*/temp*_label` properties
    // will not wake the device, and thus not block,
    // and meaning no sensors have to be hidden depending on `power_state`
    //
    // It would probably be more ideal to use a proper async runtime; this would also allow easy cancellation/timeouts.
    for file_path in dirs {
        let hwmon_name = fs::read_to_string(file_path.join("name")).ok();

        // Whether the temperature should *actually* be read during enumeration
        // Set to false if the device is in ACPI D3cold.
        let should_read_temp = {
            // Documented at https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-power_state
            let device = file_path.join("device");
            let power_state = device.join("power_state");
            if power_state.exists() {
                let state = fs::read_to_string(power_state)?;
                let state = state.trim();
                // The zenpower3 kernel module (incorrectly?) reports "unknown", causing this check
                // to fail and temperatures to appear as zero instead of having the file not exist.
                //
                // Their self-hosted git instance has disabled sign up, so this bug cant be reported either.
                state == "D0" || state == "unknown"
            } else {
                true
            }
        };

        // Enumerate the devices temperature sensors
        for entry in file_path.read_dir()? {
            let file = entry?;
            let name = file.file_name();
            let name = name
                .to_str()
                .ok_or_else(|| anyhow!("temperature device filenames should be ASCII"))?;

            // We only want temperature sensors, skip others early
            if !(name.starts_with("temp") && name.ends_with("input")) {
                continue;
            }

            let temp_path = file.path();
            let temp_label = file_path.join(name.replace("input", "label"));
            let temp_label = fs::read_to_string(temp_label).ok();

            // Do some messing around to get a more sensible name for sensors:
            // - For GPUs, this will use the kernel device name, ex `card0`
            // - For nvme drives, this will also use the kernel name, ex `nvme0`.
            //   This is found differently than for GPUs
            // - For whatever acpitz is, on my machine this is now `thermal_zone0`.
            // - For k10temp, this will still be k10temp, but it has to be handled special.
            let human_hwmon_name = {
                let device = file_path.join("device");
                // This will exist for GPUs but not others, this is how we find their kernel name.
                let drm = device.join("drm");
                if drm.exists() {
                    // This should never actually be empty
                    let mut gpu = None;
                    for card in drm.read_dir()? {
                        let card = card?;
                        let name = card.file_name().to_str().unwrap_or_default().to_owned();
                        if name.starts_with("card") {
                            if let Some(hwmon_name) = hwmon_name.as_ref() {
                                gpu = Some(format!("{} ({})", name, hwmon_name.trim()));
                            } else {
                                gpu = Some(name)
                            }
                            break;
                        }
                    }
                    gpu
                } else {
                    // This little mess is to account for stuff like k10temp. This is needed because the
                    // `device` symlink points to `nvme*` for nvme drives, but to PCI buses for anything
                    // else. If the first character is alphabetic, it's an actual name like k10temp or
                    // nvme0, not a PCI bus.
                    let link = fs::read_link(device)?
                        .file_name()
                        .map(|f| f.to_str().unwrap_or_default().to_owned())
                        .unwrap();
                    if link.as_bytes()[0].is_ascii_alphabetic() {
                        if let Some(hwmon_name) = hwmon_name.as_ref() {
                            Some(format!("{} ({})", link, hwmon_name.trim()))
                        } else {
                            Some(link)
                        }
                    } else {
                        hwmon_name.clone()
                    }
                }
            };

            let name = match (&human_hwmon_name, &temp_label) {
                (Some(name), Some(label)) => format!("{}: {}", name.trim(), label.trim()),
                (None, Some(label)) => label.to_string(),
                (Some(name), None) => name.to_string(),
                (None, None) => String::default(),
            };

            if is_temp_filtered(filter, &name) {
                let temp = if should_read_temp {
                    if let Ok(temp) = read_temp(&temp_path) {
                        temp
                    } else {
                        continue;
                    }
                } else {
                    0.0
                };

                temperatures.push(TempHarvest {
                    name,
                    temperature: convert_temp_unit(temp, temp_type),
                });
            }
        }
    }

    Ok(HwmonResults { temperatures })
}

/// Gets data from `/sys/class/thermal/thermal_zone*`. This should only be used if
/// [`get_from_hwmon`] doesn't return anything.
///
/// See [the Linux kernel documentation](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-thermal)
/// for more details.
fn get_from_thermal_zone(
    temperatures: &mut Vec<TempHarvest>, temp_type: &TemperatureType, filter: &Option<Filter>,
) {
    let path = Path::new("/sys/class/thermal");
    let Ok(read_dir) = path.read_dir() else {
        return
    };

    let mut seen_names: HashMap<String, u32> = HashMap::new();

    for entry in read_dir.flatten() {
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with("thermal_zone")
        {
            let file_path = entry.path();
            let name_path = file_path.join("type");

            if let Ok(name) = fs::read_to_string(name_path) {
                let name = name.trim_end();

                if is_temp_filtered(filter, name) {
                    let temp_path = file_path.join("temp");
                    if let Ok(temp) = read_temp(&temp_path) {
                        let name = if let Some(count) = seen_names.get_mut(name) {
                            *count += 1;
                            format!("{name} ({})", *count)
                        } else {
                            seen_names.insert(name.to_string(), 0);
                            name.to_string()
                        };

                        temperatures.push(TempHarvest {
                            name,
                            temperature: convert_temp_unit(temp, temp_type),
                        });
                    }
                }
            }
        }
    }
}

/// Gets temperature sensors and data.
pub fn get_temperature_data(
    temp_type: &TemperatureType, filter: &Option<Filter>,
) -> Result<Option<Vec<TempHarvest>>> {
    let mut results = get_from_hwmon(temp_type, filter).unwrap_or_default();

    get_from_thermal_zone(&mut results.temperatures, temp_type, filter);

    #[cfg(feature = "nvidia")]
    {
        super::nvidia::add_nvidia_data(&mut results.temperatures, temp_type, filter)?;
    }

    Ok(Some(results.temperatures))
}
