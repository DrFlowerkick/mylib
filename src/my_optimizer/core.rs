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
pub trait Explorer<TS>: ProgressReporter {
    fn explore<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
        population_size: usize, // Top-N results
    ) -> anyhow::Result<Population<TS>>
    where
        TS: ToleranceSettings;
}

// common trait for all optimizer
pub trait Optimizer<TS>: ProgressReporter {
    fn optimize<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
        population_size: usize,
    ) -> anyhow::Result<Population<TS>>
    where
        TS: ToleranceSettings;
}

pub trait ToleranceSettings:
    PartialEq + Eq + Clone + Default + Send + Sync + std::fmt::Debug
{
    // tolerance of numeric comparing and noise generation
    fn epsilon() -> f64;

    // decimal precision for discreet values or hashing
    fn precision() -> usize;
}

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct DefaultTolerance;

impl ToleranceSettings for DefaultTolerance {
    fn epsilon() -> f64 {
        1e-8
    }
    fn precision() -> usize {
        8
    }
}
