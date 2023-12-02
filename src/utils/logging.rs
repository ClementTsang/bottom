#[cfg(feature = "logging")]
pub static OFFSET: once_cell::sync::Lazy<time::UtcOffset> = once_cell::sync::Lazy::new(|| {
    use time::util::local_offset::Soundness;

    // SAFETY: We only invoke this once, quickly, and it should be invoked in a single-thread context.
    // We also should only ever hit this logging at all in a debug context which is generally fine,
    // release builds should have this logging disabled entirely for now.
    unsafe {
        // XXX: If we ever DO add general logging as a release feature, evaluate this again and whether this is
        // something we want enabled in release builds! What might be safe is falling back to the non-set-soundness
        // mode when specifically using certain feature flags (e.g. dev-logging feature enables this behaviour).

        time::util::local_offset::set_soundness(Soundness::Unsound);
        let res = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
        time::util::local_offset::set_soundness(Soundness::Sound);

        res
    }
});

#[cfg(feature = "logging")]
pub fn init_logger(
    min_level: log::LevelFilter, debug_file_name: &std::ffi::OsStr,
) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let offset_time = {
                let utc = time::OffsetDateTime::now_utc();
                utc.checked_to_offset(*OFFSET).unwrap_or(utc)
            };

            out.finish(format_args!(
                "{}[{}][{}] {}",
                offset_time
                    .format(&time::macros::format_description!(
                        // The weird "[[[" is because we need to escape a bracket ("[[") to show one "[".
                        // See https://time-rs.github.io/book/api/format-description.html
                        "[[[year]-[month]-[day]][[[hour]:[minute]:[second][subsecond digits:9]]"
                    ))
                    .unwrap(),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(min_level)
        .chain(fern::log_file(debug_file_name)?)
        .apply()?;

    Ok(())
}

#[macro_export]
macro_rules! c_debug {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::debug!($($x)*)
        }
    };
}

#[macro_export]
macro_rules! c_error {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::error!($($x)*)
        }
    };
}

#[macro_export]
macro_rules! c_info {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::info!($($x)*)
        }
    };
}

#[macro_export]
macro_rules! c_log {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::log!($($x)*)
        }
    };
}

#[macro_export]
macro_rules! c_trace {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::trace!($($x)*)
        }
    };
}

#[macro_export]
macro_rules! c_warn {
    ($($x:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::warn!($($x)*)
        }
    };
}
