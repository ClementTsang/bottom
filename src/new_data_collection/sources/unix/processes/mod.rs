pub mod user_table;

/// A UNIX process ID.
#[cfg(target_family = "unix")]
pub type Pid = libc::pid_t;
