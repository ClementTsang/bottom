//! AMD GPU temperature collection for Windows via ADL (AMD Display Library).
//!
//! Windows exposes no documented, vendor-neutral user-mode API for GPU
//! temperatures (see issue #1460), so the generic [`sysinfo`] temperature
//! path only ever yields the ACPI thermal zone. Task Manager reads the same
//! sensors through WDDM, which has no stable public query surface for
//! third-party monitors.
//!
//! The AMD display driver already ships everything we need: `atiadlxx.dll`
//! (64-bit) exposes the ADL2 API also used by LibreHardwareMonitor and
//! fastfetch. The DLL is loaded dynamically and only the temperature
//! symbols are resolved, so:
//!
//! - systems without an AMD GPU/driver simply report no AMD temperatures,
//! - no new dependency or driver install is required,
//! - no administrator privileges are required.
//!
//! Temperature is read edge-first via `ADL2_OverdriveN_Temperature_Get`
//! (millidegrees Celsius), then `ADL2_Overdrive6_Temperature_Get`, then the
//! PMLog query (`ADL2_New_QueryPMLogData_Get`, degrees Celsius) that newer
//! ASIC families — including Ryzen iGPUs, which answer
//! `ADL_ERR_NOT_SUPPORTED` to both Overdrive calls — require. Anything
//! missing or unsupported is treated as "no sensor", never as an error.

use std::{
    ffi::{c_int, c_void},
    sync::OnceLock,
};

use windows::{
    Win32::{
        Foundation::HMODULE,
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
    },
    core::{PCSTR, PCWSTR},
};

use crate::{app::filter::Filter, collection::temperature::TempSensorData};

/// `ADL_OK` from the ADL SDK (`adl_defines.h`).
const ADL_OK: c_int = 0;

/// `ADLODNTemperatureType::EDGE`; hotspot/memory vary by driver support,
/// edge is the universally reported one.
const ADL_ODN_TEMP_EDGE: c_int = 1;

/// `ATI_VENDOR_ID` from the ADL SDK.
const ATI_VENDOR_ID: u32 = 0x1002;

/// `ADL_MAX_PATH` from the ADL SDK.
const ADL_MAX_PATH: usize = 256;

/// `ADL_PMLOG_MAX_SENSORS` from the ADL SDK.
const ADL_PMLOG_MAX_SENSORS: usize = 256;

/// PMLog temperature sensor indices (`ADLPMLogSensors` in the ADL SDK),
/// in the order we prefer them.
const ADL_PMLOG_TEMP_SENSORS: &[usize] = &[
    8,  // ADL_PMLOG_TEMPERATURE_EDGE
    27, // ADL_PMLOG_TEMPERATURE_HOTSPOT
    28, // ADL_PMLOG_TEMPERATURE_GFX
    29, // ADL_PMLOG_TEMPERATURE_SOC
    9,  // ADL_PMLOG_TEMPERATURE_MEM
    32, // ADL_PMLOG_TEMPERATURE_CPU
];

/// `AdapterInfo` from the ADL SDK (`adl_structures.h`).
///
/// Layout must match the C struct exactly; only the fields we read are
/// named after their use, the rest preserves positional layout.
#[repr(C)]
#[derive(Clone)]
struct AdapterInfo {
    size: c_int,
    adapter_index: c_int,
    udid: [u8; ADL_MAX_PATH],
    bus_number: c_int,
    device_number: c_int,
    function_number: c_int,
    vendor_id: c_int,
    adapter_name: [u8; ADL_MAX_PATH],
    display_name: [u8; ADL_MAX_PATH],
    present: c_int,
    exist: c_int,
    driver_path: [u8; ADL_MAX_PATH],
    driver_path_ext: [u8; ADL_MAX_PATH],
    pnp_string: [u8; ADL_MAX_PATH],
    os_display_index: c_int,
}

impl Default for AdapterInfo {
    fn default() -> Self {
        // SAFETY: all-zero is a valid initial state; `size` is set by the
        // caller before handing the buffer to ADL.
        unsafe { std::mem::zeroed() }
    }
}

/// One PMLog sensor reading (`ADLSingleSensorData` in the ADL SDK).
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct PmLogSensor {
    supported: c_int,
    value: c_int,
}

/// `ADLPMLogDataOutput` in the ADL SDK.
#[repr(C)]
struct PmLogData {
    size: c_int,
    sensors: [PmLogSensor; ADL_PMLOG_MAX_SENSORS],
}

impl Default for PmLogData {
    fn default() -> Self {
        Self {
            size: std::mem::size_of::<Self>() as c_int,
            sensors: [PmLogSensor::default(); ADL_PMLOG_MAX_SENSORS],
        }
    }
}

/// Opaque ADL client context (`ADL_CONTEXT_HANDLE`).
type AdlContext = *mut c_void;

