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
pub use parameters::{ParamBound, ParamDescriptor, generate_random_params};
pub use population::{
    Candidate, CsvConversion, Population, PopulationSaver, SharedPopulation, load_population,
    save_population,
};
pub use random_search::RandomSearch;
pub use schedule::{
    ConstantSchedule, DecaySchedule, ExponentialSchedule, LinearSchedule, Schedule, SigmoidSchedule,
};
pub use trace_analysis::{
    ClampStats, ClampedLogEntry, DefaultLogEntry, EvoFields, EvoSpan, LogEntryParser, MutationKey,
    MutationParentAndOffspring, MutationStats, analyze_clamp_events, analyze_evo_log_entries,
    read_log_file, read_logs_from_dir,
};
pub use utils::{
    FileLogConfig, LogFormat, SharedError, TracingConfig, evaluate_with_shared_error,
    increment_progress_counter_by, reset_progress_counter, update_progress,
};
