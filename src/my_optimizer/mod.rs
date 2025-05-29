// library for generic optimization functions.

pub mod core;
pub mod evolutionary;
pub mod grid_search;
pub mod parameters;
pub mod population;
pub mod random_search;
pub mod trace_analysis;
pub mod utils;
pub mod schedule;

pub use self::core::{
    Explorer,ObjectiveFunction, Optimizer, ProgressReporter,
};
pub use schedule::{
    Schedule, ConstantSchedule, LinearSchedule, ExponentialSchedule, DecaySchedule, SigmoidSchedule
};
pub use evolutionary::EvolutionaryOptimizer;
pub use grid_search::GridSearch;
pub use parameters::{ParamBound, ParamDescriptor};
pub use population::{
    load_population, save_population, Candidate, CsvConversion, Population, PopulationSaver,
    SharedPopulation,
};
pub use random_search::RandomSearch;
pub use trace_analysis::{analyze_clamps_from_dir, ClampStats};
pub use utils::{
    evaluate_with_shared_error, increment_progress_counter_by, reset_progress_counter,
    update_progress, FileLogConfig, LogFormat, SharedError, TracingConfig,
};
