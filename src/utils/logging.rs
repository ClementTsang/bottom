pub fn init_logger() -> Result<(), fern::InitError> {
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
		.level(if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Info })
		.chain(fern::log_file("debug.log")?)
		.apply()?;

	Ok(())
}
