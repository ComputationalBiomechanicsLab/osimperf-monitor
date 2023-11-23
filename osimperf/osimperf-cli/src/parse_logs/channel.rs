use super::TimeAndValue;
use anyhow::{ensure, Result};

#[derive(Debug, Default)]
pub struct Channel {
    label: String,
    data: Vec<TimeAndValue>,
}

impl Channel {
    pub fn new(label: impl ToString) -> Self {
        Self {
            data: Vec::new(),
            label: label.to_string(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn label(&self) -> &String {
        &self.label
    }

    pub fn append(&mut self, sample: TimeAndValue) -> Result<()> {
        self.data.push(sample);
        self.data
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        Ok(())
    }

    pub fn data(&self) -> &[TimeAndValue<f64>] {
        self.data.as_ref()
    }

    pub fn find_interpolate(&self, time: f64) -> Option<f64> {
        for i_after in 0..self.data.len() {
            if self.data[i_after].time <= time {
                continue;
            }
            let i_before = i_after - 1;
            let t0 = self.data[i_before].time;
            let t1 = self.data[i_after].time;
            let y0 = self.data[i_before].value;
            let y1 = self.data[i_after].value;

            let dt = t1 - t0;
            let dy = y1 - y0;

            return Some(y0 + dy / dt * (time - t0));
        }
        None
    }

    pub fn find_absolute_difference(&self, others: &[Self]) -> Option<f64> {
        others.iter().find_map(|other| self.absolute_difference(other))
    }

    pub fn absolute_difference(&self, other: &Self) -> Option<f64> {
        if other.label != self.label {
            return None;
        }

        let mut sum = 0.;
        let mut count = 0;
        for abs_diff in self
            .data
            .iter()
            .map(|x| other.find_interpolate(x.time).map(|y| y - x.value))
            .map(|d| d.unwrap_or_default().abs())
        {
            count += 1;
            sum += abs_diff;
        }
        if count == 0 {
            return Some(f64::NAN);
        }
        Some(sum / count as f64)
    }
}
