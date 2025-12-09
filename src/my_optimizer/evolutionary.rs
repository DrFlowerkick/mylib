// evolutionary algorithm

use super::{
    ObjectiveFunction, Optimizer, ParamDescriptor, Population, PopulationSaver, ProgressReporter,
    Schedule, SharedError, SharedPopulation, ToleranceSettings, evaluate_with_shared_error,
};
use anyhow::Context;
use rand::{prelude::*, rng};
use rayon::prelude::*;
use tracing::{Level, error, info, span, warn};

pub struct EvolutionaryOptimizer<
    Selection: Schedule,
    HardMutation: Schedule,
    SoftMutation: Schedule,
    TS: ToleranceSettings,
> {
    pub generations: usize,
    pub population_size: usize,
    pub hard_mutation_rate: HardMutation,
    pub soft_mutation_relative_std_dev: SoftMutation,
    pub max_attempts: usize, // Maximum attempts to generate a valid offspring
    pub selection_schedule: Selection,
    pub initial_population: Population<TS>,
    pub population_saver: Option<PopulationSaver>,
}

impl<Selection: Schedule, HardMutation: Schedule, SoftMutation: Schedule, TS: ToleranceSettings>
    ProgressReporter for EvolutionaryOptimizer<Selection, HardMutation, SoftMutation, TS>
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

impl<Selection: Schedule, HardMutation: Schedule, SoftMutation: Schedule, TS: ToleranceSettings>
    Optimizer<TS> for EvolutionaryOptimizer<Selection, HardMutation, SoftMutation, TS>
{
    fn optimize<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
        population_size: usize,
    ) -> anyhow::Result<Population<TS>> {
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
        for generation in 0..self.generations {
            let gen_span = span!(Level::INFO, "Generation", generation = generation + 1);
            let _gen_enter = gen_span.enter();

            let selection_fraction = self
                .selection_schedule
                .value_at(generation, self.generations)
                .clamp(0.0, 1.0);

            let parent_count = ((population_size as f64) * selection_fraction).ceil() as usize;

            if parent_count == 0 {
                warn!("Selection schedule resulted in empty selection");
                continue;
            }

            let top_parents = shared_population.top_n(parent_count);
            let hard_mutation_rate = self
                .hard_mutation_rate
                .value_at(generation, self.generations);
            let soft_mutation_relative_std_dev = self
                .soft_mutation_relative_std_dev
                .value_at(generation, self.generations);
            info!(
                parent_count,
                hard_mutation_rate, soft_mutation_relative_std_dev, "Starting offspring generation",
            );

            // offspring generation (parallel)
            let evo_span_clone = evo_span.clone();
            let gen_span_clone = gen_span.clone();
            (0..parent_count).into_par_iter().for_each(|offspring_id| {
                if shared_error.is_set() {
                    return;
                }
                let _evo_enter = evo_span_clone.enter();
                let _gen_enter = gen_span_clone.enter();
                let offspring_span = span!(
                    Level::DEBUG,
                    "Offspring",
                    id = offspring_id,
                    generation = generation + 1
                );
                let _offspring_enter = offspring_span.enter();

                let mut thread_rng = rng();
                let parent = top_parents.choose(&mut thread_rng).unwrap();
                match parent.log::<F>(Level::DEBUG, "Selected Parent") {
                    Ok(_) => {}
                    Err(e) => {
                        error!(error = %e, "Failed to log selected parent");
                        shared_error.set_if_empty(e);
                        return;
                    }
                };
                let child_params = match parent.generate_offspring_params(
                    param_bounds,
                    hard_mutation_rate,
                    soft_mutation_relative_std_dev,
                    self.max_attempts,
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

                if let Some(candidate) =
                    evaluate_with_shared_error(objective, &child_params, &shared_error)
                {
                    shared_population.insert(candidate, param_bounds, &shared_error);
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
                generation + 1,
                best.score,
                best.params
            );
        }

        // final population
        shared_population.lock().save_population(param_bounds)?;
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
