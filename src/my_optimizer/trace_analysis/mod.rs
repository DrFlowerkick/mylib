// utils for analyzing log files created with trace log. Files must be in json format.

pub mod clamp_events;
pub use clamp_events::{analyze_clamp_events, ClampStats, ClampedLogEntry};

pub mod mutation_stats;
pub use mutation_stats::{
    analyze_evo_log_entries, EvoFields, EvoSpan, MutationKey, MutationParentAndOffspring,
    MutationStats,
};

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::Context;
use chrono::NaiveDate;
use glob::glob;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

// trait to parse one line of log entry from json format
pub trait LogEntryParser<F, S>: Sized {
    fn from_json_line(line: &str) -> anyhow::Result<Self>;
    fn get_message(&self) -> Option<&str>;
    fn get_fields(&self) -> Option<&F>;
    fn get_span(&self) -> Option<&S>;
    fn get_spans(&self) -> Option<&Vec<S>>;
}

// default log entry parser for json format
#[derive(Debug)]
pub struct DefaultLogEntry<F = Value, S = Value> {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: Option<String>,
    pub fields: Option<F>,
    pub span: Option<S>,
    pub spans: Option<Vec<S>>,
}

impl<'de, F, S> Deserialize<'de> for DefaultLogEntry<F, S>
where
    F: Deserialize<'de>,
    S: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper<F, S> {
            timestamp: String,
            level: String,
            target: String,
            message: Option<String>,
            fields: Option<F>,
            span: Option<S>,
            spans: Option<Vec<S>>,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(DefaultLogEntry {
            timestamp: helper.timestamp,
            level: helper.level,
            target: helper.target,
            message: helper.message,
            fields: helper.fields,
            span: helper.span,
            spans: helper.spans,
        })
    }
}

impl<F, S> LogEntryParser<F, S> for DefaultLogEntry<F, S>
where
    F: serde::de::DeserializeOwned,
    S: serde::de::DeserializeOwned,
{
    fn from_json_line(line: &str) -> anyhow::Result<Self> {
        let entry = serde_json::from_str(line)?;
        Ok(entry)
    }
    fn get_message(&self) -> Option<&str> {
        self.message.as_deref()
    }
    fn get_fields(&self) -> Option<&F> {
        self.fields.as_ref()
    }
    fn get_span(&self) -> Option<&S> {
        self.span.as_ref()
    }
    fn get_spans(&self) -> Option<&Vec<S>> {
        self.spans.as_ref()
    }
}

// read one log file and parse it into a vector of entries
pub fn read_log_file<P, T, F, S>(path: P) -> anyhow::Result<Vec<T>>
where
    P: AsRef<std::path::Path>,
    T: LogEntryParser<F, S>,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut entries = Vec::new();
    for line in reader.lines() {
        let line = line?;
        match T::from_json_line(&line) {
            Ok(entry) => entries.push(entry),
            Err(err) => {
                eprintln!("Skipping invalid log line: {err}");
                continue;
            }
        }
    }

    Ok(entries)
}

// extract date from filename of format "patten.YYYY-MM-DD"
fn extract_date_from_filename(path: &Path) -> Option<NaiveDate> {
    path.extension()
        .and_then(|s| s.to_str())
        .and_then(|extension| NaiveDate::parse_from_str(extension, "%Y-%m-%d").ok())
}

// read log files from dir, which fit the pattern and are in date range (optional)
pub fn read_logs_from_dir<P, T, F, S>(
    dir: P,
    pattern: &str, // e.g. "patten.YYYY-MM-DD"
    date_range: Option<(NaiveDate, NaiveDate)>,
) -> anyhow::Result<Vec<T>>
where
    P: AsRef<std::path::Path>,
    T: LogEntryParser<F, S>,
{
    let dir = dir
        .as_ref()
        .as_os_str()
        .to_str()
        .context("Invalid directory path")?;
    let full_pattern = format!("{dir}/{pattern}");
    let mut log_entries = Vec::new();
    for entry in glob(&full_pattern)? {
        let path = entry?;
        if let Some(date) = extract_date_from_filename(&path) {
            if let Some((start, end)) = date_range {
                if date < start || date > end {
                    continue;
                }
            }
        }
        log_entries.append(&mut read_log_file(path)?);
    }

    Ok(log_entries)
}
