use std::{fs::File, path::PathBuf};

use clap::Parser;
use tracing_perfetto::PerfettoLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use windows_play::app::Application;

/// Command line arguments
#[derive(Debug, Parser)]
#[command(version, about = "aaa", long_about = None)]
struct Cli {
    /// Path to output perfetto trace file
    #[arg(long)]
    trace: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    rust_i18n::set_locale("zh-CN");

    if let Some(trace_path) = cli.trace {
        let perfetto_layer = PerfettoLayer::new(File::create(trace_path).unwrap());

        tracing_subscriber::registry()
            .with(perfetto_layer)
            .with(EnvFilter::from_default_env())
            .with(fmt::layer().compact())
            .init();
    } else {
        tracing_subscriber::fmt::init();
    }

    Application::run().unwrap();
}
