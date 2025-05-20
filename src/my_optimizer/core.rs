// traits & type definitions

use rand::prelude::SliceRandom;
use rand::Rng;
use rand_distr::Normal;
use std::cmp::Ordering;
use std::collections::BTreeSet;

// trait of target function
pub trait ObjectiveFunction {
    fn evaluate(&self, params: &[f64]) -> f64;
}

// enable usage of closures as ObjectiveFunction
impl<F> ObjectiveFunction for F
where
    F: Fn(&[f64]) -> f64 + Sync,
{
    fn evaluate(&self, params: &[f64]) -> f64 {
        self(params)
    }
}

pub trait ProgressReporter {
    // returns estimation of number of steps of exploration or optimization
    fn get_estimate_of_cycles(&self, param_bounds: &[ParamBound]) -> usize;
}

// common trait for all explorer
pub trait Explorer: ProgressReporter {
    fn explore<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamBound],
        population_size: usize, // Top-N results
    ) -> Population;
}

// common trait for all optimizer
pub trait Optimizer: ProgressReporter {
    fn optimize<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[ParamBound],
        population_size: usize,
    ) -> Population;
}

// ToDo: implemented saving Population every N cycles to file

// enum to provide parameter bounds
#[derive(Clone, Debug)]
pub enum ParamBound {
    Static(f64),      // static value, parameter will not be changed
    MinMax(f64, f64), // continuous value range
    List(Vec<f64>),   // discreet values
}

impl ParamBound {
    pub fn rng_sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        match self {
            ParamBound::Static(val) => *val,
            ParamBound::MinMax(min, max) => rng.gen_range(*min..=*max),
            ParamBound::List(values) => *values.choose(rng).expect("Empty parameter list."),
        }
    }

    pub fn mutate<R: Rng + ?Sized>(
        &self,
        current_value: f64,
        rng: &mut R,
        soft_mutation_std_dev: f64,
        hard_mutation_rate: f64,
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
                    (current_value + noise).clamp(*min, *max)
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
        }
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
    pub members: BTreeSet<Candidate>,
    pub capacity: usize,
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

    pub fn top_n(&self, n: usize) -> impl Iterator<Item = &Candidate> {
        self.members.iter().take(n)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Candidate> {
        self.members.iter()
    }

    pub fn size(&self) -> usize {
        self.members.len()
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

// trait to control dynamic parent selection over sequences of optimization
pub trait SelectionSchedule: Sync {
    // calculates the current fraction of selection of population (between 0.0 and 1.0)
    fn selection_fraction(&self, current_generation: usize, total_generations: usize) -> f64 {
        // default: linear interpolation
        let progress = current_generation as f64 / total_generations as f64;
        self.end_fraction() + (self.start_fraction() - self.end_fraction()) * (1.0 - progress)
    }

    fn estimate_evaluations(&self, total_generations: usize, population_size: usize) -> usize {
        let mut total = 0;
        for gen in 0..total_generations {
            let fraction = self
                .selection_fraction(gen, total_generations)
                .clamp(0.0, 1.0);
            let parents = (population_size as f64 * fraction).ceil() as usize;
            total += parents;
        }
        total
    }

    fn start_fraction(&self) -> f64 {
        1.0
    } // default: start with 100% of population
    fn end_fraction(&self) -> f64 {
        0.1
    } // default: end with top 10% of population
}

// linear selection (default)
pub struct LinearSchedule {
    pub start: f64,
    pub end: f64,
}

impl SelectionSchedule for LinearSchedule {
    fn start_fraction(&self) -> f64 {
        self.start
    }
    fn end_fraction(&self) -> f64 {
        self.end
    }
}

// exponential selection (e.g. for faster selection pressure)
pub struct ExponentialSchedule {
    pub start: f64,
    pub end: f64,
    pub exponent: f64, // e.g. 2.0 for quadratic, >1.0 für stronger pressure
}

impl SelectionSchedule for ExponentialSchedule {
    fn selection_fraction(&self, current_generation: usize, total_generations: usize) -> f64 {
        let progress = current_generation as f64 / total_generations as f64;
        self.end + (self.start - self.end) * (1.0 - progress.powf(self.exponent))
    }

    fn start_fraction(&self) -> f64 {
        self.start
    }
    fn end_fraction(&self) -> f64 {
        self.end
    }
}
