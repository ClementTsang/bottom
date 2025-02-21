//! Based on [heim's](https://github.com/heim-rs/heim/blob/master/heim-disk/src/sys/unix/bindings/mod.rs)
//! implementation.

use std::io::Error;

const MNT_NOWAIT: libc::c_int = 2;

// SAFETY: Bindings like this are inherently unsafe.
unsafe extern "C" {
    fn getfsstat64(buf: *mut libc::statfs, bufsize: libc::c_int, flags: libc::c_int)
    -> libc::c_int;
}

/// Returns all the mounts on the system at the moment.
pub(crate) fn mounts() -> anyhow::Result<Vec<libc::statfs>> {
    // SAFETY: System API FFI call, arguments should be correct.
    let expected_len = unsafe { getfsstat64(std::ptr::null_mut(), 0, MNT_NOWAIT) };

    let mut mounts: Vec<libc::statfs> = Vec::with_capacity(expected_len as usize);

    // SAFETY: System API FFI call, arguments should be correct.
    let result = unsafe {
        getfsstat64(
            mounts.as_mut_ptr(),
            std::mem::size_of::<libc::statfs>() as libc::c_int * expected_len,
            MNT_NOWAIT,
        )
    };

    if result == -1 {
        Err(anyhow::Error::from(Error::last_os_error()).context("getfsstat64"))
    } else {
        debug_assert_eq!(
            expected_len, result,
            "Expected {expected_len} statfs entries, but instead got {result} entries",
        );

        // SAFETY: We have a debug assert check, and if `result` is not correct (-1), we
        // check against it. Otherwise, getfsstat64 should return the number of
        // statfs structures if it succeeded.
        //
        // Source: https://man.freebsd.org/cgi/man.cgi?query=getfsstat&sektion=2&format=html
        unsafe {
            mounts.set_len(result as usize);
        }
        Ok(mounts)
    }
}
