//! Partial bindings from Apple's open source code for getting process
//! information. Some of this is based on [heim's binding implementation](https://github.com/heim-rs/heim/blob/master/heim-process/src/sys/macos/bindings/process.rs).

use std::mem;

use anyhow::{Result, bail};
use libc::{
    CTL_KERN, KERN_PROC, KERN_PROC_PID, MAXCOMLEN, boolean_t, c_char, c_long, c_short, c_uchar,
    c_ushort, c_void, dev_t, gid_t, itimerval, pid_t, rusage, sigset_t, timeval, uid_t, xucred,
};
use mach2::vm_types::user_addr_t;

use crate::collection::Pid;

#[repr(C)]
pub(crate) struct kinfo_proc {
    pub kp_proc: extern_proc,
    pub kp_eproc: eproc,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct p_st1 {
    /// Doubly-linked run/sleep queue.
    p_forw: user_addr_t,
    p_back: user_addr_t,
}

#[repr(C)]
pub union p_un {
    pub p_st1: p_st1,

    /// process start time
    pub p_starttime: timeval,
}

/// Exported fields for kern sysctl. See
/// [`proc.h`](https://opensource.apple.com/source/xnu/xnu-201/bsd/sys/proc.h)
#[repr(C)]
pub(crate) struct extern_proc {
    pub p_un: p_un,

    /// Address space.
    pub p_vmspace: *mut vmspace,

    /// Signal actions, state (PROC ONLY). Should point to
    /// a `sigacts` but we don't really seem to need this.
    pub p_sigacts: user_addr_t,

    /// P_* flags.
    pub p_flag: i32,

    /// S* process status.
    pub p_stat: c_char,

    /// Process identifier.
    pub p_pid: pid_t,

    /// Save parent pid during ptrace.
    pub p_oppid: pid_t,

    /// Sideways return value from fdopen.
    pub p_dupfd: i32,

    /// where user stack was allocated
    pub user_stack: caddr_t,

    /// Which thread is exiting?
    pub exit_thread: *mut c_void,

    /// allow to debug
    pub p_debugger: i32,

    /// indication to suspend
    pub sigwait: boolean_t,

    /// Time averaged value of p_cpticks.
    pub p_estcpu: u32,

    /// Ticks of cpu time.
    pub p_cpticks: i32,

    /// %cpu for this process during p_swtime
    pub p_pctcpu: fixpt_t,

    /// Sleep address.
    pub p_wchan: *mut c_void,

    /// Reason for sleep.
    pub p_wmesg: *mut c_char,

    /// Time swapped in or out.
    pub p_swtime: u32,

    /// Time since last blocked.
    pub p_slptime: u32,

    /// Alarm timer.
    pub p_realtimer: itimerval,

    /// Real time.
    pub p_rtime: timeval,

    /// Statclock hit in user mode.
    pub p_uticks: u64,

    /// Statclock hits in system mode.
    pub p_sticks: u64,

    /// Statclock hits processing intr.
    pub p_iticks: u64,

    /// Kernel trace points.
    pub p_traceflag: i32,

    /// Trace to vnode. Originally a pointer to a struct of vnode.
    pub p_tracep: *mut c_void,

    /// DEPRECATED.
    pub p_siglist: i32,

    /// Vnode of executable. Originally a pointer to a struct of vnode.
    pub p_textvp: *mut c_void,

    /// If non-zero, don't swap.
    pub p_holdcnt: i32,

    /// DEPRECATED.
    pub p_sigmask: sigset_t,

    /// Signals being ignored.
    pub p_sigignore: sigset_t,

    /// Signals being caught by user.
    pub p_sigcatch: sigset_t,

    /// Process priority.
    pub p_priority: c_uchar,

    /// User-priority based on p_cpu and p_nice.
    pub p_usrpri: c_uchar,

    /// Process "nice" value.
    pub p_nice: c_char,

    pub p_comm: [c_char; MAXCOMLEN + 1],

    /// Pointer to process group. Originally a pointer to a `pgrp`.
    pub p_pgrp: *mut c_void,

    /// Kernel virtual addr of u-area (PROC ONLY). Originally a pointer to a
    /// `user`.
    pub p_addr: *mut c_void,

    /// Exit status for wait; also stop signal.
    pub p_xstat: c_ushort,

    /// Accounting flags.
    pub p_acflag: c_ushort,

