// schedule from start to end value of progress

use std::fmt::{Debug, Display};

// trait to schedule progress from a start to an end value over sequences of optimization / exploration
pub trait Schedule: Sync + Send + Debug + Display {
    fn value_at(&self, current_sequence: usize, total_sequences: usize) -> f64;
    fn start_schedule(&self) -> f64;
    fn end_schedule(&self) -> f64;
}

// constant schedule
#[derive(Clone, Debug)]
pub struct ConstantSchedule {
    pub value: f64,
}

impl Display for ConstantSchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Constant({:.3})", self.value)
    }
}

impl Schedule for ConstantSchedule {
    fn value_at(&self, _current_sequence: usize, _total_sequences: usize) -> f64 {
        self.value.clamp(0.0, 1.0)
    }

    fn start_schedule(&self) -> f64 {
        self.value
    }
    fn end_schedule(&self) -> f64 {
        self.value
    }
}

// linear selection
#[derive(Clone, Debug)]
pub struct LinearSchedule {
    pub start: f64,
    pub end: f64,
}

impl Display for LinearSchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Linear({:.3} → {:.3})", self.start, self.end)
    }
}

impl Schedule for LinearSchedule {
    fn value_at(&self, current_sequence: usize, total_sequences: usize) -> f64 {
        let progress = current_sequence as f64 / total_sequences as f64;
        self.end_schedule() + (self.start_schedule() - self.end_schedule()) * (1.0 - progress)
    }
    fn start_schedule(&self) -> f64 {
        self.start
    }
    fn end_schedule(&self) -> f64 {
        self.end
    }
}

// exponential schedule
#[derive(Clone, Debug)]
pub struct ExponentialSchedule {
    pub start: f64,
    pub end: f64,
    // exponent > 1.0: slower transition at start with faster reaching of end
    // exponent = 1.0: linear transition
    // exponent < 1.0: faster transition at start with slower reaching of end
    pub exponent: f64,
}

impl Display for ExponentialSchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Exponential({:.3} → {:.3}, exponent: {:.3})",
            self.start, self.end, self.exponent
        )
    }
}

impl Schedule for ExponentialSchedule {
    fn value_at(&self, current_sequence: usize, total_sequences: usize) -> f64 {
        let progress = current_sequence as f64 / total_sequences as f64;
        self.end + (self.start - self.end) * (1.0 - progress.powf(self.exponent))
    }

    fn start_schedule(&self) -> f64 {
        self.start
    }
    fn end_schedule(&self) -> f64 {
        self.end
    }
}

// decay schedule for values that decay over time, e.g. learning rate
#[derive(Clone, Debug)]
pub struct DecaySchedule {
    pub start: f64,
    pub end: f64,
    pub decay: f64, // e.g. 0.95
}
impl Display for DecaySchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Decay({:.3} → {:.3}, decay: {:.3})",
            self.start, self.end, self.decay
        )
    }
}
impl Schedule for DecaySchedule {
    fn value_at(&self, generation: usize, _total: usize) -> f64 {
        self.end + (self.start - self.end) * self.decay.powf(generation as f64)
    }

    fn start_schedule(&self) -> f64 {
        self.start
    }
    fn end_schedule(&self) -> f64 {
        self.end
    }
}

// sigmoid schedule
#[derive(Clone, Debug)]
pub struct SigmoidSchedule {
    pub start: f64,
    pub end: f64,
    pub steepness: f64, // steeper = faster transition, e.g 10.0
}

impl Display for SigmoidSchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Sigmoid({:.3} → {:.3}, steepness: {:.3})",
            self.start, self.end, self.steepness
        )
    }
}

impl Schedule for SigmoidSchedule {
    fn value_at(&self, current_sequence: usize, total_sequences: usize) -> f64 {
        // range of progress is [0.0, 1.0]
        let progress = current_sequence as f64 / total_sequences as f64;
        // shift progress to [-0.5, 0.5] for sigmoid calculation and apply steepness
        let x = (progress - 0.5) * self.steepness;
        // sigmoid function: 1 / (1 + exp(-x))
        let sigmoid = 1.0 / (1.0 + (-x).exp());
        // use 1.0 - sigmoid to invert the sigmoid curve
        self.end + (self.start - self.end) * (1.0 - sigmoid)
    }

    fn start_schedule(&self) -> f64 {
        self.start
    }
    fn end_schedule(&self) -> f64 {
        self.end
    }
}
