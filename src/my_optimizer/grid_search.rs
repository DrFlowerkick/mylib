// grid search for significant parameter sets

use super::{
    Candidate, Explorer, ObjectiveFunction, ParamBound, Population, PopulationSaver,
    ProgressReporter,
};
use crossbeam::channel::{bounded, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{debug, info, span, Level};

pub struct GridSearch {
    pub steps_per_param: usize,
    pub channel_capacity: usize, // control over back pressure
    pub worker_threads: usize,   // number of consumer workers
    pub population_saver: Option<PopulationSaver>,
}

impl ProgressReporter for GridSearch {
    fn get_estimate_of_cycles(&self, param_bounds: &[ParamBound]) -> usize {
        param_bounds
            .iter()
            .map(|bound| match bound {
                ParamBound::Static(_) => 1,
                ParamBound::MinMax(_, _) => self.steps_per_param,
                ParamBound::List(values) => values.len(),
            })
            .product()
    }
}

impl Explorer for GridSearch {
    fn explore<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamBound],
        population_size: usize,
    ) -> Population {
        let span_search = span!(Level::INFO, "GridSearch");
        let _enter = span_search.enter();

        info!(
            "Starting Grid Search with {} consumers",
            self.worker_threads
        );

        let (sender, receiver) = bounded::<Vec<f64>>(self.channel_capacity);

        // Producer-Thread
        let producer_handle = {
            //let sender = sender.clone();
            let steps_per_param = self.steps_per_param;
            let param_bounds = param_bounds.to_vec();
            thread::spawn(move || {
                generate_params_recursive(
                    &param_bounds,
                    steps_per_param.max(2),
                    &mut vec![],
                    sender,
                );
            })
        };

        // Shared Population and Saver
        let shared_population = Arc::new(Mutex::new(Population::new(population_size)));
        let shared_population_saver = Arc::new(Mutex::new(self.population_saver.clone()));

        // Consumers parallelization with Rayon
        let objective = Arc::new(objective);
        rayon::scope(|s| {
            for _ in 0..self.worker_threads {
                let receiver = receiver.clone();
                let shared_pop = Arc::clone(&shared_population);
                let objective = Arc::clone(&objective);
                let shared_ps = Arc::clone(&shared_population_saver);

                s.spawn(move |_| {
                    for params in receiver.iter() {
                        let score = objective.evaluate(&params);

                        debug!(?params, score, "Generated Coarse Grid Search candidate");

                        let mut pop = shared_pop.lock().expect("Population lock poisoned.");
                        pop.insert(Candidate { params, score });
                        let ops = shared_ps.lock().expect("PopulationSaver lock poisoned.");
                        if let Some(ps) = ops.as_ref() {
                            ps.save_population(&pop);
                        }
                    }
                });
            }
        });

        // wait on end of Producer
        producer_handle.join().expect("Producer thread panicked.");

        let population = Arc::try_unwrap(shared_population)
            .expect("Population still has multiple references.")
            .into_inner()
            .unwrap();

        info!(
            "Coarse Grid Search completed. Best Score: {:.3}",
            population.best().map(|c| c.score).unwrap_or(-1.0)
        );

        population
    }
}

fn generate_params_recursive(
    param_bounds: &[ParamBound],
    steps_per_param: usize,
    current_params: &mut Vec<f64>,
    sender: Sender<Vec<f64>>,
) {
    if current_params.len() == param_bounds.len() {
        debug!(
            ?current_params,
            "Generated parameter combination for grid search."
        );

        if sender.send(current_params.clone()).is_err() {
            tracing::warn!(
                "Failed to send parameter combination: Receiver has likely been dropped. Aborting this branch."
            );
        }

        return;
    }

    match &param_bounds[current_params.len()] {
        ParamBound::Static(val) => {
            current_params.push(*val);
            generate_params_recursive(param_bounds, steps_per_param, current_params, sender);
            current_params.pop();
        }
        ParamBound::MinMax(min, max) => {
            for step in 0..steps_per_param {
                let value = min + (max - min) * (step as f64) / (steps_per_param - 1) as f64;
                current_params.push(value);
                generate_params_recursive(
                    param_bounds,
                    steps_per_param,
                    current_params,
                    sender.clone(),
                );
                current_params.pop();
            }
        }
        ParamBound::List(values) => {
            for &value in values {
                current_params.push(value);
                generate_params_recursive(
                    param_bounds,
                    steps_per_param,
                    current_params,
                    sender.clone(),
                );
                current_params.pop();
            }
        }
    }
}
