use std::env;
use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};

pub fn setup_logging() {
    let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "plain".to_string());
    let env_filter = EnvFilter::from_default_env()
        .add_directive(Level::INFO.into())
        .add_directive("aws_config=off".parse().unwrap());

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_ansi(true)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_thread_names(false);

    if log_format == "json" {
        subscriber.json().init();
    } else {
        subscriber.init();
    }
}
