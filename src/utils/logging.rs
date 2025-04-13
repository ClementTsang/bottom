#[cfg(feature = "logging")]
use std::sync::OnceLock;

#[cfg(feature = "logging")]
pub static OFFSET: OnceLock<time::UtcOffset> = OnceLock::new();

#[cfg(feature = "logging")]
pub fn init_logger(
    min_level: log::LevelFilter, debug_file_name: Option<&std::ffi::OsStr>,
) -> anyhow::Result<()> {
    let dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            let offset = OFFSET.get_or_init(|| {
                time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC)
            });

            let offset_time = {
                let utc = time::OffsetDateTime::now_utc();
                utc.checked_to_offset(*offset).unwrap_or(utc)
            };

            out.finish(format_args!(
                "{}[{}][{}] {}",
                offset_time
                    .format(&time::macros::format_description!(
                        // The weird "[[[" is because we need to escape a bracket ("[[") to show
                        // one "[". See https://time-rs.github.io/book/api/format-description.html
                        "[[[year]-[month]-[day]][[[hour]:[minute]:[second][subsecond digits:9]]"
                    ))
                    .unwrap(),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(min_level);

    if let Some(debug_file_name) = debug_file_name {
        dispatch.chain(fern::log_file(debug_file_name)?).apply()?;
    } else {
        dispatch.chain(std::io::stdout()).apply()?;
    }

    Ok(())
}

#[macro_export]
macro_rules! error {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::error!($($x)*);
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::warn!($($x)*);
        }
    };
}

#[macro_export]
macro_rules! info {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::info!($($x)*);
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::debug!($($x)*);
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::trace!($($x)*);
        }
    };
}

#[macro_export]
macro_rules! log {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::log!(log::Level::Trace, $($x)*);
        }
    };
    ($level:expr, $($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::log!($level, $($x)*);
        }
    };
}

#[macro_export]
macro_rules! info_every_n_secs {
    ($n:expr, $($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            $crate::log_every_n_secs!(log::Level::Info, $n, $($x)*);
        }
    };
}

#[macro_export]
macro_rules! log_every_n_secs {
    ($level:expr, $n:expr, $($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            static LAST_LOG: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let since_last_log = LAST_LOG.load(std::sync::atomic::Ordering::Relaxed);
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("should be valid").as_secs();

            if now - since_last_log > $n {
                LAST_LOG.store(now, std::sync::atomic::Ordering::Relaxed);
                log::log!($level, $($x)*);
            }
        }
    };
}

#[cfg(test)]
mod test {
    #[cfg(feature = "logging")]
    /// We do this to ensure that the test logger is only initialized _once_ for
    /// things like the default test runner that run tests in the same process.
    ///
    /// This doesn't do anything if you use something like nextest, which runs
    /// a test-per-process, but that's fine.
    fn init_test_logger() {
        use std::sync::atomic::{AtomicBool, Ordering};

        static LOG_INIT: AtomicBool = AtomicBool::new(false);

        if LOG_INIT.load(Ordering::SeqCst) {
            return;
        }

        LOG_INIT.store(true, Ordering::SeqCst);
        super::init_logger(log::LevelFilter::Trace, None)
            .expect("initializing the logger should succeed");
    }

    #[cfg(feature = "logging")]
    #[test]
    fn test_logging_macros() {
        init_test_logger();

        error!("This is an error.");
        warn!("This is a warning.");
        info!("This is an info.");
        debug!("This is a debug.");
        info!("This is a trace.");
    }

    #[cfg(feature = "logging")]
    #[test]
    fn test_log_every_macros() {
        init_test_logger();

        info_every_n_secs!(10, "This is an info every 10 seconds.");
    }
}
