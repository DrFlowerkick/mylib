// traits & type definitions

use super::{ParamDescriptor, Population};

// trait of target function
pub trait ObjectiveFunction {
    type Config: for<'a> TryFrom<&'a [f64], Error = anyhow::Error>;

    fn evaluate(&self, config: Self::Config) -> anyhow::Result<f64>;
}

pub trait ProgressReporter {
    // returns estimation of number of steps of exploration or optimization
    fn get_estimate_of_cycles(&self, param_bounds: &[ParamDescriptor]) -> anyhow::Result<usize>;
}

// common trait for all explorer
pub trait Explorer: ProgressReporter {
    fn explore<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
        population_size: usize, // Top-N results
    ) -> anyhow::Result<Population>;
}

// common trait for all optimizer
pub trait Optimizer: ProgressReporter {
    fn optimize<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
        population_size: usize,
    ) -> anyhow::Result<Population>;
}