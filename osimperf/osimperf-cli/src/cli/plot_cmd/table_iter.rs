use super::ArgOrStdinIter;
use super::ResultInfo;
use anyhow::{Context, Result};
use clap::Args;
use std::{fs::File, path::PathBuf};

#[derive(Clone, Debug, Default)]
pub struct Table {
    pub installed: Vec<InstallNode>,
    pub benchmarks: Vec<BenchmarkNode>,
    pub results: Vec<ResultInfo>,
    pub reference: Option<Vec<ResultInfo>>,
}

impl Table {
    pub fn new(arg_path: &Option<PathBuf>, ref_name: &Option<String>) -> Result<Self> {
        let mut reference: Vec<ResultInfo> = Vec::new();
        let mut out = Self::default();
        for path in ArgOrStdinIter::new(arg_path) {
            let result = crate::read_json::<ResultInfo>(&path)?;

            if ref_name.as_ref() == Some(&result.opensim_name) {
                reference.push(result.clone());
            }

            let cell_name = result.cell_name.clone().unwrap_or(result.name.clone());

            if out
                .benchmarks
                .iter()
                .find(|&x| x.name == result.name)
                .is_none()
            {
                out.benchmarks.push(BenchmarkNode {
                    name: result.name.clone(),
                    cell_name,
                });
            }

            if out
                .installed
                .iter()
                .find(|x| x.name == result.opensim_name)
                .is_none()
            {
                out.installed.push(InstallNode {
                    name: result.opensim_name.clone(),
                    date: result.date.clone(),
                    cell_name: format!("{} ({})", result.opensim_name, result.date),
                });
            }

            out.results.push(result);
        }
        reference.sort_by(|a, b| b.date.cmp(&a.date));
        if reference.len() > 0 {
            out.reference = Some(reference);
        }
        Ok(out)
    }
}

impl<'a> IntoIterator for &'a Table {
    type Item = ColIterator<'a>;
    type IntoIter = RowIterator<'a>;

    // Required method
    fn into_iter(self) -> Self::IntoIter {
        RowIterator {
            table: self,
            direction: TableOrientation::BenchmarksOnRow,
            index: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BenchmarkNode {
    name: String,
    cell_name: String,
}

#[derive(Clone, Debug)]
pub struct InstallNode {
    name: String,
    date: String,
    cell_name: String,
}

#[derive(Copy, Clone, Debug)]
enum TableOrientation {
    InstallsOnRow,
    BenchmarksOnRow,
}

pub struct RowIterator<'a> {
    table: &'a Table,
    direction: TableOrientation,
    index: Option<usize>,
}

impl<'a> RowIterator<'a> {
    pub fn row_name(&self) -> &'a str {
        let index = self.index.unwrap_or_default();
        match self.direction {
            TableOrientation::InstallsOnRow => &self.table.installed[index].cell_name,
            TableOrientation::BenchmarksOnRow => &self.table.benchmarks[index].name,
        }
    }
}

pub struct ColIterator<'a> {
    table: &'a Table,
    direction: TableOrientation,
    row_name: &'a str,
    row_index: usize,
    col_index: Option<usize>,
}

impl<'a> IntoIterator for &'a RowIterator<'a> {
    type Item = TableCell<'a>;
    type IntoIter = ColIterator<'a>;

    // Required method
    fn into_iter(self) -> Self::IntoIter {
        ColIterator {
            table: self.table,
            direction: self.direction,
            row_index: self.index.unwrap_or(0),
            col_index: None,
            row_name: self.row_name(),
        }
    }
}

impl<'a> ColIterator<'a> {
    pub fn row_name(&self) -> &'a str {
        &self.row_name
    }

    pub fn col_name(&self) -> &'a str {
        let index = self.col_index.unwrap_or_default();
        match self.direction {
            TableOrientation::BenchmarksOnRow => &self.table.installed[index].cell_name,
            TableOrientation::InstallsOnRow => &self.table.benchmarks[index].name,
        }
    }
}

