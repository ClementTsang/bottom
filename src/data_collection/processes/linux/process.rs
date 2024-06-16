//! Linux process code for getting process data via `/proc/`.
//! Based on the [procfs](https://github.com/eminence/procfs) crate.

use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::PathBuf,
    sync::OnceLock,
};

use anyhow::anyhow;
use libc::uid_t;
use rustix::{
    fd::OwnedFd,
    fs::{Mode, OFlags},
    path::Arg,
};

use crate::data_collection::processes::Pid;

static PAGESIZE: OnceLock<u64> = OnceLock::new();

#[inline]
fn next_part<'a>(iter: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, io::Error> {
    iter.next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
}

/// A wrapper around the data in `/proc/<PID>/stat`. For documentation, see
/// [here](https://man7.org/linux/man-pages/man5/proc.5.html).
///
/// Note this does not necessarily get all fields, only the ones we use in
/// bottom.
pub(crate) struct Stat {
    /// The filename of the executable without parentheses.
    pub comm: String,

    /// The current process state, represented by a char.
    pub state: char,

    /// The parent process PID.
    pub ppid: Pid,

    /// The amount of time this process has been scheduled in user mode in clock
    /// ticks.
    pub utime: u64,

    /// The amount of time this process has been scheduled in kernel mode in
    /// clock ticks.
    pub stime: u64,

    /// The resident set size, or the number of pages the process has in real
    /// memory.
    pub rss: u64,

    /// The start time of the process, represented in clock ticks.
    pub start_time: u64,
}

impl Stat {
    #[inline]
    fn from_file(mut f: File, buffer: &mut String) -> anyhow::Result<Stat> {
        // Since this is just one line, we can read it all at once. However, since it
        // might have non-utf8 characters, we can't just use read_to_string.
        f.read_to_end(unsafe { buffer.as_mut_vec() })?;

        let line = buffer.to_string_lossy();
        let line = line.trim();

        let (comm, rest) = {
            let start_paren = line
                .find('(')
                .ok_or_else(|| anyhow!("start paren missing"))?;
            let end_paren = line.find(')').ok_or_else(|| anyhow!("end paren missing"))?;

            (
                line[start_paren + 1..end_paren].to_string(),
                &line[end_paren + 2..],
            )
        };

        let mut rest = rest.split(' ');
        let state = next_part(&mut rest)?
            .chars()
            .next()
            .ok_or_else(|| anyhow!("missing state"))?;
        let ppid: Pid = next_part(&mut rest)?.parse()?;

        // Skip 9 fields until utime (pgrp, session, tty_nr, tpgid, flags, minflt,
        // cminflt, majflt, cmajflt).
        let mut rest = rest.skip(9);
        let utime: u64 = next_part(&mut rest)?.parse()?;
        let stime: u64 = next_part(&mut rest)?.parse()?;

        // Skip 6 fields until starttime (cutime, cstime, priority, nice, num_threads,
        // itrealvalue).
        let mut rest = rest.skip(6);
        let start_time: u64 = next_part(&mut rest)?.parse()?;

        // Skip one field until rss (vsize)
        let mut rest = rest.skip(1);
        let rss: u64 = next_part(&mut rest)?.parse()?;

        Ok(Stat {
            comm,
            state,
            ppid,
            utime,
            stime,
            rss,
            start_time,
        })
    }

    /// Returns the Resident Set Size in bytes.
    #[inline]
    pub fn rss_bytes(&self) -> u64 {
        self.rss * PAGESIZE.get_or_init(|| rustix::param::page_size() as u64)
    }
}

/// A wrapper around the data in `/proc/<PID>/io`.
///
/// Note this does not necessarily get all fields, only the ones we use in
/// bottom.
pub(crate) struct Io {
    pub read_bytes: u64,
    pub write_bytes: u64,
}

