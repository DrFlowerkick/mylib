// Random Search explorer

use super::{Candidate, Explorer, ObjectiveFunction, ParamBound, Population};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, span, Level};

pub struct RandomSearch {
    pub iterations: usize,
}

impl Explorer for RandomSearch {
    fn explore<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamBound],
        population_size: usize,
    ) -> Population {
        let search_span = span!(Level::INFO, "RandomSearch", iterations = self.iterations);
        let _enter = search_span.enter();

        info!("Starting Random Search with {} iterations", self.iterations);

        let shared_population = Arc::new(Mutex::new(Population::new(population_size)));
        (0..self.iterations).into_par_iter().for_each(|_| {
            let iter_span = span!(Level::DEBUG, "Iteration");
            let _iter_enter = iter_span.enter();

            let mut rng = rand::thread_rng();
            let params: Vec<f64> = param_bounds
                .iter()
                .map(|pb| pb.rng_sample(&mut rng))
                .collect();

            debug!(?params, "Generated random parameters");

            let score = objective.evaluate(&params);
            debug!(score, "Evaluated candidate");

            let mut pop = shared_population.lock().expect("Population lock poisoned.");
            pop.insert(Candidate { params, score });
        });

        let population = Arc::try_unwrap(shared_population)
            .expect("Expected sole ownership of Arc")
            .into_inner()
            .expect("Population lock poisoned.");

        info!(
            "Random Search completed. Best Score: {:.3}",
            population.best().map(|c| c.score).unwrap_or(-1.0)
        );

        population
    }
}
