//! Gets temperature sensor data for Linux platforms.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use hashbrown::{HashMap, HashSet};

use super::{TempHarvest, TemperatureType};
use crate::app::filter::Filter;

const EMPTY_NAME: &str = "Unknown";

/// Returned results from grabbing hwmon/coretemp temperature sensor
/// values/names.
struct HwmonResults {
    temperatures: Vec<TempHarvest>,
    num_hwmon: usize,
}

/// Parses and reads temperatures that were in millidegree Celsius, and if
/// successful, returns a temperature in Celsius.
fn parse_temp(path: &Path) -> Result<f32> {
    Ok(fs::read_to_string(path)?.trim_end().parse::<f32>()? / 1_000.0)
}

/// Get all candidates from hwmon and coretemp. It will also return the number
/// of entries from hwmon.
fn get_hwmon_candidates() -> (HashSet<PathBuf>, usize) {
    let mut dirs = HashSet::default();

    if let Ok(read_dir) = Path::new("/sys/class/hwmon").read_dir() {
        for entry in read_dir.flatten() {
            let mut path = entry.path();

            // hwmon includes many sensors, we only want ones with at least one temperature
            // sensor Reading this file will wake the device, but we're only
            // checking existence, so it should be fine.
            if !path.join("temp1_input").exists() {
                // Note we also check for a `device` subdirectory (e.g.
                // `/sys/class/hwmon/hwmon*/device/`). This is needed for
                // CentOS, which adds this extra `/device` directory. See:
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

    let num_hwmon = dirs.len();

    if let Ok(read_dir) = Path::new("/sys/devices/platform").read_dir() {
        for entry in read_dir.flatten() {
            if entry.file_name().to_string_lossy().starts_with("coretemp.") {
                if let Ok(read_dir) = entry.path().join("hwmon").read_dir() {
                    for entry in read_dir.flatten() {
                        let path = entry.path();

                        if path.join("temp1_input").exists() {
                            // It's possible that there are dupes (represented by symlinks) - the
                            // easy way is to just substitute the parent
                            // directory and check if the hwmon
                            // variant exists already in a set.
                            //
                            // For more info, see https://github.com/giampaolo/psutil/pull/1822/files
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

    (dirs, num_hwmon)
}

#[inline]
fn read_to_string_lossy<P: AsRef<Path>>(path: P) -> Option<String> {
    fs::read(path)
        .map(|v| String::from_utf8_lossy(&v).trim().to_string())
        .ok()
}

#[inline]
fn humanize_name(name: String, sensor_name: Option<&String>) -> String {
    match sensor_name {
        Some(ty) => format!("{name} ({ty})"),
        None => name,
    }
}

#[inline]
fn counted_name(seen_names: &mut HashMap<String, u32>, name: String) -> String {
    if let Some(count) = seen_names.get_mut(&name) {
        *count += 1;
        format!("{name} ({count})")
    } else {
        seen_names.insert(name.clone(), 0);
        name
    }
}

#[inline]
fn finalize_name(
    hwmon_name: Option<String>, sensor_label: Option<String>,
    fallback_sensor_name: &Option<String>, seen_names: &mut HashMap<String, u32>,
) -> String {
    let candidate_name = match (hwmon_name, sensor_label) {
        (Some(name), Some(label)) => match (name.is_empty(), label.is_empty()) {
            (false, false) => {
                format!("{name}: {label}")
            }
            (true, false) => match fallback_sensor_name {
                Some(fallback) if !fallback.is_empty() => {
                    if label.is_empty() {
                        fallback.to_owned()
                    } else {
                        format!("{fallback}: {label}")
                    }
                }
                _ => {
                    if label.is_empty() {
                        EMPTY_NAME.to_string()
                    } else {
                        label
                    }
                }
            },
            (false, true) => name.to_owned(),
            (true, true) => EMPTY_NAME.to_string(),
        },
        (None, Some(label)) => match fallback_sensor_name {
            Some(fallback) if !fallback.is_empty() => {
                if label.is_empty() {
                    fallback.to_owned()
                } else {
                    format!("{fallback}: {label}")
                }
            }
            _ => {
                if label.is_empty() {
                    EMPTY_NAME.to_string()
                } else {
                    label
                }
            }
        },
        (Some(name), None) => {
            if name.is_empty() {
                EMPTY_NAME.to_string()
            } else {
                name
            }
        }
        (None, None) => match fallback_sensor_name {
            Some(sensor_name) if !sensor_name.is_empty() => sensor_name.to_owned(),
            _ => EMPTY_NAME.to_string(),
        },
    };

    counted_name(seen_names, candidate_name)
}

/// Whether the temperature should *actually* be read during enumeration.
/// Will return false if the state is not D0/unknown, or if it does not support
/// `device/power_state`.
#[inline]
fn is_device_awake(path: &Path) -> bool {
    // Whether the temperature should *actually* be read during enumeration.
    // Set to false if the device is in ACPI D3cold.
    // Documented at https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-power_state
    let device = path.join("device");
    let power_state = device.join("power_state");
    if power_state.exists() {
        if let Ok(state) = fs::read_to_string(power_state) {
            let state = state.trim();
            // The zenpower3 kernel module (incorrectly?) reports "unknown", causing this
            // check to fail and temperatures to appear as zero instead of
            // having the file not exist.
            //
            // Their self-hosted git instance has disabled sign up, so this bug cant be
            // reported either.
            state == "D0" || state == "unknown"
        } else {
            true
        }
    } else {
        true
    }
}

/// Get temperature sensors from the linux sysfs interface `/sys/class/hwmon`
/// and `/sys/devices/platform/coretemp.*`. It returns all found temperature
/// sensors, and the number of checked hwmon directories (not coretemp
/// directories).
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
fn hwmon_temperatures(temp_type: &TemperatureType, filter: &Option<Filter>) -> HwmonResults {
    let mut temperatures: Vec<TempHarvest> = vec![];
    let mut seen_names: HashMap<String, u32> = HashMap::new();

    let (dirs, num_hwmon) = get_hwmon_candidates();

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
    // It would probably be more ideal to use a proper async runtime; this would
    // also allow easy cancellation/timeouts.
    for file_path in dirs {
        let sensor_name = read_to_string_lossy(file_path.join("name"));

        if !is_device_awake(&file_path) {
            let name = finalize_name(None, None, &sensor_name, &mut seen_names);
            temperatures.push(TempHarvest {
                name,
                temperature: None,
            });

            continue;
        }

        if let Ok(dir_entries) = file_path.read_dir() {
            // Enumerate the devices temperature sensors
            for file in dir_entries.flatten() {
                let name = file.file_name();
                let name = name.to_string_lossy();

                // We only want temperature sensors, skip others early
                if !(name.starts_with("temp") && name.ends_with("input")) {
                    continue;
                }

                let temp_path = file.path();
                let sensor_label_path = file_path.join(name.replace("input", "label"));
                let sensor_label = read_to_string_lossy(sensor_label_path);

                // Do some messing around to get a more sensible name for sensors:
                // - For GPUs, this will use the kernel device name, ex `card0`
                // - For nvme drives, this will also use the kernel name, ex `nvme0`. This is
                //   found differently than for GPUs
                // - For whatever acpitz is, on my machine this is now `thermal_zone0`.
                // - For k10temp, this will still be k10temp, but it has to be handled special.
                let hwmon_name = {
                    let device = file_path.join("device");

                    // This will exist for GPUs but not others, this is how we find their kernel
                    // name.
                    let drm = device.join("drm");
                    if drm.exists() {
                        // This should never actually be empty. If it is though, we'll fall back to
                        // the sensor name later on.
                        let mut gpu = None;

                        if let Ok(cards) = drm.read_dir() {
                            for card in cards.flatten() {
                                if let Some(name) = card.file_name().to_str() {
                                    if name.starts_with("card") {
                                        gpu = Some(humanize_name(
                                            name.trim().to_string(),
                                            sensor_name.as_ref(),
                                        ));
                                        break;
                                    }
                                }
                            }
                        }

                        gpu
                    } else {
                        // This little mess is to account for stuff like k10temp. This is needed
                        // because the `device` symlink points to `nvme*`
                        // for nvme drives, but to PCI buses for anything
                        // else. If the first character is alphabetic, it's an actual name like
                        // k10temp or nvme0, not a PCI bus.
                        fs::read_link(device).ok().and_then(|link| {
                            let link = link
                                .file_name()
                                .and_then(|f| f.to_str())
                                .map(|s| s.trim().to_owned());

                            match link {
                                Some(link) if link.as_bytes()[0].is_ascii_alphabetic() => {
                                    Some(humanize_name(link, sensor_name.as_ref()))
                                }
                                _ => None,
                            }
                        })
                    }
                };

                let name = finalize_name(hwmon_name, sensor_label, &sensor_name, &mut seen_names);

                // TODO: It's possible we may want to move the filter check further up to avoid
                // probing hwmon if not needed?
                if Filter::optional_should_keep(filter, &name) {
                    if let Ok(temp_celsius) = parse_temp(&temp_path) {
                        temperatures.push(TempHarvest {
                            name,
                            temperature: Some(temp_type.convert_temp_unit(temp_celsius)),
                        });
                    }
                }
            }
        }
    }

    HwmonResults {
        temperatures,
        num_hwmon,
    }
}

/// Gets data from `/sys/class/thermal/thermal_zone*`. This should only be used
/// if [`hwmon_temperatures`] doesn't return anything to avoid duplicate sensor
/// results.
///
/// See [the Linux kernel documentation](https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-thermal)
/// for more details.
fn add_thermal_zone_temperatures(
    temperatures: &mut Vec<TempHarvest>, temp_type: &TemperatureType, filter: &Option<Filter>,
) {
    let path = Path::new("/sys/class/thermal");
    let Ok(read_dir) = path.read_dir() else {
        return;
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

            if let Some(name) = read_to_string_lossy(name_path) {
                let name = if name.is_empty() {
                    EMPTY_NAME.to_string()
                } else {
                    name
                };

                if Filter::optional_should_keep(filter, &name) {
                    let temp_path = file_path.join("temp");
                    if let Ok(temp_celsius) = parse_temp(&temp_path) {
                        let name = counted_name(&mut seen_names, name);

                        temperatures.push(TempHarvest {
                            name,
                            temperature: Some(temp_type.convert_temp_unit(temp_celsius)),
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
    let mut results = hwmon_temperatures(temp_type, filter);

    if results.num_hwmon == 0 {
        add_thermal_zone_temperatures(&mut results.temperatures, temp_type, filter);
    }

    Ok(Some(results.temperatures))
}

#[cfg(test)]
mod tests {
    use hashbrown::HashMap;

    use super::finalize_name;

    #[test]
    fn test_finalize_name() {
        let mut seen_names = HashMap::new();

        assert_eq!(
            finalize_name(
                Some("hwmon".to_string()),
                Some("sensor".to_string()),
                &Some("test".to_string()),
                &mut seen_names
            ),
            "hwmon: sensor"
        );

        assert_eq!(
            finalize_name(
                Some("hwmon".to_string()),
                None,
                &Some("test".to_string()),
                &mut seen_names
            ),
            "hwmon"
        );

        assert_eq!(
            finalize_name(
                None,
                Some("sensor".to_string()),
                &Some("test".to_string()),
                &mut seen_names
            ),
            "test: sensor"
        );

        assert_eq!(
            finalize_name(
                Some("hwmon".to_string()),
                Some("sensor".to_string()),
                &Some("test".to_string()),
                &mut seen_names
            ),
            "hwmon: sensor (1)"
        );

        assert_eq!(
            finalize_name(None, None, &Some("test".to_string()), &mut seen_names),
            "test"
        );

        assert_eq!(finalize_name(None, None, &None, &mut seen_names), "Unknown");

        assert_eq!(
            finalize_name(None, None, &Some("test".to_string()), &mut seen_names),
            "test (1)"
        );

        assert_eq!(
            finalize_name(None, None, &None, &mut seen_names),
            "Unknown (1)"
        );

        assert_eq!(
            finalize_name(Some(String::default()), None, &None, &mut seen_names),
            "Unknown (2)"
        );

        assert_eq!(
            finalize_name(None, Some(String::default()), &None, &mut seen_names),
            "Unknown (3)"
        );

        assert_eq!(
            finalize_name(None, None, &Some(String::default()), &mut seen_names),
            "Unknown (4)"
        );
    }
}
