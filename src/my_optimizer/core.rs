// traits & type definitions

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

// common trait for all optimizer
pub trait Optimizer {
    fn optimize<F: ObjectiveFunction + Sync>(
        &self,
        objective: &F,
        param_bounds: &[(f64, f64)],
        population_size: usize,
    ) -> Population;
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

// trait to control dynamic parent selection over sequences of optimization
pub trait SelectionSchedule: Sync {
    // calculates the current fraction of selection of population (between 0.0 and 1.0)
    fn selection_fraction(&self, current_generation: usize, total_generations: usize) -> f64 {
        // default: linear interpolation
        let progress = current_generation as f64 / total_generations as f64;
        self.end_fraction() + (self.start_fraction() - self.end_fraction()) * (1.0 - progress)
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