#[derive(Debug)]
pub struct TableCell<'a> {
    pub result: Option<&'a ResultInfo>,
    pub reference: Option<&'a ResultInfo>,
    pub row_name: &'a str,
    pub col_name: &'a str,
}

impl<'a> Iterator for RowIterator<'a> {
    type Item = ColIterator<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let len = match self.direction {
            TableOrientation::InstallsOnRow => self.table.installed.len(),
            TableOrientation::BenchmarksOnRow => self.table.benchmarks.len(),
        };
        let index = *self
            .index
            .insert(self.index.map(|i| i + 1).unwrap_or_default());
        if index >= len {
            return None;
        }

        Some(ColIterator {
            table: self.table,
            direction: self.direction,
            row_index: index,
            col_index: None,
            row_name: self.row_name(),
        })
    }
}

impl<'a> Iterator for ColIterator<'a> {
    type Item = TableCell<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let len = match self.direction {
            TableOrientation::InstallsOnRow => self.table.benchmarks.len(),
            TableOrientation::BenchmarksOnRow => self.table.installed.len(),
        };
        let col_index = *self
            .col_index
            .insert(self.col_index.map(|i| i + 1).unwrap_or_default());
        if col_index >= len {
            return None;
        }
        let row_index = self.row_index;

        let installed_node = match self.direction {
            TableOrientation::InstallsOnRow => &self.table.installed[row_index],
            TableOrientation::BenchmarksOnRow => &self.table.installed[col_index],
        };

        let benchmark_node = match self.direction {
            TableOrientation::InstallsOnRow => &self.table.benchmarks[col_index],
            TableOrientation::BenchmarksOnRow => &self.table.benchmarks[row_index],
        };

        // println!("find: date = {}", installed_node.date);
        // println!("find: name = {}", benchmark_node.name);
        // println!("find: results = {:#?}", self.table.results.iter().find(|res| res.name == benchmark_node.name));

        Some(TableCell {
            result: self.table.results.iter().find(|res| {
                ((res.opensim_name == installed_node.name) && (res.date == installed_node.date))
                    && (res.name == benchmark_node.name)
            }),
            reference: self
                .table
                .reference
                .as_ref()
                .and_then(|x| x.iter().find(|res| res.name == benchmark_node.name)),
            row_name: self.row_name(),
            col_name: self.col_name(),
        })
    }
}

impl<'a> TableCell<'a> {
    pub fn percentage(&self) -> Option<f64> {
        let reference = self.reference?;
        let result = self.result?;
        Some(
            (result.durations.get_mean()? - reference.durations.get_mean()?)
                / result.durations.get_mean()?
                * 100.,
        )
    }

    pub fn log_diff(&self) -> Option<f64> {
        let file_a =
            std::fs::File::open(self.result?.opensim_log.as_ref()?).expect("failed to open result file");
        let data_a = crate::parse_logs::Data::read_opensim_file(file_a)
            .expect("failed to parse result file");

        let file_b =
            std::fs::File::open(self.reference?.opensim_log.as_ref()?).expect("failed to open result file");
        let data_b = crate::parse_logs::Data::read_opensim_file(file_b)
            .expect("failed to parse result file");

        let diff = crate::parse_logs::Diff::new(&data_a, &data_b).expect("failed to compute diff");
        let sum: f64 = diff.channels.iter().filter_map(|x| x.diff).sum();

        Some(sum)
    }

    pub fn write_cell_str(&self, s: &mut String) -> Option<()> {
        s.push_str(&format!(
            " {:.3} ({:.3})",
            self.result?.durations.get_mean().unwrap_or(f64::NAN),
            self.result?.durations.get_stddev().unwrap_or(f64::NAN),
        ));

        s.push_str(&format!(" {:.1}%", self.percentage()?));

        s.push_str(&format!(" E{:.1}", self.log_diff()?));

        Some(())
    }
}
