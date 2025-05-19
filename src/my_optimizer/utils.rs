// utilities of optimization

use std::io;
use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*, Registry};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    PlainText,
    Json,
}

pub struct FileLogConfig<P: AsRef<Path>> {
    pub directory: P,
    pub prefix: String,
}

impl<P: AsRef<Path>> FileLogConfig<P> {
    pub fn prepare_writer(&self) -> (tracing_appender::non_blocking::NonBlocking, WorkerGuard) {
        let file_appender = rolling::daily(&self.directory, &self.prefix);
        tracing_appender::non_blocking(file_appender)
    }
}

pub struct TracingConfig<'a, P: AsRef<Path>> {
    pub default_level: &'a str,
    pub format: LogFormat,
    pub file_log: Option<FileLogConfig<P>>,
}

impl<'a, P: AsRef<Path>> TracingConfig<'a, P> {
    /// Initialize tracing with configurable console and optional file logging.
    /// Returns an optional `WorkerGuard` to ensure file logs are properly flushed at shutdown.
    pub fn init(self) -> Option<WorkerGuard> {
        let env_filter = EnvFilter::from_default_env()
            .add_directive(self.default_level.parse().expect("Invalid log level"));
        let base_registry = Registry::default().with(env_filter);

        match self.file_log {
            None => {
                // console only
                match self.format {
                    LogFormat::PlainText => {
                        base_registry
                            .with(fmt::layer().with_writer(io::stdout))
                            .init();
                    }
                    LogFormat::Json => {
                        base_registry
                            .with(fmt::layer().json().with_writer(io::stdout))
                            .init();
                    }
                }
                None
            }
            Some(file_cfg) => {
                let (non_blocking, guard) = file_cfg.prepare_writer();

                match self.format {
                    LogFormat::PlainText => {
                        base_registry
                            .with(fmt::layer().with_writer(io::stdout))
                            .with(fmt::layer().with_writer(non_blocking))
                            .init();
                    }
                    LogFormat::Json => {
                        base_registry
                            .with(fmt::layer().json().with_writer(io::stdout))
                            .with(fmt::layer().json().with_writer(non_blocking))
                            .init();
                    }
                }
                Some(guard)
            }
        }
    }
}
