use anyhow::Result;
use std::path::Path;

pub fn read_adjacency_matrix(path: &Path) -> Result<(Vec<String>, Vec<Vec<i32>>)> {
    let mut codes = Vec::new();
    let mut adjacency_matrix = Vec::new();
    let mut reader = csv::Reader::from_path(path)?;
    for result in reader.records() {
        let record = result?;
        codes.push(record.get(0).unwrap().to_string());
        let mut row = Vec::new();
        for i in 1..record.len() {
            row.push(record.get(i).unwrap().parse::<i32>()?);
        }
        adjacency_matrix.push(row);
    }
    Ok((codes, adjacency_matrix))
}

