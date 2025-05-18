// Random Search Optimizer

use super::{Candidate, ObjectiveFunction, Optimizer, Population};
use rand::prelude::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub struct RandomSearch {
    pub iterations: usize,
}

impl Optimizer for RandomSearch {
    fn optimize<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[(f64, f64)],
        population_size: usize,
    ) -> Population {
        let shared_population = Arc::new(Mutex::new(Population::new(population_size)));

        let _ = (0..self.iterations).into_par_iter().map(|_| {
            let mut rng = rand::thread_rng();
            let params: Vec<f64> = param_bounds
                .iter()
                .map(|(min, max)| rng.gen_range(*min..=*max))
                .collect();

            let score = objective.evaluate(&params);

            let mut pop = shared_population.lock().expect("Population lock poisoned.");
            pop.insert(Candidate { params, score });
        });
        Arc::try_unwrap(shared_population)
            .expect("Expected sole ownership of Arc")
            .into_inner()
            .expect("Population lock poisoned.")
    }
}
