//! Based on iced's implementation.

#[derive(Debug, Default)]
pub struct Hasher(twox_hash::XxHash64);

impl core::hash::Hasher for Hasher {
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes)
    }
}
