//! C FFI bindings for [IOKit](https://developer.apple.com/documentation/iokit/).
//!
//! Based on [heim's implementation](https://github.com/heim-rs/heim/blob/master/heim-common/src/sys/macos/iokit/ffi.rs).
//!
//! Ideally, we can remove this if sysinfo ever gains disk I/O capabilities... it even already
//! does all of these bindings!

use mach2::{kern_return::kern_return_t, port::mach_port_t};

pub type io_object_t = mach_port_t;

extern "C" {
    /// See <https://developer.apple.com/documentation/iokit/kiomasterportdefault> for more info.
    ///
    /// Note this is deprecated in favour of [`kIOMainPortDefault`] for macOS 12.0+.
    pub static kIOMasterPortDefault: mach_port_t;

    /// See <https://developer.apple.com/documentation/iokit/kiomainportdefault> for more info.
    pub static kIOMainPortDefault: mach_port_t;

    /// Returns the mach port used to initiate communication with IOKit. The port should be
    /// deallocated with [`IOObjectRelease`]. See <https://developer.apple.com/documentation/iokit/1514652-iomasterport>
    /// for more info.
    ///
    /// Note this is deprecated in favour of [`IOMainPort`] for macOS 12.0+.
    pub fn IOMasterPort(bootstrapPort: mach_port_t, masterPort: *mut mach_port_t) -> kern_return_t;

    /// Returns the mach port used to initiate communication with IOKit. The port should be
    /// deallocated with [`IOObjectRelease`]. See <https://developer.apple.com/documentation/iokit/3753260-iomainport>
    /// for more info.
    pub fn IOMainPort(bootstrapPort: mach_port_t, masterPort: *mut mach_port_t) -> kern_return_t;

    /// Releases an object handle previously returned by IOKitLib. See <https://developer.apple.com/documentation/iokit/1514627-ioobjectrelease>
    /// for more info.
    pub fn IOObjectRelease(object: io_object_t) -> kern_return_t;
}
