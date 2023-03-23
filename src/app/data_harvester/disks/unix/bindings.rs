//! Based on [heim's](https://github.com/heim-rs/heim/blob/master/heim-disk/src/sys/unix/bindings/mod.rs)
//! implementation.

use std::io::Error;

const MNT_NOWAIT: libc::c_int = 2;

extern "C" {
    fn getfsstat64(buf: *mut libc::statfs, bufsize: libc::c_int, flags: libc::c_int)
        -> libc::c_int;
}

/// Returns all the mounts on the system at the moment.
pub(crate) fn mounts() -> anyhow::Result<Vec<libc::statfs>> {
    let expected_len = unsafe { getfsstat64(std::ptr::null_mut(), 0, MNT_NOWAIT) };
    let mut mounts: Vec<libc::statfs> = Vec::with_capacity(expected_len as usize);
    let result = unsafe {
        getfsstat64(
            mounts.as_mut_ptr(),
            std::mem::size_of::<libc::statfs>() as libc::c_int * expected_len,
            MNT_NOWAIT,
        )
    };
    if result == -1 {
        return Err(anyhow::Error::from(Error::last_os_error()).context("getfsstat64"));
    } else {
        debug_assert_eq!(
            expected_len, result,
            "Expected {expected_len} statfs entries, but instead got {result} entries",
        );
        unsafe {
            mounts.set_len(result as usize);
        }
    }

    Ok(mounts)
}
