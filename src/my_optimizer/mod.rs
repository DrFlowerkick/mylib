// library for generic optimization functions.

pub mod coarse_grid_search;
/// Example usage of tracing
/// tracing setup in main.rs for console (env controlled)
/*
use tracing_subscriber::FmtSubscriber;

fn main() {
    // default console output, (use RUST_LOG to set tracing level)
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG) // Standard-Log-Level
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    // start your optimizer
    run_optimizer();
}
*/

/// tracing setup for tracing file + console
/*
use tracing_subscriber::{fmt, EnvFilter};
use tracing_appender::rolling;

fn main() {
    // tracing file, daily rotation
    let file_appender = rolling::daily("./logs", "optimizer.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = fmt::Subscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(non_blocking) // tracing output in file
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    // start your optimizer
    run_optimizer();
}
*/
pub mod core;
pub mod evolutionary;
pub mod random_search;

pub use self::core::{
    Candidate, Explorer, ObjectiveFunction, Optimizer, ParamBound, Population, SelectionSchedule,
};
pub use evolutionary::EvolutionaryOptimizer;
pub use random_search::RandomSearch;
