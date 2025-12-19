use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use tracing::log::*;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, Registry, prelude::*};
use windows_services::Service;
fn main() {
    let _logger = init_logging("logs");

    let _servcie = Service::new()
        .can_pause()
        .can_stop()
        .can_fallback(|_service| {
            debug!("service fallback called");
            info!("fallback");
            warn!("service is falling back");
            error!("service encountered an error during fallback");
            trace!("service trace during fallback");
            sleep(Duration::from_secs(10));
        })
        .run(|_service, command| {
            info!("command: {command:?}");
        });
}

fn init_logging<P: AsRef<Path>>(dir: P) -> WorkerGuard {
    let rolling = RollingFileAppender::builder()
        .rotation(Rotation::WEEKLY)
        .filename_suffix("service.log")
        .build(dir)
        .unwrap();

    let (writer, guard) = tracing_appender::non_blocking(rolling);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(writer)
        .with_ansi(false)
        .with_filter(EnvFilter::new("debug"));

    let stdio = tracing_subscriber::fmt::layer().with_filter(EnvFilter::new("info"));

    Registry::default().with(file_layer).with(stdio).init();

    guard
}
