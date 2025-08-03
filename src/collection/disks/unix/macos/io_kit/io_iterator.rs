//! Based on [heim's](https://github.com/heim-rs/heim/blob/master/heim-common/src/sys/macos/iokit/io_iterator.rs).
//! implementation.

use std::ops::{Deref, DerefMut};

use mach2::kern_return;

use super::{bindings::*, io_object::IoObject};

/// Safe wrapper around the IOKit `io_iterator_t` type.
#[derive(Debug)]
pub struct IoIterator(io_iterator_t);

impl From<io_iterator_t> for IoIterator {
    fn from(iter: io_iterator_t) -> IoIterator {
        IoIterator(iter)
    }
}

impl Deref for IoIterator {
    type Target = io_iterator_t;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for IoIterator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Iterator for IoIterator {
    type Item = IoObject;

    fn next(&mut self) -> Option<Self::Item> {
        // Basically, we just stop when we hit 0.

        // SAFETY: IOKit call, the passed argument (an `io_iterator_t`) is what is
        // expected.
        match unsafe { IOIteratorNext(self.0) } {
            0 => None,
            io_object => Some(IoObject::from(io_object)),
        }
    }
}

impl Drop for IoIterator {
    fn drop(&mut self) {
        // SAFETY: IOKit call, the passed argument (an `io_iterator_t`) is what is
        // expected.
        let result = unsafe { IOObjectRelease(self.0) };
        assert_eq!(result, kern_return::KERN_SUCCESS);
    }
}
