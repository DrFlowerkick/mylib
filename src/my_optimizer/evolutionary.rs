// evolutionary algorithm

use super::{Candidate, ObjectiveFunction, Optimizer, Population, SelectionSchedule};
use rand::prelude::*;
use rand_distr::Normal;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, span, Level};

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
        let init_span = span!(Level::INFO, "InitPopulation", size = population_size);
        let _enter = init_span.enter();

        info!(
            "Initializing population with {} candidates",
            population_size
        );

        let shared_population = Arc::new(Mutex::new(Population::new(population_size)));
        (0..population_size).into_par_iter().for_each(|_| {
            let mut rng = rand::thread_rng();
            let params: Vec<f64> = param_bounds
                .iter()
                .map(|(min, max)| rng.gen_range(*min..=*max))
                .collect();
            debug!(?params, "Generated initial candidate parameters");

            let score = objective.evaluate(&params);
            debug!(score, "Initial candidate evaluated");

            let mut pop = shared_population.lock().expect("Population lock poisoned.");
            pop.insert(Candidate { params, score });
        });

        let population = Arc::try_unwrap(shared_population)
            .expect("Expected sole ownership of Arc")
            .into_inner()
            .expect("Population lock poisoned.");

        info!(
            "Initial population created. Best Score: {:.3}",
            population.best().map(|c| c.score).unwrap_or(-1.0)
        );

        population
    }
}

impl<S: SelectionSchedule + Sync> Optimizer for EvolutionaryOptimizer<S> {
    fn optimize<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[(f64, f64)],
        population_size: usize,
    ) -> Population {
        let evo_span = span!(
            Level::INFO,
            "EvolutionaryOptimizer",
            generations = self.generations
        );
        let _evo_enter = evo_span.enter();

        info!(
            "Starting Evolutionary Optimizer with {} generations",
            self.generations
        );

        // use initial_population if provided
        let initial_population = self
            .initial_population
            .clone()
            .unwrap_or_else(|| self.init_population(objective, param_bounds, population_size));

        let shared_population = Arc::new(Mutex::new(initial_population));

        // evolution loop
        for gen in 0..self.generations {
            let gen_span = span!(Level::INFO, "Generation", number = gen + 1);
            let _gen_enter = gen_span.enter();

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

            info!(
                "Starting offspring generation: Parent count = {}, Mutation rate = {:.2}",
                parent_count, self.mutation_rate
            );

            // offspring generation (parallel)
            (0..parent_count).into_par_iter().for_each(|offspring_id| {
                let offspring_span = span!(Level::DEBUG, "Offspring", id = offspring_id);
                let _offspring_enter = offspring_span.enter();

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

                debug!(?child_params, score, "Generated offspring candidate");

                let mut pop = shared_population.lock().expect("Population lock poisoned.");
                pop.insert(Candidate {
                    params: child_params,
                    score,
                });
            });

            // Logging best candidate after this generation
            let population = shared_population.lock().expect("Population lock poisoned.");
            let best = population.best().expect("Population is empty!");

            info!(
                "Generation {} completed. Best Score: {:.3}, Params: {:?}",
                gen + 1,
                best.score,
                best.params
            );
        }

        // final population
        let population = Arc::try_unwrap(shared_population)
            .expect("Expected sole ownership of Arc")
            .into_inner()
            .expect("Population lock poisoned.");

        info!(
            "Evolutionary Optimizer completed. Best Score: {:.3}",
            population.best().map(|c| c.score).unwrap_or(-1.0)
        );

        population
    }
}
