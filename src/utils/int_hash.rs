//! A simple hasher that literally just maps an int to itself.
//!
//! Originally based on <https://github.com/tetcoin/nohash/blob/master/src/lib.rs>.

use std::{
    hash::{BuildHasherDefault, Hasher},
    marker::PhantomData,
};

/// A hash map that directly maps from an integer key to a value.
pub type IntHashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<IntHasher<K>>>;

/// A hash set that directly uses integer keys.
#[allow(dead_code)]
pub type IntHashSet<K> = std::collections::HashSet<K, BuildHasherDefault<IntHasher<K>>>;

pub trait SupportedInt {}

impl SupportedInt for u8 {}
impl SupportedInt for u16 {}
impl SupportedInt for u32 {}
impl SupportedInt for u64 {}
impl SupportedInt for usize {}
impl SupportedInt for i8 {}
impl SupportedInt for i16 {}
impl SupportedInt for i32 {}
impl SupportedInt for i64 {}
impl SupportedInt for isize {}

#[derive(Default)]
pub struct IntHasher<T: SupportedInt> {
    inner: u64,
    _marker: PhantomData<T>,
}

impl<T: SupportedInt> Hasher for IntHasher<T> {
    fn finish(&self) -> u64 {
        self.inner
    }

    fn write(&mut self, _bytes: &[u8]) {
        panic!("IntHasher does not support arbitrary writes")
    }

    fn write_u8(&mut self, i: u8) {
        self.inner = i as u64;
    }

    fn write_u16(&mut self, i: u16) {
        self.inner = i as u64;
    }

    fn write_u32(&mut self, i: u32) {
        self.inner = i as u64;
    }

    fn write_u64(&mut self, i: u64) {
        self.inner = i;
    }

    fn write_usize(&mut self, i: usize) {
        self.inner = i as u64;
    }

    fn write_i8(&mut self, i: i8) {
        self.inner = i as u64;
    }

    fn write_i16(&mut self, i: i16) {
        self.inner = i as u64;
    }

    fn write_i32(&mut self, i: i32) {
        self.inner = i as u64;
    }

    fn write_i64(&mut self, i: i64) {
        self.inner = i as u64;
    }

    fn write_isize(&mut self, i: isize) {
        self.inner = i as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8() {
        let mut hasher = IntHasher::<u8>::default();
        hasher.write_u8(42);
        assert_eq!(hasher.finish(), 42);
    }

    #[test]
    fn test_u16() {
        let mut hasher = IntHasher::<u16>::default();
        hasher.write_u16(4242);
        assert_eq!(hasher.finish(), 4242);
    }

    #[test]
    fn test_u32() {
        let mut hasher = IntHasher::<u32>::default();
        hasher.write_u32(424242);
        assert_eq!(hasher.finish(), 424242);
    }

    #[test]
    fn test_u64() {
        let mut hasher = IntHasher::<u64>::default();
        hasher.write_u64(4242424242);
        assert_eq!(hasher.finish(), 4242424242);
    }

    #[test]
    fn test_usize() {
        let mut hasher = IntHasher::<usize>::default();
        hasher.write_usize(999999);
        assert_eq!(hasher.finish(), 999999);
    }

    #[test]
    fn test_i8() {
        let mut hasher = IntHasher::<i8>::default();
        hasher.write_i8(-42);
        assert_eq!(hasher.finish(), -42_i8 as u64);
    }

    #[test]
    fn test_i16() {
        let mut hasher = IntHasher::<i16>::default();
        hasher.write_i16(-4242);
        assert_eq!(hasher.finish(), -4242_i64 as u64);
    }

    #[test]
    fn test_i32() {
        let mut hasher = IntHasher::<i32>::default();
        hasher.write_i32(-424242);
        assert_eq!(hasher.finish(), -424242_i64 as u64);
    }

    #[test]
    fn test_i64() {
        let mut hasher = IntHasher::<i64>::default();
        hasher.write_i64(-4242424242);
        assert_eq!(hasher.finish(), -4242424242_i64 as u64);
    }

    #[test]
    fn test_isize() {
        let mut hasher = IntHasher::<isize>::default();
        hasher.write_isize(-424242);
        assert_eq!(hasher.finish(), -424242_isize as u64);
    }

    #[test]
    fn test_int_hash_map() {
        let mut map = IntHashMap::<u32, &str>::default();
        map.insert(1, "one");
        map.insert(2, "two");
        assert_eq!(map.get(&1), Some(&"one"));
        assert_eq!(map.get(&2), Some(&"two"));
        assert_eq!(map.get(&3), None);
    }

    #[test]
    fn test_int_hash_set() {
        let mut set = IntHashSet::<u32>::default();
        set.insert(1);
        set.insert(2);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(!set.contains(&3));
    }
}