/// `ADL_MAIN_MALLOC_CALLBACK`: `void* (*)(int)`.
type AdlMalloc = extern "C" fn(c_int) -> *mut c_void;

type FnCreate = unsafe extern "C" fn(AdlMalloc, c_int, *mut AdlContext) -> c_int;
type FnNumAdapters = unsafe extern "C" fn(AdlContext, *mut c_int) -> c_int;
type FnAdapterInfo = unsafe extern "C" fn(AdlContext, *mut AdapterInfo, c_int) -> c_int;
type FnTempN = unsafe extern "C" fn(AdlContext, c_int, c_int, *mut c_int) -> c_int;
type FnTemp6 = unsafe extern "C" fn(AdlContext, c_int, *mut c_int) -> c_int;
type FnPmLog = unsafe extern "C" fn(AdlContext, c_int, *mut PmLogData) -> c_int;

/// Dynamically loaded ADL2 entry points plus the client context.
///
/// The library handle is intentionally kept loaded for the process lifetime
/// (same as fastfetch's ADL wrapper): unloading while the context is alive
/// would leave dangling code pointers.
struct AdlApi {
    _lib: HMODULE,
    context: AdlContext,
    num_adapters: FnNumAdapters,
    adapter_info: FnAdapterInfo,
    temp_n: Option<FnTempN>,
    temp_6: Option<FnTemp6>,
    pm_log: Option<FnPmLog>,
}

// SAFETY: initialized exactly once via `ADL`; `btm` drives collection from
// a single thread, and the stored entry points are safe to call.
unsafe impl Send for AdlApi {}
unsafe impl Sync for AdlApi {}

static ADL: OnceLock<Option<AdlApi>> = OnceLock::new();

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
}

/// Allocation callback handed to `ADL2_Main_Control_Create`.
///
/// Forwards to the shared CRT heap, matching what ADL expects (this is what
/// the ADL SDK samples and fastfetch do with plain `malloc`).
extern "C" fn adl_malloc(size: c_int) -> *mut c_void {
    if size <= 0 {
        std::ptr::null_mut()
    } else {
        // SAFETY: `size` was checked positive; OOM yields null, which ADL
        // treats as allocation failure.
        unsafe { malloc(size as usize) }
    }
}

/// Raw resolved export, transmuted to its ADL SDK signature at each call
/// site where the concrete type is known.
fn load_raw(lib: HMODULE, name: &[u8]) -> Option<unsafe extern "system" fn() -> isize> {
    // `GetProcAddress` itself just resolves an address; calling the result
    // is where the safety obligations live (handled at each call site).
    unsafe { GetProcAddress(lib, PCSTR(name.as_ptr())) }
}

