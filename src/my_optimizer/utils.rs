// utilities of optimization

use super::ObjectiveFunction;
use anyhow::Error;
use once_cell::sync::Lazy;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{error, info};
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
    pub format: LogFormat,
}

impl<P: AsRef<Path>> FileLogConfig<P> {
    pub fn prepare_writer(&self) -> (tracing_appender::non_blocking::NonBlocking, WorkerGuard) {
        let file_appender = rolling::daily(&self.directory, &self.prefix);
        tracing_appender::non_blocking(file_appender)
    }
}

pub struct TracingConfig<'a, P: AsRef<Path>> {
    pub default_level: &'a str,
    pub console_format: LogFormat,
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
                match self.console_format {
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

                match (self.console_format, file_cfg.format) {
                    (LogFormat::PlainText, LogFormat::PlainText) => {
                        base_registry
                            .with(
                                fmt::layer()
                                    .with_writer(io::stdout)
                                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
                            )
                            .with(fmt::layer().with_writer(non_blocking))
                            .init();
                    }
                    (LogFormat::PlainText, LogFormat::Json) => {
                        base_registry
                            .with(
                                fmt::layer()
                                    .with_writer(io::stdout)
                                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
                            )
                            .with(fmt::layer().json().with_writer(non_blocking))
                            .init();
                    }
                    (LogFormat::Json, LogFormat::PlainText) => {
                        base_registry
                            .with(
                                fmt::layer()
                                    .json()
                                    .with_writer(io::stdout)
                                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
                            )
                            .with(fmt::layer().with_writer(non_blocking))
                            .init();
                    }
                    (LogFormat::Json, LogFormat::Json) => {
                        base_registry
                            .with(
                                fmt::layer()
                                    .json()
                                    .with_writer(io::stdout)
                                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
                            )
                            .with(fmt::layer().json().with_writer(non_blocking))
                            .init();
                    }
                }
                Some(guard)
            }
        }
    }
}

static PROGRESS_COUNTER: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));

pub fn update_progress(total: Option<usize>, step_size: usize) {
    let current = increment_progress_counter_by(1);

    if current % step_size == 0 {
        let percent = total.map(|t| (current as f64 / t as f64) * 100.0);
        if let Some(p) = percent {
            info!(
                current = current,
                total = total,
                "Progress update ({:.1}%):",
                p
            );
        } else {
            info!(current = current, total = total, "Progress update:");
        }
    }
}

pub fn increment_progress_counter_by(inc: usize) -> usize {
    PROGRESS_COUNTER.fetch_add(inc, Ordering::Relaxed) + inc
}

// reset counter at start of a new optimization or exploration
pub fn reset_progress_counter() {
    PROGRESS_COUNTER.store(0, Ordering::Relaxed);
}

// thread safe error handling
#[derive(Clone, Default)]
pub struct SharedError {
    inner: Arc<Mutex<Option<Error>>>,
}

impl SharedError {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    // save first error, discard any successive errors (with log)
    pub fn set_if_empty(&self, err: Error) {
        let mut guard = self.inner.lock().expect("SharedError lock poisoned.");
        if guard.is_none() {
            *guard = Some(err);
        } else {
            tracing::trace!("Discarded error because one was already set");
        }
    }

    pub fn is_set(&self) -> bool {
        self.inner
            .lock()
            .expect("SharedError lock poisoned.")
            .is_some()
    }

    pub fn take(&self) -> Option<Error> {
        self.inner
            .lock()
            .expect("SharedError lock poisoned.")
            .take()
    }
}

// execute evaluate and config conversion with shared error
pub fn evaluate_with_shared_error<F: ObjectiveFunction>(
    objective: &F,
    params: &[f64],
    error_slot: &SharedError,
) -> Option<f64> {
    if error_slot.is_set() {
        return None;
    }

    let config = match F::Config::try_from(params) {
        Ok(c) => c,
        Err(e) => {
            error!(
                ?params,
                error = %e,
                "Parameter conversion failed, aborting..."
            );
            error_slot.set_if_empty(e);
            return None;
        }
    };

    match objective.evaluate(config) {
        Ok(score) => Some(score),
        Err(e) => {
            error!(
                ?params,
                error = %e,
                "Evaluation failed, aborting..."
            );
            error_slot.set_if_empty(e);
            None
        }
    }
}
