//! C FFI bindings for [IOKit](https://developer.apple.com/documentation/iokit/).
//!
//! Based on [heim](https://github.com/heim-rs/heim/blob/master/heim-common/src/sys/macos/iokit/io_master_port.rs)
//! and [sysinfo's implementation](https://github.com/GuillaumeGomez/sysinfo/blob/master/src/apple/macos/ffi.rs).
//!
//! Ideally, we can remove this if sysinfo ever gains disk I/O capabilities.

use core_foundation::{
    base::{CFAllocatorRef, mach_port_t},
    dictionary::CFMutableDictionaryRef,
};
use libc::c_char;
use mach2::{kern_return::kern_return_t, port::MACH_PORT_NULL};

#[expect(non_camel_case_types)]
pub type io_object_t = mach_port_t;

#[expect(non_camel_case_types)]
pub type io_iterator_t = io_object_t;
#[expect(non_camel_case_types)]
pub type io_registry_entry_t = io_object_t;

pub type IOOptionBits = u32;

/// See https://github.com/1kc/librazermacos/pull/27#issuecomment-1042368531.
#[expect(non_upper_case_globals)]
pub const kIOMasterPortDefault: mach_port_t = MACH_PORT_NULL;

#[expect(non_upper_case_globals)]
pub const kIOServicePlane: &str = "IOService\0";

#[expect(non_upper_case_globals)]
pub const kIOMediaClass: &str = "IOMedia\0";

// SAFETY: Bindings like this are inherently unsafe. See [here](https://developer.apple.com/documentation/iokit) for
// more details.
unsafe extern "C" {

    pub fn IOServiceGetMatchingServices(
        mainPort: mach_port_t, matching: CFMutableDictionaryRef, existing: *mut io_iterator_t,
    ) -> kern_return_t;

    pub fn IOServiceMatching(name: *const c_char) -> CFMutableDictionaryRef;

    pub fn IOIteratorNext(iterator: io_iterator_t) -> io_object_t;

    pub fn IOObjectRelease(obj: io_object_t) -> kern_return_t;

    pub fn IORegistryEntryGetParentEntry(
        entry: io_registry_entry_t, plane: *const libc::c_char, parent: *mut io_registry_entry_t,
    ) -> kern_return_t;

    // pub fn IOObjectConformsTo(object: io_object_t, className: *const
    // libc::c_char) -> mach2::boolean::boolean_t;

    pub fn IORegistryEntryCreateCFProperties(
        entry: io_registry_entry_t, properties: *mut CFMutableDictionaryRef,
        allocator: CFAllocatorRef, options: IOOptionBits,
    ) -> kern_return_t;

}
