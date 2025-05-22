// Random Search explorer

use super::{
    evaluate_with_shared_error, Candidate, Explorer, ObjectiveFunction, ParamDescriptor,
    Population, PopulationSaver, ProgressReporter, SharedError, SharedPopulation,
};
use anyhow::Context;
use rayon::prelude::*;
use tracing::{debug, info, span, Level};

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
        );
        let shared_error = SharedError::new();

        (0..self.iterations).into_par_iter().for_each(|_| {
            if shared_error.is_set() {
                return;
            }
            let iter_span = span!(Level::DEBUG, "Iteration");
            let _iter_enter = iter_span.enter();

            let mut rng = rand::thread_rng();
            let params: Vec<f64> = param_bounds
                .iter()
                .map(|pb| pb.rng_sample(&mut rng))
                .collect();

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