impl Io {
    #[inline]
    fn from_file(f: File, buffer: &mut String) -> anyhow::Result<Io> {
        const NUM_FIELDS: u16 = 0; // Make sure to update this if you want more fields!
        enum Fields {
            ReadBytes,
            WriteBytes,
        }

        let mut read_fields = 0;
        let mut reader = BufReader::new(f);

        let mut read_bytes = 0;
        let mut write_bytes = 0;

        // This saves us from doing a string allocation on each iteration compared to
        // `lines()`.
        while let Ok(bytes) = reader.read_line(buffer) {
            if bytes > 0 {
                if buffer.is_empty() {
                    // Empty, no need to clear.
                    continue;
                }

                let mut parts = buffer.split_whitespace();

                if let Some(field) = parts.next() {
                    let curr_field = match field {
                        "read_bytes:" => Fields::ReadBytes,
                        "write_bytes:" => Fields::WriteBytes,
                        _ => {
                            buffer.clear();
                            continue;
                        }
                    };

                    if let Some(value) = parts.next() {
                        let value = value.parse::<u64>()?;
                        match curr_field {
                            Fields::ReadBytes => {
                                read_bytes = value;
                                read_fields += 1;
                            }
                            Fields::WriteBytes => {
                                write_bytes = value;
                                read_fields += 1;
                            }
                        }
                    }
                }

                // Quick short circuit if we have already read all the required fields.
                if read_fields == NUM_FIELDS {
                    break;
                }

                buffer.clear();
            } else {
                break;
            }
        }

        Ok(Io {
            read_bytes,
            write_bytes,
        })
    }
}

/// A wrapper around a Linux process operations in `/proc/<PID>`.
///
/// Core documentation based on [proc's manpages](https://man7.org/linux/man-pages/man5/proc.5.html).
pub(crate) struct Process {
    pub pid: Pid,
    pub uid: Option<uid_t>,
    pub stat: Stat,
    pub io: anyhow::Result<Io>,
    pub cmdline: anyhow::Result<Vec<String>>,
}

#[inline]
fn reset(root: &mut PathBuf, buffer: &mut String) {
    root.pop();
    buffer.clear();
}

impl Process {
    /// Creates a new [`Process`] given a `/proc/<PID>` path. This may fail if
    /// the process no longer exists or there are permissions issues.
    ///
    /// Note that this pre-allocates fields on **creation**! As such, some data
    /// might end up "outdated" depending on when you call some of the
    /// methods. Therefore, this struct is only useful for either fields
    /// that are unlikely to change, or are short-lived and
    /// will be discarded quickly.
    pub(crate) fn from_path(pid_path: PathBuf) -> anyhow::Result<Process> {
        // TODO: Pass in a buffer vec/string to share?

        let fd = rustix::fs::openat(
            rustix::fs::CWD,
            &pid_path,
            OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )?;

        let pid = pid_path
            .as_path()
            .components()
            .last()
            .and_then(|s| s.to_string_lossy().parse::<Pid>().ok())
            .or_else(|| {
                rustix::fs::readlinkat(rustix::fs::CWD, &pid_path, vec![])
                    .ok()
                    .and_then(|s| s.to_string_lossy().parse::<Pid>().ok())
            })
            .ok_or_else(|| anyhow!("PID for {pid_path:?} was not found"))?;

        let uid = {
            let metadata = rustix::fs::fstat(&fd);
            match metadata {
                Ok(md) => Some(md.st_uid),
                Err(_) => None,
            }
        };

        let mut root = pid_path;
        let mut buffer = String::new();

        // NB: Whenever you add a new stat, make sure to pop the root and clear the
        // buffer!
        let stat =
            open_at(&mut root, "stat", &fd).and_then(|file| Stat::from_file(file, &mut buffer))?;
        reset(&mut root, &mut buffer);

        let cmdline = cmdline(&mut root, &fd, &mut buffer);
        reset(&mut root, &mut buffer);

        let io = open_at(&mut root, "io", &fd).and_then(|file| Io::from_file(file, &mut buffer));

        Ok(Process {
            pid,
            uid,
            stat,
            io,
            cmdline,
        })
    }
}

#[inline]
fn cmdline(root: &mut PathBuf, fd: &OwnedFd, buffer: &mut String) -> anyhow::Result<Vec<String>> {
    open_at(root, "cmdline", fd)
        .map(|mut file| file.read_to_string(buffer))
        .map(|_| {
            buffer
                .split('\0')
                .filter_map(|s| {
                    if !s.is_empty() {
                        Some(s.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .map_err(Into::into)
}

/// Opens a path. Note that this function takes in a mutable root - this will
/// mutate it to avoid allocations. You probably will want to pop the most
/// recent child after if you need to use the buffer again.
#[inline]
fn open_at(root: &mut PathBuf, child: &str, fd: &OwnedFd) -> anyhow::Result<File> {
    root.push(child);
    let new_fd = rustix::fs::openat(fd, &*root, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())?;

    Ok(File::from(new_fd))
}
