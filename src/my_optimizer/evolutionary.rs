// evolutionary algorithm

use super::{
    evaluate_with_shared_error, Candidate, ObjectiveFunction, Optimizer, ParamDescriptor,
    Population, PopulationSaver, ProgressReporter, Schedule, SharedError, SharedPopulation,
};
use anyhow::Context;
use rand::prelude::*;
use rayon::prelude::*;
use tracing::{debug, error, info, span, warn, Level};

pub struct EvolutionaryOptimizer<
    Selection: Schedule,
    HardMutation: Schedule,
    SoftMutation: Schedule,
> {
    pub generations: usize,
    pub population_size: usize,
    pub hard_mutation_rate: HardMutation,
    pub soft_mutation_std_dev: SoftMutation,
    pub max_attempts: usize, // Maximum attempts to generate a valid offspring
    pub tolerance: f64,      // Tolerance for comparing offspring with existing candidates
    pub precision: usize,    // Precision for floating-point hashing of candidates
    pub selection_schedule: Selection,
    pub initial_population: Population,
    pub population_saver: Option<PopulationSaver>,
}

impl<Selection: Schedule, HardMutation: Schedule, SoftMutation: Schedule> ProgressReporter
    for EvolutionaryOptimizer<Selection, HardMutation, SoftMutation>
{
    fn get_estimate_of_cycles(&self, _param_bounds: &[ParamDescriptor]) -> anyhow::Result<usize> {
        let mut estimation = 0;
        for seq in 0..self.generations {
            let fraction = self
                .selection_schedule
                .value_at(seq, self.generations)
                .clamp(0.0, 1.0);
            let parents = (self.population_size as f64 * fraction).ceil() as usize;
            estimation += parents;
        }
        Ok(estimation)
    }
}

impl<Selection: Schedule, HardMutation: Schedule, SoftMutation: Schedule> Optimizer
    for EvolutionaryOptimizer<Selection, HardMutation, SoftMutation>
{
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
            Some(self.precision),
        );
        let shared_error = SharedError::new();

        // evolution loop
        for gen in 0..self.generations {
            let gen_span = span!(Level::INFO, "Generation", generation = gen + 1);
            let _gen_enter = gen_span.enter();

            let selection_fraction = self
                .selection_schedule
                .value_at(gen, self.generations)
                .clamp(0.0, 1.0);

            let parent_count = ((population_size as f64) * selection_fraction).ceil() as usize;

            if parent_count == 0 {
                warn!("Selection schedule resulted in empty selection");
                continue;
            }

            let top_parents = shared_population.top_n(parent_count);
            let hard_mutation_rate = self.hard_mutation_rate.value_at(gen, self.generations);
            let soft_mutation_std_dev = self.soft_mutation_std_dev.value_at(gen, self.generations);
            info!(
                "Starting offspring generation: Parent count = {}, hard mutation rate = {:.2}, soft mutation std dev = {:.2}",
                parent_count, hard_mutation_rate, soft_mutation_std_dev
            );

            // offspring generation (parallel)
            (0..parent_count).into_par_iter().for_each(|offspring_id| {
                if shared_error.is_set() {
                    return;
                }
                let offspring_span = span!(
                    Level::DEBUG,
                    "Offspring",
                    id = offspring_id,
                    generation = gen + 1
                );
                let _offspring_enter = offspring_span.enter();

                let mut rng = rand::thread_rng();
                let parent = top_parents.choose(&mut rng).unwrap();
                let child_params = match parent.generate_offspring_params(
                    param_bounds,
                    hard_mutation_rate,
                    soft_mutation_std_dev,
                    self.max_attempts,
                    self.tolerance,
                    &shared_population,
                ) {
                    Ok(child_params) => child_params,
                    Err(e) => {
                        error!(
                            error = %e,
                            "Mutation failed, aborting..."
                        );
                        shared_error.set_if_empty(e);
                        return;
                    }
                };

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
