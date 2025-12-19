use tracing::log::*;

fn main() {
    let file_appender = tracing_appender::rolling::hourly("logs", "prefix.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    info!("This is an info message");
    debug!("This is a debug message");
    warn!("This is a warning message");
    error!("This is an error message");
    trace!("This is a trace message");
}
