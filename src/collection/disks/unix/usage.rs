pub struct Usage(libc::statvfs);

// Note that x86 returns `u32` values while x86-64 returns `u64`s, so we convert
// everything to `u64` for consistency.
#[expect(clippy::useless_conversion)]
impl Usage {
    pub(crate) fn new(vfs: libc::statvfs) -> Self {
        Self(vfs)
    }

    /// Returns the total number of bytes available.
    pub fn total(&self) -> u64 {
        u64::from(self.0.f_blocks) * u64::from(self.0.f_frsize)
    }

    /// Returns the available number of bytes used. Note this is not necessarily
    /// the same as [`Usage::free`].
    pub fn available(&self) -> u64 {
        u64::from(self.0.f_bfree) * u64::from(self.0.f_frsize)
    }

    #[expect(dead_code)]
    /// Returns the total number of bytes used. Equal to `total - available` on
    /// Unix.
    pub fn used(&self) -> u64 {
        let avail_to_root = u64::from(self.0.f_bfree) * u64::from(self.0.f_frsize);
        self.total() - avail_to_root
    }

    /// Returns the total number of bytes free. Note this is not necessarily the
    /// same as [`Usage::available`].
    pub fn free(&self) -> u64 {
        u64::from(self.0.f_bavail) * u64::from(self.0.f_frsize)
    }
}
