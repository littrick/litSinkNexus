use clap::Parser;
use lit_sink_nexus::{
    app::{AppConfig, Application},
    init_i18n,
};
use std::{path::PathBuf, sync::OnceLock};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt, prelude::*};

/// Command line arguments
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, value_name = "DIR", default_value = "logs")]
    log: PathBuf,

    #[arg(short, long, value_name = "FILE", default_value = "config.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    init_logger(&cli);
    init_i18n();

    Application::run(AppConfig::parse_or_default(cli.config)).unwrap();
}

fn init_logger(cli: &Cli) {
    let subscriber = Registry::default().with(fmt::layer());
    static LOGGER_WORKER: OnceLock<WorkerGuard> = OnceLock::new();

    let appender = RollingFileAppender::builder()
        .rotation(Rotation::WEEKLY)
        .filename_suffix("app.log")
        .build(&cli.log)
        .unwrap();

    let (non_blocking, guard) = tracing_appender::non_blocking(appender);

    LOGGER_WORKER.set(guard).unwrap();
    let layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_filter(EnvFilter::new("debug"));

    subscriber.with(layer).init();
}
