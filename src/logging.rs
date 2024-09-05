use log::LevelFilter;

pub fn setup(level: LevelFilter) {
    env_logger::Builder::from_default_env()
        .filter(None, LevelFilter::Warn)
        .filter(Some("monitorhosts"), level)
        .format_module_path(false)
        .format_target(false)
        .init();
}