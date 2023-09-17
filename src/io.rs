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

pub fn write_nodes_to_areas(
    path: &Path,
    codes: &Vec<String>,
    nodes_to_areas: &std::collections::HashMap<usize, usize>,
) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record(&["code", "area"])?;
    for (node, area) in nodes_to_areas {
        writer.write_record(&[codes[*node].as_str(), area.to_string().as_str()])?;
    }
    Ok(())
}