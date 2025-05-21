// evolutionary algorithm

use super::{
    Candidate, ObjectiveFunction, Optimizer, ParamDescriptor, Population, PopulationSaver,
    ProgressReporter, SelectionSchedule,
};
use rand::prelude::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, span, warn, Level};

pub struct EvolutionaryOptimizer<S: SelectionSchedule> {
    pub generations: usize,
    pub population_size: usize,
    pub mutation_rate: f64,
    pub hard_mutation_rate: f64,
    pub soft_mutation_std_dev: f64,
    pub selection_schedule: S,
    pub initial_population: Population,
    pub population_saver: Option<PopulationSaver>,
}

impl<S: SelectionSchedule + Sync> ProgressReporter for EvolutionaryOptimizer<S> {
    fn get_estimate_of_cycles(&self, _param_bounds: &[ParamDescriptor]) -> usize {
        self.selection_schedule
            .estimate_evaluations(self.generations, self.population_size)
    }
}

impl<S: SelectionSchedule + Sync> Optimizer for EvolutionaryOptimizer<S> {
    fn optimize<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
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

        if self.population_size != population_size {
            warn!(
                "Input of population_size {} is not equal to self.population_size {}",
                population_size, self.population_size
            );
        }

        // Shared Population and Saver
        let shared_population = Arc::new(Mutex::new(self.initial_population.clone()));
        let shared_population_saver = Arc::new(Mutex::new(self.population_saver.clone()));

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

                for (i, pb) in param_bounds.iter().enumerate() {
                    child_params[i] = pb.mutate(
                        child_params[i],
                        &mut rng,
                        self.soft_mutation_std_dev,
                        self.hard_mutation_rate,
                    );
                }

                let score = objective.evaluate(&child_params);

                debug!(?child_params, score, "Generated offspring candidate");

                let mut pop = shared_population.lock().expect("Population lock poisoned.");
                pop.insert(Candidate {
                    params: child_params,
                    score,
                });
                let ops = shared_population_saver
                    .lock()
                    .expect("PopulationSaver lock poisoned.");
                if let Some(ps) = ops.as_ref() {
                    ps.save_population(&pop, param_bounds);
                }
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
