//! This file is meant to house (OS specific) implementations on how to kill processes.

#[cfg(target_os = "windows")]
pub(crate) mod windows;

#[cfg(target_family = "unix")]
pub(crate) mod unix;