fn load_adl_library() -> Option<HMODULE> {
    use std::os::windows::ffi::OsStrExt;

    // Prefer the absolute System32 path so a plain filename lookup (and its
    // search-order hijack surface) is only a fallback.
    let mut candidates = Vec::with_capacity(2);
    if let Ok(system_root) = std::env::var("SystemRoot") {
        candidates.push(format!("{system_root}\\System32\\atiadlxx.dll"));
    }
    candidates.push("atiadlxx.dll".to_string());

    candidates.into_iter().find_map(|path| {
        let wide: Vec<u16> = std::ffi::OsStr::new(&path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        // `LoadLibraryW` failure just means "try the next candidate".
        unsafe { LoadLibraryW(PCWSTR(wide.as_ptr())).ok() }
    })
}

fn init_adl() -> Option<AdlApi> {
    let lib = load_adl_library()?;

    // SAFETY: each address is transmuted only to the signature the ADL SDK
    // declares for that export name. Missing *required* symbols mean this
    // driver has no usable ADL2 surface; missing temperature symbols just
    // rule out that read path.
    unsafe {
        let create: FnCreate = std::mem::transmute(load_raw(lib, b"ADL2_Main_Control_Create\0")?);
        let num_adapters: FnNumAdapters =
            std::mem::transmute(load_raw(lib, b"ADL2_Adapter_NumberOfAdapters_Get\0")?);
        let adapter_info: FnAdapterInfo =
            std::mem::transmute(load_raw(lib, b"ADL2_Adapter_AdapterInfo_Get\0")?);
        let temp_n: Option<FnTempN> =
            load_raw(lib, b"ADL2_OverdriveN_Temperature_Get\0").map(|f| std::mem::transmute(f));
        let temp_6: Option<FnTemp6> =
            load_raw(lib, b"ADL2_Overdrive6_Temperature_Get\0").map(|f| std::mem::transmute(f));
        let pm_log: Option<FnPmLog> =
            load_raw(lib, b"ADL2_New_QueryPMLogData_Get\0").map(|f| std::mem::transmute(f));

        if temp_n.is_none() && temp_6.is_none() && pm_log.is_none() {
            return None;
        }

        let mut context: AdlContext = std::ptr::null_mut();
        if create(adl_malloc, 1, &mut context) != ADL_OK || context.is_null() {
            return None;
        }

        Some(AdlApi {
            _lib: lib,
            context,
            num_adapters,
            adapter_info,
            temp_n,
            temp_6,
            pm_log,
        })
    }
}

/// Decode a NUL-terminated ANSI byte string from ADL.
fn decode_ansi(field: &[u8]) -> String {
    let len = field.iter().position(|&b| b == 0).unwrap_or(field.len());
    String::from_utf8_lossy(&field[..len]).trim().to_string()
}

/// Vendor ID from a Windows adapter UDID (`PCI_VEN_1002&DEV_...`).
///
/// ADL's `AdapterInfo::VendorID` integer field is known to be unreliable on
/// Windows (LibreHardwareMonitor re-parses it from the UDID for the same
/// reason), so the UDID is the source of truth here.
fn udid_vendor(udid: &str) -> Option<u32> {
    let hex = udid.split("PCI_VEN_").nth(1)?.get(..4)?;
    u32::from_str_radix(hex, 16).ok()
}

/// Read one adapter's temperature in Celsius, if the driver reports it.
fn read_adapter_temp(api: &AdlApi, adapter_index: c_int) -> Option<f32> {
    // SAFETY: `api.context` was created by `ADL2_Main_Control_Create`;
    // out-pointers are stack locals (or caller-owned buffers) valid for
    // the call.
    unsafe {
        let mut milli_celsius: c_int = 0;
        if let Some(temp_n) = api.temp_n
            && temp_n(
                api.context,
                adapter_index,
                ADL_ODN_TEMP_EDGE,
                &mut milli_celsius,
            ) == ADL_OK
        {
            return Some(milli_celsius as f32 / 1000.0);
        }
        if let Some(temp_6) = api.temp_6
            && temp_6(api.context, adapter_index, &mut milli_celsius) == ADL_OK
        {
            return Some(milli_celsius as f32 / 1000.0);
        }
        if let Some(pm_log) = api.pm_log {
            let mut data = PmLogData::default();
            if pm_log(api.context, adapter_index, &mut data) == ADL_OK {
                for &sensor in ADL_PMLOG_TEMP_SENSORS {
                    let reading = &data.sensors[sensor];
                    if reading.supported != 0 {
                        // PMLog temperatures are already in degrees Celsius
                        // (unlike the millidegree Overdrive readings).
                        return Some(reading.value as f32);
                    }
                }
            }
        }
    }
    None
}

/// AMD GPU temperatures on Windows via ADL2.
///
/// Returns `None` when no AMD driver/sensor is available. Adapters that
/// fail the temperature read are reported with `temperature: None`, matching
/// the NVIDIA collector's convention, so the widget still lists the GPU.
pub fn get_amd_windows_temps(
    filter: &Option<Filter>, graph_filter: &Option<Filter>,
) -> Option<Vec<TempSensorData>> {
    let api = ADL.get_or_init(init_adl).as_ref()?;

    // SAFETY: `api.context` is live; the info buffer is caller-owned and
    // sized for exactly `count` entries.
    unsafe {
        let mut count: c_int = 0;
        if (api.num_adapters)(api.context, &mut count) != ADL_OK || count <= 0 {
            return None;
        }

        let n = count as usize;
        let mut infos: Vec<AdapterInfo> = vec![AdapterInfo::default(); n];
        for info in &mut infos {
            info.size = std::mem::size_of::<AdapterInfo>() as c_int;
        }
        let byte_len = (n * std::mem::size_of::<AdapterInfo>()) as c_int;
        if (api.adapter_info)(api.context, infos.as_mut_ptr(), byte_len) != ADL_OK {
            return None;
        }

        let mut temps = Vec::new();
        // ADL enumerates one entry per display output, so the same physical
        // GPU appears multiple times; dedupe by PCI location.
        let mut seen_pci = Vec::with_capacity(infos.len());
        for info in &infos {
            if info.present == 0 {
                continue;
            }
            let udid = decode_ansi(&info.udid);
            if udid_vendor(&udid) != Some(ATI_VENDOR_ID) {
                continue;
            }
            let pci = (info.bus_number, info.device_number, info.function_number);
            if seen_pci.contains(&pci) {
                continue;
            }
            seen_pci.push(pci);

            let name = decode_ansi(&info.adapter_name);
            if name.is_empty()
                || !(Filter::optional_should_keep(filter, &name)
                    || Filter::optional_should_keep(graph_filter, &name))
            {
                continue;
            }

            temps.push(TempSensorData {
                name,
                temperature: read_adapter_temp(api, info.adapter_index),
            });
        }

        (!temps.is_empty()).then_some(temps)
    }
}
