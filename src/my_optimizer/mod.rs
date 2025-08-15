// library for generic optimization functions.

pub mod core;
pub mod evolutionary;
pub mod grid_search;
pub mod parameters;
pub mod population;
pub mod random_search;
pub mod schedule;
pub mod trace_analysis;
pub mod utils;

pub use self::core::{
    DefaultTolerance, Explorer, ObjectiveFunction, Optimizer, ProgressReporter, ToleranceSettings,
};
pub use evolutionary::EvolutionaryOptimizer;
pub use grid_search::GridSearch;
pub use parameters::{generate_random_params, ParamBound, ParamDescriptor};
pub use population::{
    load_population, save_population, Candidate, CsvConversion, Population, PopulationSaver,
    SharedPopulation,
};
pub use random_search::RandomSearch;
pub use schedule::{
    ConstantSchedule, DecaySchedule, ExponentialSchedule, LinearSchedule, Schedule, SigmoidSchedule,
};
pub use trace_analysis::{
    analyze_clamp_events, analyze_evo_log_entries, read_log_file, read_logs_from_dir, ClampStats,
    ClampedLogEntry, DefaultLogEntry, EvoFields, EvoSpan, LogEntryParser, MutationKey,
    MutationParentAndOffspring, MutationStats,
};
pub use utils::{
    evaluate_with_shared_error, increment_progress_counter_by, reset_progress_counter,
    update_progress, FileLogConfig, LogFormat, SharedError, TracingConfig,
};
