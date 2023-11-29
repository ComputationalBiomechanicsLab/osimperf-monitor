use crate::parse_logs::TimeAndValue;

use super::channel::Channel;
use anyhow::{ensure, Context, Result};
use std::io::{BufRead, BufReader, Read};

#[derive(Debug, Default)]
pub struct Data {
    channels: Vec<Channel>,
}

impl Data {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
        }
    }

    pub fn print_info(&self) {
        println!(
            "Log info:\n    number of channels = {}",
            self.channels.len()
        );
        let mut s = String::from("    \nlabels[");

        for c in self.channels.iter() {
            s.push_str(",\n    ");
            s.push_str(c.label());
            let len = format!(" (len = {})", c.len());
            s.push_str(&len);
        }
        println!("{}", s);
    }

    pub fn channels(&self) -> &[Channel] {
        &self.channels
    }

    pub fn find_label(&self, label: &str) -> Option<usize> {
        for (i, c) in self.channels.iter().enumerate() {
            if c.label() == label {
                return Some(i);
            }
        }
        None
    }

    fn get_labeled_or(&mut self, label: &str) -> usize {
        let i = if let Some(i) = self.find_label(label) {
            i
        } else {
            self.channels.push(Channel::new(String::from(label)));
            self.channels.len() - 1
        };
        i
    }

    pub fn add_sample(
        &mut self,
        label: &str,
        time_and_value: super::TimeAndValue,
    ) -> Result<usize> {
        let index = self.get_labeled_or(label);
        self.channels[index]
            .append(time_and_value)
            .context(format!("error adding sample to channel {:?}", label))?;
        Ok(index)
    }

    pub fn channels_mut(&mut self) -> &mut Vec<Channel> {
        &mut self.channels
    }

    // Dynamically allocate a new Vector with current labels.
    pub fn new_labels(&self) -> Vec<String> {
        self.channels.iter().map(|c| c.label().clone()).collect()
    }

    pub fn read_opensim_file(buffer: impl Read) -> Result<Self> {
        let mut line = String::new();

        let mut reader = BufReader::new(buffer);

        // Find end of header.
        while !line.contains("endheader") {
            line.clear();
            reader.read_line(&mut line)?;
        }

        // Read labels.
        let mut labels = Vec::new();
        line.clear();
        reader.read_line(&mut line).context("no labels present")?;
        let mut words = line.split("\t");
        ensure!(
            words.next().context("no labels found")?.trim() == "time",
            "expected first label to be time"
        );
        for label in words {
            labels.push(String::from(label.trim()));
        }

        // Read data.
        let mut data = Data::new();
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                break;
            }

            let mut words = line.trim().split("\t");

            let time = words
                .next()
                .context("failed to read time")?
                .trim()
                .parse::<f64>()
                .context("failed to parse time")?;
            for label in labels.iter_mut() {
                let svalue = words.next().context("failed to read value")?.trim();
                let value = svalue.parse::<f64>().context("failed to parse value")?;
                data.add_sample(label.trim(), TimeAndValue { time, value })?;
            }
        }

        Ok(data)
    }
}

#[derive(Debug)]
pub struct ChannelDiff {
    pub label: String,
    pub diff: Option<f64>,
}

#[derive(Debug)]
pub struct Diff {
    pub channels: Vec<ChannelDiff>,
}

impl Diff {
    pub fn new(a: &Data, b: &Data) -> Result<Self> {
        let mut channels = Vec::new();
        for channel in a.channels.iter() {
            let label = channel.label().to_owned();
            let diff = if let Some(diff_a) = channel.find_absolute_difference(&b.channels) {
                let i = b
                    .find_label(channel.label())
                    .with_context(|| format!("found label in b but not in a"))?;
                if let Some(diff_b) = b.channels[i].find_absolute_difference(&a.channels) {
                    Some((diff_a + diff_b) / 2.)
                } else {
                    Some(diff_a)
                }
            } else {
                None
            };
            channels.push(ChannelDiff { label, diff });
        }
        Ok(Self { channels })
    }
}
