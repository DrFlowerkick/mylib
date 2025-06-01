// handling populations of candidates

use super::{
    evaluate_with_shared_error, generate_random_params, ObjectiveFunction, ParamDescriptor,
    SharedError, ToleranceSettings,
};
use anyhow::Context;
use rand::Rng;
use rayon::prelude::*;
use std::cmp;
use std::collections::{BTreeSet, HashSet};
use std::fmt::Display;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use tracing::{debug, error, info, span, warn, Level};

// csv conversion trait
pub trait CsvConversion {
    fn to_csv(&self, precision: usize) -> String;
    fn from_csv(csv: &str) -> Option<Self>
    where
        Self: Sized;
}

// struct to return optimization result
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate<TS: ToleranceSettings> {
    pub params: Vec<f64>,
    pub score: f64,
    noise_score: f64,
    phantom: std::marker::PhantomData<TS>,
}

impl<TS: ToleranceSettings> Candidate<TS> {
    pub fn new(params: Vec<f64>, score: f64) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            params,
            score,
            noise_score: score + rng.gen_range(0.0..TS::epsilon()),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn describe_candidate(&self, param_bounds: &[ParamDescriptor]) -> Vec<(String, f64)> {
        self.params
            .iter()
            .enumerate()
            .map(|(i, &val)| (param_bounds[i].name.to_owned(), val))
            .collect()
    }

    pub fn is_similar_params(&self, params: &[f64], tolerance: f64) -> bool {
        if self.params.len() != params.len() {
            return false;
        }
        self.params.iter().zip(params.iter()).all(|(a, b)| {
            let denom = a.abs().max(b.abs()).max(1e-8);
            (a - b).abs() / denom < tolerance
        })
    }

    pub fn generate_offspring_params(
        &self,
        param_bounds: &[ParamDescriptor],
        hard_mutation_rate: f64,
        soft_mutation_relative_std_dev: f64,
        max_attempts: usize,
        shared_population: &SharedPopulation<TS>,
    ) -> anyhow::Result<Vec<f64>> {
        let mut rng = rand::thread_rng();
        for _ in 0..max_attempts {
            let params = param_bounds
                .iter()
                .enumerate()
                .map(|(i, pb)| {
                    pb.mutate(
                        self.params[i],
                        &mut rng,
                        hard_mutation_rate,
                        soft_mutation_relative_std_dev,
                    )
                })
                .collect::<anyhow::Result<Vec<_>>>()?;
            if !shared_population.is_similar_params(&params) {
                debug!(?params, "Generated offspring parameters");
                return Ok(params);
            }
            debug!(
                ?params,
                "Generated offspring parameters are similar to existing ones, retrying..."
            );
        }
        let random_params = generate_random_params(param_bounds)?;
        warn!(?random_params, "Failed to generate unique offspring parameters after {} attempts, Generating random offspring.", max_attempts);
        Ok(random_params)
    }
}

impl<TS: ToleranceSettings> CsvConversion for Candidate<TS> {
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
            return Some(Candidate::new(params, score));
        }
        None
    }
}

impl<TS: ToleranceSettings> Eq for Candidate<TS> {}

impl<TS: ToleranceSettings> Ord for Candidate<TS> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.score.total_cmp(&other.score) {
            cmp::Ordering::Equal => {
                if self
                    .params
                    .iter()
                    .zip(other.params.iter())
                    .map(|(a, b)| a.total_cmp(b))
                    .all(|ord| ord == cmp::Ordering::Equal)
                {
                    return cmp::Ordering::Equal;
                }
                self.noise_score.total_cmp(&other.noise_score)
            }
            ord => ord,
        }
    }
}

impl<TS: ToleranceSettings> PartialOrd for Candidate<TS> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub enum PopulationInsertResult<TS: ToleranceSettings> {
    Accepted,
    Rejected,
    Replaced(Candidate<TS>),
}

// struct of population of candidates
#[derive(Debug, Clone)]
pub struct Population<TS: ToleranceSettings> {
    members: BTreeSet<Candidate<TS>>,
    capacity: usize,
}

