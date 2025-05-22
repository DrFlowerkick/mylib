// utils for analyzing log files created with trace log. Files must be in json format.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use chrono::NaiveDate;
use glob::glob;
use serde_json::Value;

use super::{ParamBound, ParamDescriptor};

#[derive(Default, Debug)]
pub struct ClampStats {
    pub min_count: usize,
    pub max_count: usize,
    pub min_deviation_sum: f64,
    pub max_deviation_sum: f64,
}

impl ClampStats {
    pub fn record(&mut self, value: f64, bound: &ParamBound) {
        match bound {
            ParamBound::MinMax(min, max) | ParamBound::LogScale(min, max) => {
                if (value - min).abs() < 1e-9 {
                    self.min_count += 1;
                    self.min_deviation_sum += value - min;
                } else if (value - max).abs() < 1e-9 {
                    self.max_count += 1;
                    self.max_deviation_sum += value - max;
                }
            }
            _ => {}
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
    param_bounds: &[ParamDescriptor],
) -> anyhow::Result<()> {
    let mut stats: HashMap<String, ClampStats> = HashMap::new();
    let name_to_bound: HashMap<&str, &ParamBound> = param_bounds
        .iter()
        .map(|desc| (desc.name.as_str(), &desc.bound))
        .collect();

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

        analyze_file(&path, &name_to_bound, &mut stats)?;
    }

    for (name, stat) in stats {
        stat.print(&name);
    }

    Ok(())
}

fn analyze_file(
    path: &Path,
    name_to_bound: &HashMap<&str, &ParamBound>,
    stats: &mut HashMap<String, ClampStats>,
) -> anyhow::Result<()> {
    let file = File::open(path)?;
    for line in BufReader::new(file).lines() {
        let line = line?;
        let json: Value = match serde_json::from_str(&line) {
            Ok(val) => val,
            Err(_) => continue,
        };

        let message = json.get("message").and_then(|v| v.as_str()).unwrap_or("");
        if !message.contains("clamped") {
            continue;
        }

        let name = json.get("name").and_then(|v| v.as_str());
        let value = json.get("value").and_then(|v| v.as_f64());
        if let (Some(name), Some(value)) = (name, value) {
            if let Some(bound) = name_to_bound.get(name) {
                stats
                    .entry(name.to_string())
                    .or_default()
                    .record(value, bound);
            }
        }
    }
    Ok(())
}

// extract date from filename of format "patten.YYYY-MM-DD"
fn extract_date_from_filename(path: &Path) -> Option<NaiveDate> {
    path.file_stem().and_then(|s| s.to_str()).and_then(|stem| {
        let parts: Vec<&str> = stem.split('.').collect();
        if parts.len() >= 2 {
            NaiveDate::parse_from_str(parts[1], "%Y-%m-%d").ok()
        } else {
            None
        }
    })
}
