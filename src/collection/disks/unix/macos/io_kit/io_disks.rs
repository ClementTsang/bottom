use anyhow::bail;
use mach2::kern_return;

use super::{IoIterator, bindings::*};

pub fn get_disks() -> anyhow::Result<IoIterator> {
    let mut media_iter: io_iterator_t = 0;

    // SAFETY: This is a safe syscall via IOKit, all the arguments should be safe.
    let result = unsafe {
        IOServiceGetMatchingServices(
            kIOMasterPortDefault,
            IOServiceMatching(kIOMediaClass.as_ptr().cast()),
            &mut media_iter,
        )
    };

    if result == kern_return::KERN_SUCCESS {
        Ok(media_iter.into())
    } else {
        bail!("IOServiceGetMatchingServices failed, error code {result}");
    }
}
