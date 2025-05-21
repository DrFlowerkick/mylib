// data types for handling parameters

use super::ObjectiveFunction;
use rand::prelude::*;
use rand_distr::Normal;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, span, Level};

#[derive(Clone, Debug, PartialEq)]
pub enum ParamBound {
    Static(f64),        // static value, parameter will not be changed
    MinMax(f64, f64),   // continuous value range
    List(Vec<f64>),     // discreet values
    LogScale(f64, f64), // logarithmic parameter scaling
}

impl ParamBound {
    pub fn rng_sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        match self {
            ParamBound::Static(val) => *val,
            ParamBound::MinMax(min, max) => rng.gen_range(*min..=*max),
            ParamBound::List(values) => *values.choose(rng).expect("Empty parameter list."),
            ParamBound::LogScale(min, max) => {
                let log_min = min.ln();
                let log_max = max.ln();
                let sample_log = rng.gen_range(log_min..=log_max);
                sample_log.exp()
            }
        }
    }

    pub fn mutate<R: Rng + ?Sized>(
        &self,
        current_value: f64,
        rng: &mut R,
        soft_mutation_std_dev: f64,
        hard_mutation_rate: f64,
        name: &str,
    ) -> f64 {
        match self {
            ParamBound::Static(val) => *val, // mutation is not allowed
            ParamBound::MinMax(min, max) => {
                if rng.gen::<f64>() < hard_mutation_rate {
                    // hard mutation → new value in range
                    rng.gen_range(*min..=*max)
                } else {
                    // soft mutation → Gaussian Noise
                    let noise = rng.sample(Normal::new(0.0, soft_mutation_std_dev).unwrap());
                    let value = (current_value + noise).clamp(*min, *max);
                    if value == *min || value == *max {
                        debug!(%name, %value, "Value clamped to bounds.");
                    }
                    value
                }
            }
            ParamBound::List(values) => {
                if rng.gen::<f64>() < hard_mutation_rate {
                    // hard mutation → random value from list
                    *values.choose(rng).expect("Parameter list is empty!")
                } else {
                    // soft mutation → choose value nearest to current value plus noise
                    let noise = rng.sample(Normal::new(0.0, soft_mutation_std_dev).unwrap());
                    let target_value = current_value + noise;

                    *values
                        .iter()
                        .min_by(|&&a, &&b| {
                            (a - target_value)
                                .abs()
                                .total_cmp(&(b - target_value).abs())
                        })
                        .expect("Parameter list is empty!")
                }
            }
            ParamBound::LogScale(min, max) => {
                let log_min = min.ln();
                let log_max = max.ln();
                let current_log = current_value.ln();

                if rng.gen::<f64>() < hard_mutation_rate {
                    // hard mutation → random value in log range
                    let new_log = rng.gen_range(log_min..=log_max);
                    new_log.exp()
                } else {
                    // soft mutation → Gaussian Noise in log range
                    let noise = rng.sample(Normal::new(0.0, soft_mutation_std_dev).unwrap());
                    let log_value = (current_log + noise).clamp(log_min, log_max);
                    let value = log_value.exp();
                    if value <= *min || value >= *max {
                        debug!(%name, %value, "Log-clamped value");
                    }
                    value
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParamDescriptor {
    pub name: String,
    pub bound: ParamBound,
}

impl ParamDescriptor {
    pub fn rng_sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        self.bound.rng_sample(rng)
    }
    pub fn mutate<R: Rng + ?Sized>(
        &self,
        current_value: f64,
        rng: &mut R,
        soft_mutation_std_dev: f64,
        hard_mutation_rate: f64,
    ) -> f64 {
        self.bound.mutate(
            current_value,
            rng,
            soft_mutation_std_dev,
            hard_mutation_rate,
            &self.name,
        )
    }
}

// conversion trait
pub trait CsvConversion {
    fn to_csv(&self, precision: usize) -> String;
    fn from_csv(csv: &str) -> Option<Self>
    where
        Self: Sized;
}

// struct to return optimization result
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub params: Vec<f64>,
    pub score: f64,
}

impl Candidate {
    pub fn new(params: Vec<f64>, score: f64) -> Self {
        Self { params, score }
    }

    pub fn describe_candidate(&self, param_bounds: &[ParamDescriptor]) -> Vec<(String, f64)> {
        self.params
            .iter()
            .enumerate()
            .map(|(i, &val)| (param_bounds[i].name.to_owned(), val))
            .collect()
    }
}

impl CsvConversion for Candidate {
    fn to_csv(&self, precision: usize) -> String {
        let mut csv_line = String::new();

        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                csv_line.push(',');
            }
            csv_line.push_str(&format!("{:.precision$}", param, precision = precision));
        }

        // Append the score at the end
        csv_line.push(',');
        csv_line.push_str(&format!(
            "{:.precision$}",
            self.score,
            precision = precision
        ));

        csv_line
    }
    fn from_csv(csv: &str) -> Option<Self> {
        if let Ok(mut params) = csv
            .split(',')
            .map(|num| num.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
        {
            let Some(score) = params.pop() else { return None };
            return Some(Candidate { params, score });
        }
        None
    }
}

impl Eq for Candidate {}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.total_cmp(&other.score).reverse()
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// struct of population of candidates
#[derive(Debug, Clone)]
pub struct Population {
    members: BTreeSet<Candidate>,
    capacity: usize,
}

impl Population {
    pub fn new(capacity: usize) -> Self {
        Self {
            members: BTreeSet::new(),
            capacity,
        }
    }

    // if capacity is reached, remove worst candidate and return it
    pub fn insert(&mut self, candidate: Candidate) -> Option<Candidate> {
        self.members.insert(candidate);
        if self.members.len() > self.capacity {
            return self.members.pop_last();
        }
        None
    }

    pub fn populate<F: ObjectiveFunction + Sync>(
        self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
    ) -> Population {
        // since objective may be a costly objective, tracing is used to signal status of populate
        let init_span = span!(Level::INFO, "PopulatePopulation", size = self.capacity);
        let _enter = init_span.enter();

        let current_population_size = self.members.len();

        if current_population_size == self.capacity {
            info!(
                "Population is already populated with {} candidates",
                self.capacity
            );
            return self;
        }

        if current_population_size > 0 {
            info!(
                "Population contains already {} candidates.",
                current_population_size,
            );
        }

        info!("Populating population until {} candidates", self.capacity);
        let remaining_candidates = self.capacity - current_population_size;
        let shared_population = Arc::new(Mutex::new(self));

        (0..remaining_candidates).into_par_iter().for_each(|_| {
            let mut rng = rand::thread_rng();
            let params: Vec<f64> = param_bounds
                .iter()
                .map(|pb| pb.rng_sample(&mut rng))
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
            "Population fully populated. Best Score: {:.3}",
            population.best().map(|c| c.score).unwrap_or(-1.0)
        );

        population
    }

    pub fn resize_population<F: ObjectiveFunction + Sync>(
        mut self,
        new_capacity: usize,
        populate_with: Option<(&F, &[ParamDescriptor])>,
    ) -> Population {
        match self.capacity.cmp(&new_capacity) {
            Ordering::Equal => (),
            Ordering::Greater => {
                self.capacity = new_capacity;
                (0..self.capacity - new_capacity).for_each(|_| {
                    self.members.pop_last();
                });
            }
            Ordering::Less => {
                if let Some((objective, param_bounds)) = populate_with {
                    self = self.populate(objective, param_bounds);
                }
            }
        }
        self
    }

    pub fn top_n(&self, n: usize) -> impl Iterator<Item = &Candidate> {
        self.members.iter().take(n)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Candidate> {
        self.members.iter()
    }

    pub fn size(&self) -> usize {
        self.members.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // merge with other population considering capacity
    pub fn merge(&mut self, other: Population) {
        for candidate in other.members.into_iter() {
            self.insert(candidate);
        }
    }

    pub fn best(&self) -> Option<&Candidate> {
        self.members.first()
    }

    pub fn pop_best(&mut self) -> Option<Candidate> {
        self.members.pop_first()
    }
}

impl CsvConversion for Population {
    fn to_csv(&self, precision: usize) -> String {
        let mut csv = String::new();

        for (index, candidate) in self.members.iter().enumerate() {
            if index > 0 {
                csv.push('\n');
            }
            csv.push_str(&candidate.to_csv(precision));
        }

        csv
    }
    fn from_csv(csv: &str) -> Option<Self>
    where
        Self: Sized,
    {
        let candidates: Vec<Candidate> = csv.lines().filter_map(Candidate::from_csv).collect();
        let capacity = candidates.len();
        if capacity == 0 {
            return None;
        }
        let mut population = Population::new(capacity);
        candidates.into_iter().for_each(|c| {
            population.insert(c);
        });
        Some(population)
    }
}
