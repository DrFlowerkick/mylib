// utils to analyze log events

use super::*;

// log entry of clamp events in json format
#[derive(Debug, Deserialize)]
pub struct ClampedLogEntry {
    pub message: String,
    pub name: String,
    pub delta_clamp: f64,
}

// count min and max clamps and their deviations
#[derive(Default, Debug)]
pub struct ClampStats {
    pub min_count: usize,
    pub max_count: usize,
    pub min_deviation_sum: f64,
    pub max_deviation_sum: f64,
}

impl ClampStats {
    pub fn record(&mut self, value: f64) {
        if value < 0.0 {
            self.min_count += 1;
            self.min_deviation_sum += value;
        } else {
            self.max_count += 1;
            self.max_deviation_sum += value;
        }
    }

    pub fn print(&self, name: &str) {
        println!("\nParameter: {name}");
        if self.min_count > 0 {
            println!(
                "  Min clamps: {:3} (avg deviation: {:>+.5})",
                self.min_count,
                self.min_deviation_sum / self.min_count as f64
            );
        }
        if self.max_count > 0 {
            println!(
                "  Max clamps: {:3} (avg deviation: {:>+.5})",
                self.max_count,
                self.max_deviation_sum / self.max_count as f64
            );
        }
    }
}

// simple example tool to analyze clamp events from log entries.
// it ignores spans and only looks for messages containing "clamped".
pub fn analyze_clamp_events<T, S>(
    log_entries: Vec<T>,
) -> anyhow::Result<HashMap<String, ClampStats>>
where
    T: LogEntryParser<ClampedLogEntry, S>,
{
    let mut stats: HashMap<String, ClampStats> = HashMap::new();
    for entry in log_entries {
        if let Some(clamp_entry) = entry.get_fields()
            && clamp_entry.message.contains("clamped")
        {
            stats
                .entry(clamp_entry.name.to_owned())
                .or_default()
                .record(clamp_entry.delta_clamp);
        }
    }
    Ok(stats)
}
