// Random Search explorer

use super::{
    evaluate_with_shared_error, Candidate, Explorer, ObjectiveFunction, ParamDescriptor,
    Population, PopulationSaver, ProgressReporter, SharedError, SharedPopulation, generate_random_params
};
use anyhow::Context;
use rayon::prelude::*;
use tracing::{debug, error, info, span, Level};

pub struct RandomSearch {
    pub iterations: usize,
    pub population_saver: Option<PopulationSaver>,
}

impl ProgressReporter for RandomSearch {
    fn get_estimate_of_cycles(&self, _param_bounds: &[ParamDescriptor]) -> anyhow::Result<usize> {
        Ok(self.iterations)
    }
}

impl Explorer for RandomSearch {
    fn explore<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
        population_size: usize,
    ) -> anyhow::Result<Population> {
        let search_span = span!(Level::INFO, "RandomSearch", iterations = self.iterations);
        let _enter = search_span.enter();

        info!("Starting Random Search with {} iterations", self.iterations);

        // Shared Population and Error
        let shared_population = SharedPopulation::new(
            Population::new(population_size),
            self.population_saver.clone(),
            None,
        );
        let shared_error = SharedError::new();

        (0..self.iterations).into_par_iter().for_each(|_| {
            if shared_error.is_set() {
                return;
            }
            let iter_span = span!(Level::DEBUG, "Iteration");
            let _iter_enter = iter_span.enter();

            let params = match generate_random_params(param_bounds)
            {
                Ok(params) => params,
                Err(err) => {
                    error!(error = ?err, "Failed to sample parameters");
                    shared_error.set_if_empty(err);
                    return;
                }
            };

            if let Some(score) = evaluate_with_shared_error(objective, &params, &shared_error) {
                debug!(?params, score, "Evaluated random generated candidate");

                shared_population.insert(Candidate { params, score }, param_bounds, &shared_error);
            }
        });

        if let Some(err) = shared_error.take() {
            return Err(err);
        }

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
