// utils to analyze mutation stats from log entries
use super::*;
use crate::my_optimizer::ObjectiveFunction;

// span struct, which supports all spans of evolutionary optimizer
#[derive(Debug, Deserialize)]
pub struct EvoSpan {
    pub name: String,
    pub generations: Option<usize>,
    pub generation: Option<usize>,
    pub id: Option<usize>,
}

// fields struct, which supports all fields of evolutionary optimizer
#[derive(Debug, Deserialize)]
pub struct EvoFields {
    pub message: String,
    pub config: Option<String>,
    pub score: Option<f64>,
    pub id: Option<usize>,
    pub parent_count: Option<usize>,
    pub hard_mutation_rate: Option<f64>,
    pub soft_mutation_relative_std_dev: Option<f64>,
}

#[derive(Debug)]
pub struct MutationParentAndOffspring<F: ObjectiveFunction> {
    pub parent_config: F::Config,
    pub parent_score: f64,
    pub offspring_config: F::Config,
    pub offspring_score: f64,
}

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone, Copy)]
pub struct MutationKey {
    pub generation: usize,
    pub id: usize,
}

pub type MutationStats<F> = HashMap<MutationKey, MutationParentAndOffspring<F>>;

pub fn analyze_evo_log_entries<T, F>(log_entries: Vec<T>) -> anyhow::Result<MutationStats<F>>
where
    T: LogEntryParser<EvoFields, EvoSpan>,
    F: ObjectiveFunction,
{
    let mut stats: MutationStats<F> = HashMap::new();
    let mut parents: HashMap<MutationKey, (F::Config, f64)> = HashMap::new();
    let mut offspring: HashMap<MutationKey, (F::Config, f64)> = HashMap::new();

    for entry in log_entries {
        if let Some(fields) = entry.get_fields() {
            let is_parent = match fields.message.as_str() {
                "Selected Parent" => true,
                "Evaluated candidate" => false,
                _ => continue, // Ignore other messages
            };
            let Some(span) = entry.get_span() else {
                continue; // Skip entries without a span
            };
            let Some(generation) = span.generation else {
                continue; // Skip entries without a generation
            };
            let Some(id) = span.id else {
                continue; // Skip entries without an ID
            };
            let Some(config) = fields.config.clone() else {
                continue; // Skip entries without a config
            };
            // Deserialize the config string into the appropriate type
            let config: F::Config = serde_json::from_str(&config)?;
            let Some(score) = fields.score else {
                continue; // Skip entries without a score
            };
            let key = MutationKey { generation, id };
            if is_parent {
                // Store parent information
                parents.insert(key, (config, score));
            } else {
                // Store offspring information
                offspring.insert(key, (config, score));
            }
        }
    }

    // Combine parents and offspring into stats
    for (key, (parent_config, parent_score)) in parents {
        if let Some((offspring_config, offspring_score)) = offspring.remove(&key) {
            stats.insert(
                key,
                MutationParentAndOffspring {
                    parent_config,
                    parent_score,
                    offspring_config,
                    offspring_score,
                },
            );
        } else {
            // If no offspring found for this parent, we can log or handle it as needed
            return Err(anyhow::anyhow!(
                "No offspring found for parent with key: {:?}",
                key
            ));
        }
    }

    Ok(stats)
}