impl<TS: ToleranceSettings> Population<TS> {
    pub fn new(capacity: usize) -> Self {
        Self {
            members: BTreeSet::new(),
            capacity,
        }
    }

    pub fn insert(&mut self, candidate: Candidate<TS>) -> PopulationInsertResult<TS> {
        // reject if candidate is worse than the worst candidate in the population
        if let Some(smallest) = self.members.first() {
            if self.members.len() == self.capacity && candidate <= *smallest {
                return PopulationInsertResult::Rejected;
            }
        }
        self.members.insert(candidate);
        // if capacity is reached, remove worst candidate and return it
        if self.members.len() > self.capacity {
            return PopulationInsertResult::Replaced(self.members.pop_first().unwrap());
        }
        PopulationInsertResult::Accepted
    }

    pub fn populate<F: ObjectiveFunction + Sync>(
        self,
        objective: &F,
        param_bounds: &[ParamDescriptor],
        population_saver: Option<PopulationSaver>,
    ) -> anyhow::Result<Population<TS>> {
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
            let params = match param_bounds
                .iter()
                .map(|pb| pb.rng_sample(&mut rng))
                .collect::<anyhow::Result<Vec<_>>>()
            {
                Ok(params) => params,
                Err(err) => {
                    error!(error = %err, "Failed to generate random parameters");
                    shared_error.set_if_empty(err);
                    return;
                }
            };

            debug!(?params, "Generated initial candidate parameters");

            if let Some(score) = evaluate_with_shared_error(objective, &params, &shared_error) {
                debug!(score, "Initial candidate evaluated");

                shared_population.insert(
                    Candidate::new(params, score),
                    param_bounds,
                    &shared_error,
                );
            }
        });

        if let Some(err) = shared_error.take() {
            return Err(err);
        }

        shared_population.lock().save_population(param_bounds)?;
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
    ) -> anyhow::Result<Population<TS>> {
        match self.capacity.cmp(&new_capacity) {
            cmp::Ordering::Equal => (),
            cmp::Ordering::Greater => {
                self.capacity = new_capacity;
                (0..self.capacity - new_capacity).for_each(|_| {
                    self.members.pop_first();
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

    pub fn top_n(&self, n: usize) -> impl Iterator<Item = &Candidate<TS>> {
        self.members.iter().rev().take(n)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Candidate<TS>> {
        self.members.iter()
    }

    pub fn size(&self) -> usize {
        self.members.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // merge with other population considering capacity
    pub fn merge(&mut self, other: Population<TS>) {
        for candidate in other.members.into_iter() {
            self.insert(candidate);
        }
    }

    pub fn best(&self) -> Option<&Candidate<TS>> {
        self.members.last()
    }

    pub fn pop_best(&mut self) -> Option<Candidate<TS>> {
        self.members.pop_last()
    }
}

impl<TS: ToleranceSettings> CsvConversion for Population<TS> {
    fn to_csv(&self, precision: usize) -> String {
        let mut csv = String::new();

        // best candidates first
        for (index, candidate) in self.members.iter().rev().enumerate() {
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
        let candidates: Vec<Candidate<TS>> = csv.lines().filter_map(Candidate::from_csv).collect();
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
pub struct SharedPopulation<TS: ToleranceSettings> {
    inner: Arc<Mutex<InnerSharedPopulation<TS>>>,
}

impl<TS: ToleranceSettings> SharedPopulation<TS> {
    pub fn new(population: Population<TS>, population_saver: Option<PopulationSaver>) -> Self {
        SharedPopulation {
            inner: Arc::new(Mutex::new(InnerSharedPopulation {
                population,
                candidate_counter: 0,
                count_candidate_inserted_last: 0,
                population_saver,
                memory: HashSet::new(),
            })),
        }
    }
    pub fn lock(&self) -> MutexGuard<InnerSharedPopulation<TS>> {
        self.inner.lock().expect("Population lock poisoned.")
    }
    pub fn is_similar_params(&self, params: &[f64]) -> bool {
        self.lock().is_similar_params(params)
    }
    pub fn insert(
        &self,
        candidate: Candidate<TS>,
        param_bounds: &[ParamDescriptor],
        shared_error: &SharedError,
    ) -> Option<PopulationInsertResult<TS>> {
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
    pub fn top_n(&self, n: usize) -> Vec<Candidate<TS>> {
        self.lock().population.top_n(n).cloned().collect()
    }
    pub fn take(self) -> Population<TS> {
        Arc::try_unwrap(self.inner)
            .expect("Expected sole ownership of Arc")
            .into_inner()
            .expect("Population lock poisoned.")
            .population
    }
}

#[derive(Clone, Debug)]
pub struct InnerSharedPopulation<TS: ToleranceSettings> {
    population: Population<TS>,
    candidate_counter: usize,
    count_candidate_inserted_last: usize,
    population_saver: Option<PopulationSaver>,
    memory: HashSet<HashedVec<TS>>,
}

impl<TS: ToleranceSettings> InnerSharedPopulation<TS> {
    fn is_similar_params(&self, params: &[f64]) -> bool {
        // check hash of params
        let hashed = HashedVec::new(params);
        if self.memory.contains(&hashed) {
            return true;
        }

        // check if any candidate in the population is similar
        for candidate in self.population.iter() {
            if candidate.is_similar_params(params, TS::epsilon()) {
                return true;
            }
        }
        false
    }
    fn insert(
        &mut self,
        candidate: Candidate<TS>,
        param_bounds: &[ParamDescriptor],
    ) -> anyhow::Result<PopulationInsertResult<TS>> {
        // hash parameters
        let hashed = HashedVec::new(&candidate.params);
        self.memory.insert(hashed);

        // insert candidate into population and save if necessary
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
    pub fn save_population(&self, param_bounds: &[ParamDescriptor]) -> anyhow::Result<()> {
        if let Some(ref ps) = self.population_saver {
            ps.save_population(&self.population, param_bounds)
        } else {
            Ok(())
        }
    }
    pub fn best(&self) -> Option<&Candidate<TS>> {
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
    pub fn save_population<TS: ToleranceSettings>(
        &self,
        population: &Population<TS>,
        param_bounds: &[ParamDescriptor],
    ) -> anyhow::Result<()> {
        let param_names = param_bounds
            .iter()
            .map(|pd| pd.name.as_str())
            .collect::<Vec<_>>();
        save_population(population, &param_names, &self.file_path, self.precision)
    }
}

pub fn save_population<P: AsRef<Path>, TS: ToleranceSettings>(
    population: &Population<TS>,
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
    // Flush the writer to ensure all data is written to the file
    writer.flush()?;

    log_or_print(&format!("Population written to {}", path.display()));
    Ok(())
}

pub fn load_population<P: AsRef<Path>, TS: ToleranceSettings>(
    filename: P,
    has_headers: bool,
) -> anyhow::Result<(Population<TS>, Vec<String>)> {
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

    log_or_print(&format!("Results loaded from {}", path.display()));

    Ok((population, parameter_names))
}

fn log_or_print(message: &str) {
    if tracing::event_enabled!(Level::INFO) {
        tracing::info!("{}", message);
    } else {
        println!("{}", message);
    }
}

// hasher for parameters
use std::hash::{Hash, Hasher};

fn quantize(value: f64, precision: usize) -> f64 {
    let factor = 10f64.powi(precision as i32);
    (value * factor).round() / factor
}

#[derive(Clone, Debug, PartialEq)]
pub struct HashedVec<TS: ToleranceSettings> {
    pub hash_vec: Vec<u64>,
    phantom: std::marker::PhantomData<TS>,
}

impl<TS: ToleranceSettings> Eq for HashedVec<TS> {}

impl<TS: ToleranceSettings> Hash for HashedVec<TS> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for val in &self.hash_vec {
            val.hash(state);
        }
    }
}

impl<TS: ToleranceSettings> HashedVec<TS> {
    pub fn new(params: &[f64]) -> Self {
        let hashed = params
            .iter()
            .map(|&p| quantize(p, TS::precision()).to_bits())
            .collect();
        HashedVec {
            hash_vec: hashed,
            phantom: std::marker::PhantomData,
        }
    }
}
