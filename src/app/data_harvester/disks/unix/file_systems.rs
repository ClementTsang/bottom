use std::str::FromStr;

/// Known filesystems. From [heim](https://github.com/heim-rs/heim/blob/master/heim-disk/src/filesystem.rs).
///
/// All physical filesystems should have their own enum element and all virtual filesystems will go into
/// the [`FileSystem::Other`] element.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
#[non_exhaustive]
pub enum FileSystem {
    /// ext2 (https://en.wikipedia.org/wiki/Ext2)
    Ext2,

    /// ext3 (https://en.wikipedia.org/wiki/Ext3)
    Ext3,

    /// ext4 (https://en.wikipedia.org/wiki/Ext4)
    Ext4,

    /// FAT (https://en.wikipedia.org/wiki/File_Allocation_Table)
    VFat,

    /// exFAT (https://en.wikipedia.org/wiki/ExFAT)
    ExFat,

    /// F2FS (https://en.wikipedia.org/wiki/F2FS)
    F2fs,

    /// NTFS (https://en.wikipedia.org/wiki/NTFS)
    Ntfs,

    /// ZFS (https://en.wikipedia.org/wiki/ZFS)
    Zfs,

    /// HFS (https://en.wikipedia.org/wiki/Hierarchical_File_System)
    Hfs,

    /// HFS+ (https://en.wikipedia.org/wiki/HFS_Plus)
    HfsPlus,

    /// JFS (https://en.wikipedia.org/wiki/JFS_(file_system))
    Jfs,

    /// ReiserFS 3 (https://en.wikipedia.org/wiki/ReiserFS)
    Reiser3,

    /// ReiserFS 4 (https://en.wikipedia.org/wiki/Reiser4)
    Reiser4,

    /// Btrfs (https://en.wikipedia.org/wiki/Btrfs)
    Btrfs,

    /// MINIX FS (https://en.wikipedia.org/wiki/MINIX_file_system)
    Minix,

    /// NILFS (https://en.wikipedia.org/wiki/NILFS)
    Nilfs,

    /// XFS (https://en.wikipedia.org/wiki/XFS)
    Xfs,

    /// APFS (https://en.wikipedia.org/wiki/Apple_File_System)
    Apfs,

    // TODO: Should it be considered as a physical FS?
    /// FUSE (https://en.wikipedia.org/wiki/Filesystem_in_Userspace)
    FuseBlk,

    // TODO: Extend list
    /// Some unspecified filesystem.
    Other(String),
}

impl FileSystem {
    /// Checks if filesystem is used for a physical devices.
    #[inline]
    pub fn is_physical(&self) -> bool {
        !self.is_virtual()
    }

    /// Checks if filesystem is used for a virtual devices (such as `tmpfs` or `smb` mounts).
    #[inline]
    pub fn is_virtual(&self) -> bool {
        matches!(self, FileSystem::Other(..))
    }

    #[allow(dead_code)]
    /// Returns a string identifying this filesystem.
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

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match () {
            _ if s.eq_ignore_ascii_case("ext2") => Ok(FileSystem::Ext2),
            _ if s.eq_ignore_ascii_case("ext3") => Ok(FileSystem::Ext3),
            _ if s.eq_ignore_ascii_case("ext4") => Ok(FileSystem::Ext4),
            _ if s.eq_ignore_ascii_case("vfat") => Ok(FileSystem::VFat),
            _ if s.eq_ignore_ascii_case("ntfs") => Ok(FileSystem::Ntfs),
            _ if s.eq_ignore_ascii_case("zfs") => Ok(FileSystem::Zfs),
            _ if s.eq_ignore_ascii_case("hfs") => Ok(FileSystem::Hfs),
            _ if s.eq_ignore_ascii_case("reiserfs") => Ok(FileSystem::Reiser3),
            _ if s.eq_ignore_ascii_case("reiser4") => Ok(FileSystem::Reiser4),
            _ if s.eq_ignore_ascii_case("exfat") => Ok(FileSystem::ExFat),
            _ if s.eq_ignore_ascii_case("f2fs") => Ok(FileSystem::F2fs),
            _ if s.eq_ignore_ascii_case("hfsplus") => Ok(FileSystem::HfsPlus),
            _ if s.eq_ignore_ascii_case("jfs") => Ok(FileSystem::Jfs),
            _ if s.eq_ignore_ascii_case("btrfs") => Ok(FileSystem::Btrfs),
            _ if s.eq_ignore_ascii_case("minix") => Ok(FileSystem::Minix),
            _ if s.eq_ignore_ascii_case("nilfs") => Ok(FileSystem::Nilfs),
            _ if s.eq_ignore_ascii_case("xfs") => Ok(FileSystem::Xfs),
            _ if s.eq_ignore_ascii_case("apfs") => Ok(FileSystem::Apfs),
            _ if s.eq_ignore_ascii_case("fuseblk") => Ok(FileSystem::FuseBlk),
            _ => Ok(FileSystem::Other(s.to_string())),
        }
    }
}
