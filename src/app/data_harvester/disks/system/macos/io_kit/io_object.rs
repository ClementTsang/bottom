//! Based on [heim's](https://github.com/heim-rs/heim/blob/master/heim-common/src/sys/macos/iokit/io_object.rs)
//! implementation.

use std::mem;

use anyhow::{anyhow, bail};
use core_foundation::base::{kCFAllocatorDefault, CFType, TCFType, ToVoid};
use core_foundation::dictionary::{
    CFDictionary, CFDictionaryGetTypeID, CFDictionaryRef, CFMutableDictionary,
    CFMutableDictionaryRef,
};
use core_foundation::number::{CFNumber, CFNumberGetTypeID};
use core_foundation::string::{CFString, CFStringGetTypeID};
use mach2::kern_return;

use super::bindings::*;

/// Safe wrapper around the IOKit `io_object_t` type.
#[derive(Debug)]
pub struct IoObject(io_object_t);

impl IoObject {
    /// Returns a typed dictionary with this object's properties.
    pub fn properties(&self) -> anyhow::Result<CFDictionary<CFString, CFType>> {
        unsafe {
            let mut props = mem::MaybeUninit::<CFMutableDictionaryRef>::uninit();

            let result = IORegistryEntryCreateCFProperties(
                self.0,
                props.as_mut_ptr(),
                kCFAllocatorDefault,
                0,
            );

            if result != kern_return::KERN_SUCCESS {
                bail!("IORegistryEntryCreateCFProperties failed, error code {result}.")
            } else {
                let props = props.assume_init();
                Ok(CFMutableDictionary::wrap_under_create_rule(props).to_immutable())
            }
        }
    }

    /// Gets the [`kIOServicePlane`] parent [`io_object_t`] for this [`io_object_t`], if there
    /// is one.
    pub fn service_parent(&self) -> anyhow::Result<IoObject> {
        let mut parent: io_registry_entry_t = 0;

        let result = unsafe {
            IORegistryEntryGetParentEntry(self.0, kIOServicePlane.as_ptr().cast(), &mut parent)
        };

        if result != kern_return::KERN_SUCCESS {
            bail!("IORegistryEntryGetParentEntry failed, error code {result}.")
        } else {
            Ok(parent.into())
        }
    }

    pub fn conforms_to_block_storage_driver(&self) -> bool {
        let result =
            unsafe { IOObjectConformsTo(self.0, "IOBlockStorageDriver\0".as_ptr().cast()) };

        result != 0
    }
}

impl From<io_object_t> for IoObject {
    fn from(obj: io_object_t) -> IoObject {
        IoObject(obj)
    }
}

impl Drop for IoObject {
    fn drop(&mut self) {
        let result = unsafe { IOObjectRelease(self.0) };
        assert_eq!(result, kern_return::KERN_SUCCESS);
    }
}

pub fn get_dict(
    dict: &CFDictionary<CFString, CFType>, raw_key: &'static str,
) -> anyhow::Result<CFDictionary<CFString, CFType>> {
    let key = CFString::from_static_string(raw_key);

    dict.find(&key)
        .map(|value_ref| {
            unsafe {
                debug_assert!(value_ref.type_of() == CFDictionaryGetTypeID());
            }

            // "Casting" `CFDictionary<*const void, *const void>` into a needed dict type

            // TODO: I think that reference to an original dict is still stored somewhere
            // and it does not decrements here.
            let ptr = value_ref.to_void() as CFDictionaryRef;

            unsafe { CFDictionary::wrap_under_get_rule(ptr) }
        })
        .ok_or_else(|| anyhow!("missing key"))
}

pub fn get_i64(
    dict: &CFDictionary<CFString, CFType>, raw_key: &'static str,
) -> anyhow::Result<i64> {
    let key = CFString::from_static_string(raw_key);

    dict.find(&key)
        .and_then(|value_ref| {
            unsafe {
                debug_assert!(value_ref.type_of() == CFNumberGetTypeID());
            }

            value_ref.downcast::<CFNumber>()
        })
        .and_then(|number| number.to_i64())
        .ok_or_else(|| anyhow!("missing key"))
}

pub fn get_string(
    dict: &CFDictionary<CFString, CFType>, raw_key: &'static str,
) -> anyhow::Result<String> {
    let key = CFString::from_static_string(raw_key);

    dict.find(&key)
        .and_then(|value_ref| {
            unsafe {
                debug_assert!(value_ref.type_of() == CFStringGetTypeID());
            }

            value_ref.downcast::<CFString>()
        })
        .map(|cf_string| cf_string.to_string())
        .ok_or_else(|| anyhow!("missing key"))
}