    /// Exit information. XXX
    pub p_ru: *mut rusage,
}

const WMESGLEN: usize = 7;
const COMAPT_MAXLOGNAME: usize = 12;

/// See `_caddr_t.h`.
#[expect(non_camel_case_types)]
type caddr_t = *const libc::c_char;

/// See `types.h`.
#[expect(non_camel_case_types)]
type segsz_t = i32;

/// See `types.h`.
#[expect(non_camel_case_types)]
type fixpt_t = u32;

/// See [`proc.h`](https://opensource.apple.com/source/xnu/xnu-201/bsd/sys/proc.h)
#[repr(C)]
pub(crate) struct pcred {
    pub pc_lock: [c_char; 72],
    pub pc_ucred: *mut xucred,
    pub p_ruid: uid_t,
    pub p_svuid: uid_t,
    pub p_rgid: gid_t,
    pub p_svgid: gid_t,
    pub p_refcnt: i32,
}

/// See `vm.h`.
#[repr(C)]
pub(crate) struct vmspace {
    pub dummy: i32,
    pub dummy2: caddr_t,
    pub dummy3: [i32; 5],
    pub dummy4: [caddr_t; 3],
}

/// See [`sysctl.h`](https://opensource.apple.com/source/xnu/xnu-344/bsd/sys/sysctl.h).
#[repr(C)]
pub(crate) struct eproc {
    /// Address of proc. We just cheat and use a c_void pointer since we aren't
    /// using this.
    pub e_paddr: *mut c_void,

    /// Session pointer.  We just cheat and use a c_void pointer since we aren't
    /// using this.
    pub e_sess: *mut c_void,

    /// Process credentials
    pub e_pcred: pcred,

    /// Current credentials
    pub e_ucred: xucred,

    /// Address space
    pub e_vm: vmspace,

    /// Parent process ID
    pub e_ppid: pid_t,

    /// Process group ID
    pub e_pgid: pid_t,

    /// Job control counter
    pub e_jobc: c_short,

    /// Controlling tty dev
    pub e_tdev: dev_t,

    /// tty process group id
    pub e_tpgid: pid_t,

    /// tty session pointer.  We just cheat and use a c_void pointer since we
    /// aren't using this.
    pub e_tsess: *mut c_void,

    /// wchan message
    pub e_wmesg: [c_char; WMESGLEN + 1],

    /// text size
    pub e_xsize: segsz_t,

    /// text rss
    pub e_xrssize: c_short,

    /// text references
    pub e_xccount: c_short,

    pub e_xswrss: c_short,

    pub e_flag: c_long,

    /// short setlogin() name
    pub e_login: [c_char; COMAPT_MAXLOGNAME],

    pub e_spare: [c_long; 4],
}

/// Obtains the [`kinfo_proc`] given a process PID.
///
/// Based on the implementation from [heim](https://github.com/heim-rs/heim/blob/master/heim-process/src/sys/macos/bindings/process.rs#L235).
pub(crate) fn kinfo_process(pid: Pid) -> Result<kinfo_proc> {
    let mut name: [i32; 4] = [CTL_KERN, KERN_PROC, KERN_PROC_PID, pid];
    let mut size = mem::size_of::<kinfo_proc>();
    let mut info = mem::MaybeUninit::<kinfo_proc>::uninit();

    // SAFETY: libc binding, we assume all arguments are valid.
    let result = unsafe {
        libc::sysctl(
            name.as_mut_ptr(),
            4,
            info.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };

    if result < 0 {
        bail!("failed to get process for pid {pid}");
    }

    // sysctl succeeds but size is zero, happens when process has gone away
    if size == 0 {
        bail!("failed to get process for pid {pid}");
    }

    // SAFETY: info is initialized if result succeeded and returned a non-negative
    // result. If sysctl failed, it returns -1 with errno set.
    //
    // Source: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/sysctl.3.html
    unsafe { Ok(info.assume_init()) }
}

#[cfg(test)]
mod test {
    use std::mem;

    use super::*;

    /// A quick test to ensure that things are sized correctly.
    #[test]
    fn test_struct_sizes() {
        assert_eq!(mem::size_of::<p_st1>(), 16);
        assert_eq!(mem::align_of::<p_st1>(), 8);

        assert_eq!(mem::size_of::<pcred>(), 104);
        assert_eq!(mem::align_of::<pcred>(), 8);

        assert_eq!(mem::size_of::<vmspace>(), 64);
        assert_eq!(mem::align_of::<vmspace>(), 8);

        assert_eq!(mem::size_of::<extern_proc>(), 296);
        assert_eq!(mem::align_of::<extern_proc>(), 8);

        assert_eq!(mem::size_of::<eproc>(), 376);
        assert_eq!(mem::align_of::<eproc>(), 8);

        assert_eq!(mem::size_of::<kinfo_proc>(), 672);
        assert_eq!(mem::align_of::<kinfo_proc>(), 8);
    }
}
