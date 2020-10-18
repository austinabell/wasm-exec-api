use log::LevelFilter;

/// Parses RUST_LOG configuration, as well as initializing async logger.
pub fn setup_logger() {
    let mut logger_builder = pretty_env_logger::formatted_timed_builder();
    if let Ok(s) = ::std::env::var("RUST_LOG") {
        logger_builder.parse_filters(&s);
    } else {
        logger_builder.filter(None, LevelFilter::Info);
    }
    let logger = logger_builder.build();
    async_log::Logger::wrap(logger, || 12)
        .start(log::LevelFilter::Trace)
        .unwrap();
}
