// utilities of optimization

use super::{CsvConversion, ObjectiveFunction, ParamDescriptor, Population};
use anyhow::Error;
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::{fmt::Display, io};
use tracing::{error, info, Level};
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
                            .with(
                                fmt::layer()
                                    .with_writer(io::stdout)
                                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
                            )
                            .with(fmt::layer().with_writer(non_blocking))
                            .init();
                    }
                    LogFormat::Json => {
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

static SAVE_POPULATION_COUNTER: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));

// helper to save population after N cycles
#[derive(Debug, Clone)]
pub struct PopulationSaver {
    pub file_path: PathBuf,
    pub step_size: usize,
    pub precision: usize,
}

impl PopulationSaver {
    pub fn save_population(&self, population: &Population, param_bounds: &[ParamDescriptor]) {
        let current = SAVE_POPULATION_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;

        if current % self.step_size == 0 {
            let param_names = param_bounds
                .iter()
                .map(|pd| pd.name.as_str())
                .collect::<Vec<_>>();
            save_population(population, &param_names, &self.file_path, self.precision);
        }
    }
}

pub fn save_population<P: AsRef<Path>>(
    population: &Population,
    param_names: &[impl Display],
    filename: P,
    precision: usize,
) {
    let path = filename.as_ref();
    let file = File::create(path).expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    if !param_names.is_empty() {
        let header = param_names
            .iter()
            .map(|name| name.to_string())
            .collect::<Vec<String>>()
            .join(",");

        writeln!(writer, "{},average_score", header).expect("Unable to write header to file");
    }

    writeln!(writer, "{}", population.to_csv(precision))
        .expect("Unable to write population to file");

    log_or_print(&format!("Population written to {}", path.display()));
}

pub fn load_population<P: AsRef<Path>>(
    filename: P,
    has_headers: bool,
) -> Option<(Population, Vec<String>)> {
    let path = filename.as_ref();
    let csv = std::fs::read_to_string(path).expect("Unable to read from file");

    let (parameter_names, csv) = if has_headers {
        let (parameter_names, csv) = csv.split_once('\n')?;
        let mut parameter_names = parameter_names
            .split(',')
            .map(|pn| pn.to_string())
            .collect::<Vec<_>>();
        // remove "average_score" at the end
        parameter_names.pop();
        (parameter_names, csv)
    } else {
        (vec![], csv.as_str())
    };

    let population = Population::from_csv(csv);

    log_or_print(&format!("Results written to {}", path.display()));

    population.map(|p| (p, parameter_names))
}

fn log_or_print(message: &str) {
    if tracing::event_enabled!(Level::INFO) {
        tracing::info!("{}", message);
    } else {
        println!("{}", message);
    }
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
        let mut guard = self.inner.lock().unwrap();
        if guard.is_none() {
            *guard = Some(err);
        } else {
            tracing::trace!("Discarded error because one was already set");
        }
    }

    pub fn is_set(&self) -> bool {
        self.inner.lock().unwrap().is_some()
    }

    pub fn take(&self) -> Option<Error> {
        self.inner.lock().unwrap().take()
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
            error_slot.set_if_empty(e.into());
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
