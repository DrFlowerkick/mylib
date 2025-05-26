// utils for analyzing log files created with trace log. Files must be in json format.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use chrono::NaiveDate;
use glob::glob;
use serde_json::Value;

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

// analyze all log files in dir, which fit the pattern and are in date range (optional)
pub fn analyze_clamps_from_dir(
    dir: &str,
    pattern: &str, // e.g. "patten.YYYY-MM-DD"
    date_range: Option<(NaiveDate, NaiveDate)>,
) -> anyhow::Result<()> {
    let mut stats: HashMap<String, ClampStats> = HashMap::new();

    let full_pattern = format!("{dir}/{pattern}");
    for entry in glob(&full_pattern)? {
        let path = entry?;
        if let Some(date) = extract_date_from_filename(&path) {
            if let Some((start, end)) = date_range {
                if date < start || date > end {
                    continue;
                }
            }
        }
        analyze_file(&path, &mut stats)?;
    }

    for (name, stat) in stats {
        stat.print(&name);
    }

    Ok(())
}

fn analyze_file(path: &Path, stats: &mut HashMap<String, ClampStats>) -> anyhow::Result<()> {
    let file = File::open(path)?;
    for line in BufReader::new(file).lines() {
        let line = line?;
        let json: Value = match serde_json::from_str(&line) {
            Ok(val) => val,
            Err(_) => continue,
        };
        let Some(field) = json.get("fields") else { continue; };
        let message = field.get("message").and_then(|v| v.as_str()).unwrap_or("");
        if !message.contains("clamped") {
            continue;
        }
        let name = field.get("name").and_then(|v| v.as_str());
        let delta_clamp = field
            .get("delta_clamp")
            .and_then(|v| v.as_str())
            .and_then(|v| v.parse::<f64>().ok());
        if let (Some(name), Some(value)) = (name, delta_clamp) {
            stats.entry(name.to_string()).or_default().record(value);
        }
    }
    Ok(())
}

// extract date from filename of format "patten.YYYY-MM-DD"
fn extract_date_from_filename(path: &Path) -> Option<NaiveDate> {
    path.extension()
        .and_then(|s| s.to_str())
        .and_then(|extension| NaiveDate::parse_from_str(extension, "%Y-%m-%d").ok())
}
