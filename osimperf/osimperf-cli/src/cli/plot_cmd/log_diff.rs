use std::fs;

use super::*;

pub fn print_diffs(table: &Table) -> Result<()> {
    for row in table {
        println!("row = {:?}", row.row_name());
        for cell in row.into_iter() {
            println!("    cell = {:?}", cell);
            // println!("        col = {:?}", cell.col_name);
            let file_a: Option<&PathBuf> = cell.result.and_then(|r| r.opensim_log.as_ref());
            let file_b: Option<&PathBuf> = cell.reference.and_then(|r| r.opensim_log.as_ref());
            println!("        file_a = {:?}", file_a);
            println!("        file_b = {:?}", file_b);
            // println!("");
            and_print_cell_diff(file_a, file_b).transpose()?;
        }
    }

    Ok(())
}

fn and_print_cell_diff(a: Option<&PathBuf>, b: Option<&PathBuf>) -> Option<Result<()>> {
    Some(print_cell_diff(a?, b?))
}

fn print_cell_diff(a: &PathBuf, b: &PathBuf) -> Result<()> {
    println!("file_a = {:?}", a);
    println!("file_b = {:?}", b);
    let file_a = fs::File::open(a)?;
    let data_a = crate::parse_logs::Data::read_opensim_file(file_a)?;

    let file_b = fs::File::open(b)?;
    let data_b = crate::parse_logs::Data::read_opensim_file(file_b)?;

    let diff = crate::parse_logs::Diff::new(&data_a, &data_b)?;
    let sum: f64 = diff.channels.iter().filter_map(|x| x.diff).sum();

    println!("sum = {}", sum);
    // println!("diff: {:#?}", diff);
    Ok(())
}
