// evolutionary algorithm

use super::{Candidate, ObjectiveFunction, Optimizer, Population, SelectionSchedule};
use rand::prelude::*;
use rand_distr::Normal;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub struct EvolutionaryOptimizer<S: SelectionSchedule> {
    pub generations: usize,
    pub mutation_rate: f64,
    pub hard_mutation_rate: f64,
    pub soft_mutation_std_dev: f64,
    pub selection_schedule: S,
    pub initial_population: Option<Population>,
}

impl<S: SelectionSchedule> EvolutionaryOptimizer<S> {
    pub fn init_population<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[(f64, f64)],
        population_size: usize,
    ) -> Population {
        let shared_population = Arc::new(Mutex::new(Population::new(population_size)));
        let _ = (0..population_size).into_par_iter().map(|_| {
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

impl<S: SelectionSchedule + Sync> Optimizer for EvolutionaryOptimizer<S> {
    fn optimize<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[(f64, f64)],
        population_size: usize,
    ) -> Population {
        // use initial_population if provided
        let initial_population = self
            .initial_population
            .clone()
            .unwrap_or_else(|| self.init_population(objective, param_bounds, population_size));

        let shared_population = Arc::new(Mutex::new(initial_population));

        // evolution loop
        for gen in 0..self.generations {
            let selection_fraction = self
                .selection_schedule
                .selection_fraction(gen, self.generations)
                .clamp(0.0, 1.0);

            let parent_count = ((population_size as f64) * selection_fraction).ceil() as usize;
            let top_parents: Vec<Candidate> = shared_population
                .lock()
                .expect("Population lock poisoned.")
                .top_n(parent_count)
                .cloned()
                .collect();

            // offspring generation (parallel)
            let _offspring = (0..parent_count).into_par_iter().map(|_| {
                let mut rng = rand::thread_rng();
                let parent = top_parents.choose(&mut rng).expect("Empty population.");
                let mut child_params = parent.params.clone();

                for (i, (min, max)) in param_bounds.iter().enumerate() {
                    if rng.gen::<f64>() < self.mutation_rate {
                        if rng.gen::<f64>() < self.hard_mutation_rate {
                            // hard mutation: random value in given range
                            child_params[i] = rng.gen_range(*min..=*max);
                        } else {
                            // soft mutation: Normal distribution
                            let noise =
                                rng.sample(Normal::new(0.0, self.soft_mutation_std_dev).unwrap());
                            child_params[i] = (child_params[i] + noise).clamp(*min, *max);
                        }
                    }
                }

                let score = objective.evaluate(&child_params);
                let mut pop = shared_population.lock().expect("Population lock poisoned.");
                pop.insert(Candidate {
                    params: child_params,
                    score,
                });
            });

            // Logging
            let population = shared_population.lock().expect("Population lock poisoned.");
            let best = population.best().expect("Empty population.");
            println!(
                "Generation {}: Best Score = {:.3}, Params = {:?}",
                gen + 1,
                best.score,
                best.params
            );
        }

        // best candidate is first candidate in population
        Arc::try_unwrap(shared_population)
            .expect("Expected sole ownership of Arc")
            .into_inner()
            .expect("Population lock poisoned.")
    }
}
