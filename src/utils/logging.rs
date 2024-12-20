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
            log::error!($($x)*)
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::warn!($($x)*)
        }
    };
}

#[macro_export]
macro_rules! info {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::info!($($x)*)
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::debug!($($x)*)
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::trace!($($x)*)
        }
    };
}

#[macro_export]
macro_rules! log {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::log!(log::Level::Trace, $($x)*)
        }
    };
    ($level:expr, $($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::log!($level, $($x)*)
        }
    };
}

#[cfg(test)]
mod test {

    #[cfg(feature = "logging")]
    #[test]
    fn test_logging_macros() {
        super::init_logger(log::LevelFilter::Trace, None)
            .expect("initializing the logger should succeed");

        error!("This is an error.");
        warn!("This is a warning.");
        info!("This is an info.");
        debug!("This is a debug.");
        info!("This is a trace.");
    }
}
