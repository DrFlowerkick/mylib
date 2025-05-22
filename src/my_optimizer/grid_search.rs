// grid search for significant parameter sets

use super::{
    evaluate_with_shared_error, Candidate, Explorer, ObjectiveFunction, ParamBound,
    ParamDescriptor, Population, PopulationSaver, ProgressReporter, SharedError, SharedPopulation,
};
use anyhow::Context;
use itertools::Itertools;
use rayon::prelude::*;
use tracing::{debug, info, span, Level};

pub struct GridSearch {
    pub steps_per_param: usize,
    pub chunk_size: usize,
    pub population_saver: Option<PopulationSaver>,
}

impl ProgressReporter for GridSearch {
    fn get_estimate_of_cycles(&self, param_bounds: &[ParamDescriptor]) -> anyhow::Result<usize> {
        let mut num_cycles = 1;
        for bound in param_bounds.iter() {
            num_cycles *= match &bound.bound {
                ParamBound::Static(_) => 1,
                ParamBound::MinMax(_, _) => self.steps_per_param,
                ParamBound::List(values) => {
                    let num_entries = values.len();
                    if num_entries == 0 {
                        return Err(anyhow::anyhow!(
                            "ParamBound::List: empty list of parameter {}",
                            bound.name
                        ));
                    }
                    num_entries
                }
                ParamBound::LogScale(_, _) => self.steps_per_param,
            };
        }

        Ok(num_cycles)
    }
}

impl Explorer for GridSearch {
    fn explore<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
        population_size: usize,
    ) -> anyhow::Result<Population> {
        let span_search = span!(Level::INFO, "GridSearch");
        let _enter = span_search.enter();

        info!(
            "Starting Grid Search with {} candidates",
            self.get_estimate_of_cycles(param_bounds)?
        );

        // Shared Population and Error
        let shared_population = SharedPopulation::new(
            Population::new(population_size),
            self.population_saver.clone(),
        );
        let shared_error = SharedError::new();
        let param_generator = GridSearchIterator::new(param_bounds, self.steps_per_param);

        for (chunk_index, chunks) in param_generator
            .chunks(self.chunk_size)
            .into_iter()
            .enumerate()
        {
            let chunk_span = span!(Level::INFO, "GridChunk", chunk_index);
            let _chunk_enter = chunk_span.enter();

            info!("Starting grid search in chunk {}", chunk_index);

            let batch: Vec<_> = chunks.collect();

            batch.into_par_iter().for_each(|params| {
                if let Some(score) = evaluate_with_shared_error(objective, &params, &shared_error) {
                    debug!(?params, score, "Evaluated candidate");

                    shared_population.insert(
                        Candidate { params, score },
                        param_bounds,
                        &shared_error,
                    );
                }
            });

            if let Some(err) = shared_error.take() {
                return Err(err);
            }
        }

        let population = shared_population.take();

        info!(
            "Coarse Grid Search completed. Best Score: {:.3}",
            population
                .best()
                .map(|c| c.score)
                .context("Empty population")?
        );

        Ok(population)
    }
}

struct GridSearchIterator<'a> {
    param_bounds: &'a [ParamDescriptor],
    steps_per_param: usize,
    current_indices: Vec<usize>, // index per parameter bound
    done: bool,
}

impl<'a> GridSearchIterator<'a> {
    pub fn new(param_bounds: &'a [ParamDescriptor], steps_per_param: usize) -> Self {
        GridSearchIterator {
            param_bounds,
            steps_per_param,
            current_indices: vec![0; param_bounds.len()],
            done: false,
        }
    }
}

impl<'a> Iterator for GridSearchIterator<'a> {
    type Item = Vec<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        // build params from current_indices
        let params = self
            .current_indices
            .iter()
            .enumerate()
            .map(|(i, &idx)| {
                let bound = &self.param_bounds[i].bound;
                match bound {
                    ParamBound::MinMax(min, max) => {
                        let step_size = (*max - *min) / (self.steps_per_param - 1) as f64;
                        *min + step_size * idx as f64
                    }
                    ParamBound::List(values) => {
                        assert!(
                            !values.is_empty(),
                            "Empty value list in parameter descriptor {}",
                            &self.param_bounds[i].name
                        );
                        values[idx]
                    }
                    ParamBound::Static(val) => *val,
                    ParamBound::LogScale(min, max) => {
                        let log_min = min.ln();
                        let log_max = max.ln();
                        let step_size = (log_max - log_min) / (self.steps_per_param - 1) as f64;
                        (log_min + step_size * idx as f64).exp()
                    }
                }
            })
            .collect();

        // set next current_indices
        let mut i = self.current_indices.len();
        while i > 0 {
            i -= 1;
            self.current_indices[i] += 1;
            let limit = match &self.param_bounds[i].bound {
                ParamBound::MinMax(_, _) => self.steps_per_param,
                ParamBound::List(v) => v.len(),
                ParamBound::Static(_) => 1,
                ParamBound::LogScale(_, _) => self.steps_per_param,
            };
            if self.current_indices[i] < limit {
                break;
            } else {
                self.current_indices[i] = 0;
                if i == 0 {
                    self.done = true;
                }
            }
        }

        Some(params)
    }
}
