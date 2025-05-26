// evolutionary algorithm

use super::{
    evaluate_with_shared_error, Candidate, ObjectiveFunction, Optimizer, ParamDescriptor,
    Population, PopulationSaver, ProgressReporter, SelectionSchedule, SharedError,
    SharedPopulation,
};
use anyhow::Context;
use rand::prelude::*;
use rayon::prelude::*;
use tracing::{debug, error, info, span, warn, Level};

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
    fn get_estimate_of_cycles(&self, _param_bounds: &[ParamDescriptor]) -> anyhow::Result<usize> {
        Ok(self
            .selection_schedule
            .estimate_evaluations(self.generations, self.population_size))
    }
}

impl<S: SelectionSchedule + Sync> Optimizer for EvolutionaryOptimizer<S> {
    fn optimize<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
        population_size: usize,
    ) -> anyhow::Result<Population> {
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

        // Shared Population and Error
        let shared_population = SharedPopulation::new(
            self.initial_population.clone(),
            self.population_saver.clone(),
        );
        let shared_error = SharedError::new();

        // evolution loop
        for gen in 0..self.generations {
            let gen_span = span!(Level::INFO, "Generation", generation = gen + 1);
            let _gen_enter = gen_span.enter();

            let selection_fraction = self
                .selection_schedule
                .selection_fraction(gen, self.generations)
                .clamp(0.0, 1.0);

            let parent_count = ((population_size as f64) * selection_fraction).ceil() as usize;

            if parent_count == 0 {
                warn!("Selection schedule resulted in empty selection");
                continue;
            }

            let top_parents = shared_population.top_n(parent_count);
            info!(
                "Starting offspring generation: Parent count = {}, Mutation rate = {:.2}",
                parent_count, self.mutation_rate
            );

            // offspring generation (parallel)
            (0..parent_count).into_par_iter().for_each(|offspring_id| {
                if shared_error.is_set() {
                    return;
                }
                let offspring_span = span!(Level::DEBUG, "Offspring", id = offspring_id, generation = gen + 1);
                let _offspring_enter = offspring_span.enter();

                let mut rng = rand::thread_rng();
                let parent = top_parents.choose(&mut rng).unwrap();
                let mut child_params = parent.params.clone();

                for (i, pb) in param_bounds.iter().enumerate() {
                    match pb.mutate(
                        child_params[i],
                        &mut rng,
                        self.soft_mutation_std_dev,
                        self.hard_mutation_rate,
                    ) {
                        Ok(mutate_value) => {
                            child_params[i] = mutate_value;
                        }
                        Err(e) => {
                            let param_name = &param_bounds[i].name;
                            error!(
                                ?param_name,
                                error = %e,
                                "Mutation failed, aborting..."
                            );
                            shared_error.set_if_empty(e);
                            return;
                        }
                    }
                }

                if let Some(score) =
                    evaluate_with_shared_error(objective, &child_params, &shared_error)
                {
                    debug!(?child_params, score, "Generated offspring candidate");

                    shared_population.insert(
                        Candidate {
                            params: child_params,
                            score,
                        },
                        param_bounds,
                        &shared_error,
                    );
                }
            });

            if let Some(err) = shared_error.take() {
                return Err(err);
            }

            // Logging best candidate after this generation
            let population = shared_population.lock();
            let best = population.best().context("Population is empty!")?;

            info!(
                "Generation {} completed. Best Score: {:.3}, Params: {:?}",
                gen + 1,
                best.score,
                best.params
            );
        }

        // final population
        let population = shared_population.take();

        info!(
            "Evolutionary Optimizer completed. Best Score: {:.3}",
            population
                .best()
                .map(|c| c.score)
                .context("Empty population")?
        );

        Ok(population)
    }
}
