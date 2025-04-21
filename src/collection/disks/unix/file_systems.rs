use std::str::FromStr;

use crate::multi_eq_ignore_ascii_case;

/// Known filesystems. Original list from
/// [heim](https://github.com/heim-rs/heim/blob/master/heim-disk/src/filesystem.rs).
///
/// All physical filesystems should have their own enum element and all virtual
/// filesystems will go into the [`FileSystem::Other`] element.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
#[non_exhaustive]
pub enum FileSystem {
    /// ext2 (<https://en.wikipedia.org/wiki/Ext2>)
    Ext2,

    /// ext3 (<https://en.wikipedia.org/wiki/Ext3>)
    Ext3,

    /// ext4 (<https://en.wikipedia.org/wiki/Ext4>)
    Ext4,

    /// FAT (<https://en.wikipedia.org/wiki/File_Allocation_Table>)
    VFat,

    /// exFAT (<https://en.wikipedia.org/wiki/ExFAT>)
    ExFat,

    /// F2FS (<https://en.wikipedia.org/wiki/F2FS>)
    F2fs,

    /// NTFS (<https://en.wikipedia.org/wiki/NTFS>)
    Ntfs,

    /// ZFS (<https://en.wikipedia.org/wiki/ZFS>)
    Zfs,

    /// HFS (<https://en.wikipedia.org/wiki/Hierarchical_File_System>)
    Hfs,

    /// HFS+ (<https://en.wikipedia.org/wiki/HFS_Plus>)
    HfsPlus,

    /// JFS (<https://en.wikipedia.org/wiki/JFS_(file_system)>)
    Jfs,

    /// ReiserFS 3 (<https://en.wikipedia.org/wiki/ReiserFS>)
    Reiser3,

    /// ReiserFS 4 (<https://en.wikipedia.org/wiki/Reiser4>)
    Reiser4,

    /// Btrfs (<https://en.wikipedia.org/wiki/Btrfs>)
    Btrfs,

    /// Bcachefs (<https://en.wikipedia.org/wiki/Bcachefs>)
    Bcachefs,

    /// MINIX FS (<https://en.wikipedia.org/wiki/MINIX_file_system>)
    Minix,

    /// NILFS (<https://en.wikipedia.org/wiki/NILFS>)
    Nilfs,

    /// XFS (<https://en.wikipedia.org/wiki/XFS>)
    Xfs,

    /// APFS (<https://en.wikipedia.org/wiki/Apple_File_System>)
    Apfs,

    /// FUSE (<https://en.wikipedia.org/wiki/Filesystem_in_Userspace>)
    FuseBlk,

    /// Some unspecified filesystem.
    Other(String),
}

impl FileSystem {
    /// Checks if filesystem is used for a physical devices.
    #[inline]
    pub fn is_physical(&self) -> bool {
        !self.is_virtual()
    }

    /// Checks if filesystem is used for a virtual devices (such as `tmpfs` or
    /// `smb` mounts).
    #[inline]
    pub fn is_virtual(&self) -> bool {
        matches!(self, FileSystem::Other(..))
    }

    #[expect(dead_code)]
    #[inline]
    /// Returns a string literal identifying this filesystem.
    pub fn as_str(&self) -> &str {
        match self {
            FileSystem::Ext2 => "ext2",
            FileSystem::Ext3 => "ext3",
            FileSystem::Ext4 => "ext4",
            FileSystem::VFat => "vfat",
            FileSystem::Ntfs => "ntfs",
            FileSystem::Zfs => "zfs",
            FileSystem::Hfs => "hfs",
            FileSystem::Reiser3 => "reiserfs",
            FileSystem::Reiser4 => "reiser4",
            FileSystem::FuseBlk => "fuseblk",
            FileSystem::ExFat => "exfat",
            FileSystem::F2fs => "f2fs",
            FileSystem::HfsPlus => "hfs+",
            FileSystem::Jfs => "jfs",
            FileSystem::Btrfs => "btrfs",
            FileSystem::Bcachefs => "bcachefs",
            FileSystem::Minix => "minix",
            FileSystem::Nilfs => "nilfs",
            FileSystem::Xfs => "xfs",
            FileSystem::Apfs => "apfs",
            FileSystem::Other(string) => string.as_str(),
        }
    }
}

impl FromStr for FileSystem {
    type Err = anyhow::Error;

    #[inline]
    fn from_str(s: &str) -> anyhow::Result<Self> {
        // Done like this as `eq_ignore_ascii_case` avoids a string allocation.
        Ok(if s.eq_ignore_ascii_case("ext2") {
            FileSystem::Ext2
        } else if s.eq_ignore_ascii_case("ext3") {
            FileSystem::Ext3
        } else if s.eq_ignore_ascii_case("ext4") {
            FileSystem::Ext4
        } else if multi_eq_ignore_ascii_case!(s, "msdos" | "vfat") {
            FileSystem::VFat
        } else if multi_eq_ignore_ascii_case!(s, "ntfs3" | "ntfs") {
            FileSystem::Ntfs
        } else if s.eq_ignore_ascii_case("zfs") {
            FileSystem::Zfs
        } else if s.eq_ignore_ascii_case("hfs") {
            FileSystem::Hfs
        } else if s.eq_ignore_ascii_case("reiserfs") {
            FileSystem::Reiser3
        } else if s.eq_ignore_ascii_case("reiser4") {
            FileSystem::Reiser4
        } else if s.eq_ignore_ascii_case("exfat") {
            FileSystem::ExFat
        } else if s.eq_ignore_ascii_case("f2fs") {
            FileSystem::F2fs
        } else if s.eq_ignore_ascii_case("hfsplus") {
            FileSystem::HfsPlus
        } else if s.eq_ignore_ascii_case("jfs") {
            FileSystem::Jfs
        } else if s.eq_ignore_ascii_case("btrfs") {
            FileSystem::Btrfs
        } else if s.eq_ignore_ascii_case("bcachefs") {
            FileSystem::Bcachefs
        } else if s.eq_ignore_ascii_case("minix") {
            FileSystem::Minix
        } else if multi_eq_ignore_ascii_case!(s, "nilfs" | "nilfs2") {
            FileSystem::Nilfs
        } else if s.eq_ignore_ascii_case("xfs") {
            FileSystem::Xfs
        } else if s.eq_ignore_ascii_case("apfs") {
            FileSystem::Apfs
        } else if s.eq_ignore_ascii_case("fuseblk") {
            FileSystem::FuseBlk
        } else {
            FileSystem::Other(s.to_string())
        })
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::FileSystem;

    #[test]
    fn file_system_from_str() {
        // Something supported
        assert_eq!(FileSystem::from_str("ext4").unwrap(), FileSystem::Ext4);
        assert_eq!(FileSystem::from_str("msdos").unwrap(), FileSystem::VFat);
        assert_eq!(FileSystem::from_str("vfat").unwrap(), FileSystem::VFat);

        // Something unsupported
        assert_eq!(
            FileSystem::from_str("this does not exist").unwrap(),
            FileSystem::Other("this does not exist".to_owned())
        );
    }
}
