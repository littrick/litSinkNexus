use tracing_perfetto::PerfettoLayer;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self},
    prelude::*,
};
use windows_play::app::Application;

fn main() {
    let perfetto_layer = PerfettoLayer::new(std::sync::Mutex::new(
        std::fs::File::create("test.pftrace").unwrap(),
    ));

    let fmt_layer = fmt::layer().compact();
    let filter_layer = EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(perfetto_layer)
        .with(fmt_layer)
        .with(filter_layer)
        .init();

    Application::run().unwrap();
}
