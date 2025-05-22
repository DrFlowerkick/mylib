// traits & type definitions

use super::{ParamDescriptor, Population};

// trait of target function
pub trait ObjectiveFunction {
    type Config: for<'a> TryFrom<&'a [f64], Error = anyhow::Error>;

    fn evaluate(&self, config: Self::Config) -> anyhow::Result<f64>;
}

pub trait ProgressReporter {
    // returns estimation of number of steps of exploration or optimization
    fn get_estimate_of_cycles(&self, param_bounds: &[ParamDescriptor]) -> usize;
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

// trait to control dynamic parent selection over sequences of optimization
pub trait SelectionSchedule: Sync {
    // calculates the current fraction of selection of population (between 0.0 and 1.0)
    fn selection_fraction(&self, current_generation: usize, total_generations: usize) -> f64 {
        // default: linear interpolation
        let progress = current_generation as f64 / total_generations as f64;
        self.end_fraction() + (self.start_fraction() - self.end_fraction()) * (1.0 - progress)
    }

    fn estimate_evaluations(&self, total_generations: usize, population_size: usize) -> usize {
        let mut total = 0;
        for gen in 0..total_generations {
            let fraction = self
                .selection_fraction(gen, total_generations)
                .clamp(0.0, 1.0);
            let parents = (population_size as f64 * fraction).ceil() as usize;
            total += parents;
        }
        total
    }

    fn start_fraction(&self) -> f64 {
        1.0
    } // default: start with 100% of population
    fn end_fraction(&self) -> f64 {
        0.1
    } // default: end with top 10% of population
}

// linear selection (default)
pub struct LinearSchedule {
    pub start: f64,
    pub end: f64,
}

impl SelectionSchedule for LinearSchedule {
    fn start_fraction(&self) -> f64 {
        self.start
    }
    fn end_fraction(&self) -> f64 {
        self.end
    }
}

// exponential selection (e.g. for faster selection pressure)
pub struct ExponentialSchedule {
    pub start: f64,
    pub end: f64,
    pub exponent: f64, // e.g. 2.0 for quadratic, >1.0 für stronger pressure
}

impl SelectionSchedule for ExponentialSchedule {
    fn selection_fraction(&self, current_generation: usize, total_generations: usize) -> f64 {
        let progress = current_generation as f64 / total_generations as f64;
        self.end + (self.start - self.end) * (1.0 - progress.powf(self.exponent))
    }

    fn start_fraction(&self) -> f64 {
        self.start
    }
    fn end_fraction(&self) -> f64 {
        self.end
    }
}
