// handling populations of candidates

use super::{evaluate_with_shared_error, ObjectiveFunction, ParamDescriptor, SharedError};
use anyhow::Context;
use rayon::prelude::*;
use std::cmp;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use tracing::{debug, error, info, span, Level};

// csv conversion trait
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
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.score.total_cmp(&other.score).reverse()
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub enum PopulationInsertResult {
    Accepted,
    Rejected,
    Replaced(Candidate),
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
    pub fn insert(&mut self, candidate: Candidate) -> PopulationInsertResult {
        if let Some(smallest) = self.members.last() {
            if candidate <= *smallest {
                return PopulationInsertResult::Rejected;
            }
        }
        self.members.insert(candidate);
        if self.members.len() > self.capacity {
            return PopulationInsertResult::Replaced(self.members.pop_last().unwrap());
        }
        PopulationInsertResult::Accepted
    }

    pub fn populate<F: ObjectiveFunction + Sync>(
        self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
        population_saver: Option<PopulationSaver>,
    ) -> anyhow::Result<Population> {
        // since objective may be a costly objective, tracing is used to signal status of populate
        let init_span = span!(Level::INFO, "PopulatePopulation", size = self.capacity);
        let _enter = init_span.enter();

        let current_population_size = self.members.len();

        if current_population_size == self.capacity {
            info!(
                "Population is already populated with {} candidates",
                self.capacity
            );
            return Ok(self);
        }

        if current_population_size > 0 {
            info!(
                "Population contains already {} candidates.",
                current_population_size,
            );
        }

        info!("Populating population until {} candidates", self.capacity);
        let remaining_candidates = self.capacity - current_population_size;
        let shared_population = SharedPopulation::new(self, population_saver);
        let shared_error = SharedError::new();

        (0..remaining_candidates).into_par_iter().for_each(|_| {
            if shared_error.is_set() {
                return;
            }
            let mut rng = rand::thread_rng();
            let params: Vec<f64> = param_bounds
                .iter()
                .map(|pb| pb.rng_sample(&mut rng))
                .collect();
            debug!(?params, "Generated initial candidate parameters");

            if let Some(score) = evaluate_with_shared_error(objective, &params, &shared_error) {
                debug!(score, "Initial candidate evaluated");

                shared_population.insert(Candidate { params, score }, param_bounds, &shared_error);
            }
        });

        if let Some(err) = shared_error.take() {
            return Err(err);
        }

        let population = shared_population.take();

        info!(
            "Population fully populated. Best Score: {:.3}",
            population
                .best()
                .map(|c| c.score)
                .context("Empty population")?
        );

        Ok(population)
    }

    pub fn resize_population<F: ObjectiveFunction + Sync>(
        mut self,
        new_capacity: usize,
        populate_with: Option<(&F, &[ParamDescriptor])>,
        population_saver: Option<PopulationSaver>,
    ) -> anyhow::Result<Population> {
        match self.capacity.cmp(&new_capacity) {
            cmp::Ordering::Equal => (),
            cmp::Ordering::Greater => {
                self.capacity = new_capacity;
                (0..self.capacity - new_capacity).for_each(|_| {
                    self.members.pop_last();
                });
            }
            cmp::Ordering::Less => {
                if let Some((objective, param_bounds)) = populate_with {
                    self = self.populate(objective, param_bounds, population_saver)?;
                }
            }
        }
        Ok(self)
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

// thread safe population handling
#[derive(Clone)]
pub struct SharedPopulation {
    inner: Arc<Mutex<InnerSharedPopulation>>,
}

impl SharedPopulation {
    pub fn new(population: Population, population_saver: Option<PopulationSaver>) -> Self {
        SharedPopulation {
            inner: Arc::new(Mutex::new(InnerSharedPopulation {
                population,
                candidate_counter: 0,
                count_candidate_inserted_last: 0,
                population_saver,
            })),
        }
    }
    pub fn lock(&self) -> MutexGuard<InnerSharedPopulation> {
        self.inner.lock().expect("Population lock poisoned.")
    }
    pub fn insert(
        &self,
        candidate: Candidate,
        param_bounds: &[ParamDescriptor],
        shared_error: &SharedError,
    ) -> Option<PopulationInsertResult> {
        let insert_result = self.lock().insert(candidate, param_bounds);
        match insert_result {
            Ok(pir) => Some(pir),
            Err(e) => {
                error!(
                    error = %e,
                    "Insert candidate failed, aborting..."
                );
                shared_error.set_if_empty(e);
                None
            }
        }
    }
    pub fn top_n(&self, n: usize) -> Vec<Candidate> {
        self.lock().population.top_n(n).cloned().collect()
    }
    pub fn take(self) -> Population {
        Arc::try_unwrap(self.inner)
            .expect("Expected sole ownership of Arc")
            .into_inner()
            .expect("Population lock poisoned.")
            .population
    }
}

#[derive(Clone, Debug)]
pub struct InnerSharedPopulation {
    population: Population,
    candidate_counter: usize,
    count_candidate_inserted_last: usize,
    population_saver: Option<PopulationSaver>,
}

impl InnerSharedPopulation {
    fn insert(
        &mut self,
        candidate: Candidate,
        param_bounds: &[ParamDescriptor],
    ) -> anyhow::Result<PopulationInsertResult> {
        self.candidate_counter += 1;
        let insert_result = self.population.insert(candidate);
        if let Some(ref ps) = self.population_saver {
            if self.candidate_counter - self.count_candidate_inserted_last > ps.step_size
                && matches!(
                    insert_result,
                    PopulationInsertResult::Accepted | PopulationInsertResult::Replaced(_)
                )
            {
                self.count_candidate_inserted_last = self.candidate_counter;
                ps.save_population(&self.population, param_bounds)?;
            }
        }
        Ok(insert_result)
    }
    pub fn best(&self) -> Option<&Candidate> {
        self.population.best()
    }
}

// helper to save population
#[derive(Debug, Clone)]
pub struct PopulationSaver {
    pub file_path: PathBuf,
    pub step_size: usize,
    pub precision: usize,
}

impl PopulationSaver {
    pub fn save_population(
        &self,
        population: &Population,
        param_bounds: &[ParamDescriptor],
    ) -> anyhow::Result<()> {
        let param_names = param_bounds
            .iter()
            .map(|pd| pd.name.as_str())
            .collect::<Vec<_>>();
        save_population(population, &param_names, &self.file_path, self.precision)
    }
}

pub fn save_population<P: AsRef<Path>>(
    population: &Population,
    param_names: &[impl Display],
    filename: P,
    precision: usize,
) -> anyhow::Result<()> {
    let path = filename.as_ref();
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    if !param_names.is_empty() {
        let header = param_names
            .iter()
            .map(|name| name.to_string())
            .collect::<Vec<String>>()
            .join(",");

        writeln!(writer, "{},average_score", header)?;
    }

    writeln!(writer, "{}", population.to_csv(precision))?;

    log_or_print(&format!("Population written to {}", path.display()));
    Ok(())
}

pub fn load_population<P: AsRef<Path>>(
    filename: P,
    has_headers: bool,
) -> anyhow::Result<(Population, Vec<String>)> {
    let path = filename.as_ref();
    let csv = std::fs::read_to_string(path)?;

    let (parameter_names, csv) = if has_headers {
        let (parameter_names, csv) = csv.split_once('\n').context("No new line in file")?;
        let mut parameter_names = parameter_names
            .split(',')
            .map(|pn| pn.to_string())
            .collect::<Vec<_>>();
        // remove "average_score" at the end
        parameter_names.pop();
        (parameter_names, csv)
    } else {
        (vec![], csv.as_str())
    };

    let population = Population::from_csv(csv).context("Empty population")?;

    log_or_print(&format!("Results written to {}", path.display()));

    Ok((population, parameter_names))
}

fn log_or_print(message: &str) {
    if tracing::event_enabled!(Level::INFO) {
        tracing::info!("{}", message);
    } else {
        println!("{}", message);
    }
}
