// Random Search explorer

use super::{
    evaluate_with_shared_error, generate_random_params, Explorer, ObjectiveFunction,
    ParamDescriptor, Population, PopulationSaver, ProgressReporter, SharedError, SharedPopulation,
    ToleranceSettings,
};
use anyhow::Context;
use rayon::prelude::*;
use tracing::{error, info, span, Level};

pub struct RandomSearch<TS: ToleranceSettings> {
    pub iterations: usize,
    pub population_saver: Option<PopulationSaver>,
    pub phantom: std::marker::PhantomData<TS>,
}

impl<TS: ToleranceSettings> ProgressReporter for RandomSearch<TS> {
    fn get_estimate_of_cycles(&self, _param_bounds: &[ParamDescriptor]) -> anyhow::Result<usize> {
        Ok(self.iterations)
    }
}

impl<TS: ToleranceSettings> Explorer<TS> for RandomSearch<TS> {
    fn explore<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
        population_size: usize,
    ) -> anyhow::Result<Population<TS>> {
        let search_span = span!(Level::INFO, "RandomSearch", iterations = self.iterations);
        let _enter = search_span.enter();

        info!("Starting Random Search with {} iterations", self.iterations);

        // Shared Population and Error
        let shared_population = SharedPopulation::new(
            Population::new(population_size),
            self.population_saver.clone(),
        );
        let shared_error = SharedError::new();

        (0..self.iterations).into_par_iter().for_each(|_| {
            if shared_error.is_set() {
                return;
            }
            let iter_span = span!(Level::DEBUG, "Iteration");
            let _iter_enter = iter_span.enter();

            let params = match generate_random_params(param_bounds) {
                Ok(params) => params,
                Err(err) => {
                    error!(error = ?err, "Failed to sample parameters");
                    shared_error.set_if_empty(err);
                    return;
                }
            };

            if let Some(candidate) = evaluate_with_shared_error(objective, &params, &shared_error) {
                shared_population.insert(candidate, param_bounds, &shared_error);
            }
        });

        if let Some(err) = shared_error.take() {
            return Err(err);
        }

        shared_population.lock().save_population(param_bounds)?;
        let population = shared_population.take();

        info!(
            "Random Search completed. Best Score: {:.3}",
            population
                .best()
                .map(|c| c.score)
                .context("Empty population")?
        );

        Ok(population)
    }
}
