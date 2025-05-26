// data types for handling parameters

use anyhow::Context;
use rand::prelude::*;
use rand_distr::Normal;
use tracing::debug;

#[derive(Clone, Debug, PartialEq)]
pub enum ParamBound {
    Static(f64),        // static value, parameter will not be changed
    MinMax(f64, f64),   // continuous value range
    List(Vec<f64>),     // discreet values
    LogScale(f64, f64), // logarithmic parameter scaling
}

impl ParamBound {
    pub fn rng_sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        match self {
            ParamBound::Static(val) => *val,
            ParamBound::MinMax(min, max) => rng.gen_range(*min..=*max),
            ParamBound::List(values) => *values.choose(rng).expect("Empty parameter list."),
            ParamBound::LogScale(min, max) => {
                let log_min = min.ln();
                let log_max = max.ln();
                let sample_log = rng.gen_range(log_min..=log_max);
                sample_log.exp()
            }
        }
    }

    pub fn mutate<R: Rng + ?Sized>(
        &self,
        current_value: f64,
        rng: &mut R,
        soft_mutation_std_dev: f64,
        hard_mutation_rate: f64,
        name: &str,
    ) -> anyhow::Result<f64> {
        match self {
            ParamBound::Static(val) => Ok(*val), // mutation is not allowed
            ParamBound::MinMax(min, max) => {
                if max <= min {
                    return Err(anyhow::anyhow!("ParamBound::MinMax: Max <= Min"));
                }
                if rng.gen::<f64>() < hard_mutation_rate {
                    // hard mutation → new value in range
                    Ok(rng.gen_range(*min..=*max))
                } else {
                    // soft mutation → Gaussian Noise
                    let noise = rng.sample(Normal::new(0.0, soft_mutation_std_dev)?);
                    let value = current_value + noise;
                    let clamped = value.clamp(*min, *max);
                    if value != clamped {
                        let delta_clamp = value - clamped;
                        debug!(%name, %delta_clamp, "Value clamped to bounds.");
                    }
                    Ok(clamped)
                }
            }
            ParamBound::List(values) => {
                if rng.gen::<f64>() < hard_mutation_rate {
                    // hard mutation → random value from list
                    values
                        .choose(rng)
                        .cloned()
                        .context("Parameter list is empty!")
                } else {
                    // soft mutation → choose value nearest to current value plus noise
                    let noise = rng.sample(Normal::new(0.0, soft_mutation_std_dev)?);
                    let target_value = current_value + noise;

                    values
                        .iter()
                        .min_by(|&&a, &&b| {
                            (a - target_value)
                                .abs()
                                .total_cmp(&(b - target_value).abs())
                        })
                        .cloned()
                        .context("Parameter list is empty!")
                }
            }
            ParamBound::LogScale(min, max) => {
                if max <= min {
                    return Err(anyhow::anyhow!("ParamBound::LogScale: Max <= Min"));
                }
                let log_min = min.ln();
                let log_max = max.ln();
                let current_log = current_value.ln();

                if rng.gen::<f64>() < hard_mutation_rate {
                    // hard mutation → random value in log range
                    let new_log = rng.gen_range(log_min..=log_max);
                    Ok(new_log.exp())
                } else {
                    // soft mutation → Gaussian Noise in log range
                    let noise = rng.sample(Normal::new(0.0, soft_mutation_std_dev)?);
                    let value = (current_log + noise).exp();
                    let clamped = value.clamp(*min, *max);
                    if value != clamped {
                        let delta_clamp = value - clamped;
                        debug!(%name, %delta_clamp, "Value clamped to bounds.");
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
    pub fn rng_sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        self.bound.rng_sample(rng)
    }
    pub fn mutate<R: Rng + ?Sized>(
        &self,
        current_value: f64,
        rng: &mut R,
        soft_mutation_std_dev: f64,
        hard_mutation_rate: f64,
    ) -> anyhow::Result<f64> {
        self.bound.mutate(
            current_value,
            rng,
            soft_mutation_std_dev,
            hard_mutation_rate,
            &self.name,
        )
    }
}
