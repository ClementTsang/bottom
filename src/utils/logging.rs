#[cfg(feature = "fern")]
pub fn init_logger(
    min_level: log::LevelFilter, debug_file_name: &std::ffi::OsStr,
) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            // Note we aren't using local time since it only works on single-threaded processes.
            // If that ever does get patched in again, enable the "local-offset" feature.
            let offset = time::OffsetDateTime::now_utc();

            out.finish(format_args!(
                "{}[{}][{}] {}",
                offset
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
