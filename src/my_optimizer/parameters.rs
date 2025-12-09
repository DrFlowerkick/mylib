// data types for handling parameters

use anyhow::Context;
use rand::{prelude::*, rng};
use rand_distr::Normal;
use tracing::debug;

#[derive(Clone, Debug, PartialEq)]
pub enum ParamBound {
    Static(f64),         // static value, parameter will not be changed
    MinMax(f64, f64),    // continuous value range
    MinMaxInt(f64, f64), // continuous value range, values rounded to zero decimal place
    List(Vec<f64>),      // discreet values
    LogScale(f64, f64),  // logarithmic parameter scaling
}

impl ParamBound {
    pub fn rng_sample<R: Rng + ?Sized>(&self, thread_rng: &mut R) -> anyhow::Result<f64> {
        match self {
            ParamBound::Static(val) => Ok(*val),
            ParamBound::MinMax(min, max) => {
                if max <= min {
                    return Err(anyhow::anyhow!("ParamBound::MinMax: Max <= Min"));
                }
                Ok(thread_rng.random_range(*min..=*max))
            }
            ParamBound::MinMaxInt(min, max) => {
                if max <= min {
                    return Err(anyhow::anyhow!("ParamBound::MinMax: Max <= Min"));
                }
                Ok(thread_rng.random_range(*min..=*max).round())
            }
            ParamBound::List(values) => values
                .choose(thread_rng)
                .cloned()
                .context("Empty parameter list."),
            ParamBound::LogScale(min, max) => {
                if max <= min {
                    return Err(anyhow::anyhow!("ParamBound::LogScale: Max <= Min"));
                }
                let log_min = min.ln();
                let log_max = max.ln();
                let sample_log = thread_rng.random_range(log_min..=log_max);
                Ok(sample_log.exp())
            }
        }
    }

    pub fn mutate<R: Rng + ?Sized>(
        &self,
        current_value: f64,
        thread_rng: &mut R,
        hard_mutation_rate: f64,
        soft_mutation_relative_std_dev: f64,
        name: &str,
    ) -> anyhow::Result<f64> {
        match self {
            ParamBound::Static(val) => Ok(*val), // mutation is not allowed
            ParamBound::MinMax(min, max) => {
                if max <= min {
                    return Err(anyhow::anyhow!("{} - ParamBound::MinMax: Max <= Min", name));
                }
                if thread_rng.random::<f64>() < hard_mutation_rate {
                    // hard mutation → new value in range
                    Ok(thread_rng.random_range(*min..=*max))
                } else {
                    // soft mutation → Gaussian Noise
                    let value_range = max - min;
                    let relative_std_dev = soft_mutation_relative_std_dev * value_range;
                    let noise = thread_rng.sample(Normal::new(0.0, relative_std_dev)?);
                    let value = current_value + noise;
                    let clamped = value.clamp(*min, *max);
                    if value != clamped {
                        let delta_clamp = value - clamped;
                        debug!(%name, delta_clamp, "Value clamped to bounds.");
                    }
                    Ok(clamped)
                }
            }
            ParamBound::MinMaxInt(min, max) => {
                if max <= min {
                    return Err(anyhow::anyhow!("{} - ParamBound::MinMax: Max <= Min", name));
                }
                if thread_rng.random::<f64>() < hard_mutation_rate {
                    // hard mutation → new value in range
                    Ok(thread_rng.random_range(*min..=*max).round())
                } else {
                    // soft mutation → Gaussian Noise
                    let value_range = max - min;
                    let relative_std_dev = soft_mutation_relative_std_dev * value_range;
                    let noise = thread_rng.sample(Normal::new(0.0, relative_std_dev)?);
                    let value = current_value + noise;
                    let clamped = value.clamp(*min, *max);
                    if value != clamped {
                        let delta_clamp = value - clamped;
                        debug!(%name, delta_clamp, "Value clamped to bounds.");
                    }
                    Ok(clamped.round())
                }
            }
            ParamBound::List(values) => {
                if thread_rng.random::<f64>() < hard_mutation_rate {
                    // hard mutation → random value from list
                    values
                        .choose(thread_rng)
                        .cloned()
                        .context(format!("{} - Parameter list is empty!", name))
                } else {
                    // soft mutation → choose value nearest to current value plus noise
                    let min = values.iter().cloned().reduce(f64::min).unwrap();
                    let max = values.iter().cloned().reduce(f64::max).unwrap();
                    let value_range = max - min;
                    let relative_std_dev = soft_mutation_relative_std_dev * value_range;
                    let noise = thread_rng.sample(Normal::new(0.0, relative_std_dev)?);
                    let target_value = current_value + noise;

                    values
                        .iter()
                        .min_by(|&&a, &&b| {
                            (a - target_value)
                                .abs()
                                .total_cmp(&(b - target_value).abs())
                        })
                        .cloned()
                        .context(format!("{} - Parameter list is empty!", name))
                }
            }
            ParamBound::LogScale(min, max) => {
                if max <= min {
                    return Err(anyhow::anyhow!(
                        "{} - ParamBound::LogScale: Max <= Min",
                        name
                    ));
                }
                let log_min = min.ln();
                let log_max = max.ln();
                let current_log = current_value.ln();

                if thread_rng.random::<f64>() < hard_mutation_rate {
                    // hard mutation → random value in log range
                    let new_log = thread_rng.random_range(log_min..=log_max);
                    Ok(new_log.exp())
                } else {
                    // soft mutation → Gaussian Noise in log range
                    let log_range = max - min;
                    let relative_std_dev = soft_mutation_relative_std_dev * log_range;
                    let noise = thread_rng.sample(Normal::new(0.0, relative_std_dev)?);
                    let value = (current_log + noise).exp();
                    let clamped = value.clamp(*min, *max);
                    if value != clamped {
                        let delta_clamp = value - clamped;
                        debug!(%name, delta_clamp, "Value clamped to bounds.");
                    }
                    Ok(clamped)
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
    pub fn rng_sample<R: Rng + ?Sized>(&self, thread_rng: &mut R) -> anyhow::Result<f64> {
        self.bound.rng_sample(thread_rng)
    }
    pub fn mutate<R: Rng + ?Sized>(
        &self,
        current_value: f64,
        thread_rng: &mut R,
        hard_mutation_rate: f64,
        soft_mutation_relative_std_dev: f64,
    ) -> anyhow::Result<f64> {
        self.bound.mutate(
            current_value,
            thread_rng,
            hard_mutation_rate,
            soft_mutation_relative_std_dev,
            &self.name,
        )
    }
}

pub fn generate_random_params(param_bounds: &[ParamDescriptor]) -> anyhow::Result<Vec<f64>> {
    let mut thread_rng = rng();
    param_bounds
        .iter()
        .map(|pb| pb.rng_sample(&mut thread_rng))
        .collect()
}
