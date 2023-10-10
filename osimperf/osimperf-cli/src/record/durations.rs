use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Durations {
    durations: Vec<Duration>,
    mean: Option<f64>,
    stddev: Option<f64>,
}

impl Durations {
    fn set_mean(&mut self) {
        self.mean = Some(
            self.durations
                .iter()
                .map(|dt| dt.as_secs_f64())
                .sum::<f64>()
                / self.durations.len() as f64,
        )
        .filter(|_| self.durations.len() > 0)
    }

    fn set_stddev(&mut self) {
        self.stddev = self
            .mean
            .map(|mean| {
                self.durations
                    .iter()
                    .map(|dt| dt.as_secs_f64())
                    .map(|dt| dt - mean)
                    .map(|diff| diff.powi(2))
                    .sum::<f64>()
                    / (self.durations.len() as f64 - 1.)
            })
            .map(|var| var.sqrt())
            .filter(|_| self.durations.len() > 2);
    }

    pub fn clear(&mut self) {
        self.durations.clear();
        self.mean = None;
        self.stddev = None;
    }

    pub fn add_sample(&mut self, duration: Duration) {
        self.durations.push(duration);
        self.durations.sort();
        self.set_mean();
        self.set_stddev();
    }

    pub fn len(&self) -> usize {
        self.durations.len()
    }

    pub fn get(&self) -> &[Duration] {
        &self.durations
    }

    pub fn get_stddev(&self) -> Option<f64> {
        self.stddev
    }

    pub fn get_mean(&self) -> Option<f64> {
        self.mean
    }
}
