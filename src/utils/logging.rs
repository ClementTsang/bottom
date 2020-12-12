#[cfg(feature = "fern")]
pub fn init_logger(
    min_level: log::LevelFilter, debug_file_name: &std::ffi::OsStr,
) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S:%f]"),
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
