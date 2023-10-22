use crate::graph::Graph;
use anyhow::Result;
use std::path::Path;

pub fn read_adjacency_matrix_to_graph(path: &Path) -> Result<(Vec<String>, Graph)> {
    let mut adjacency_matrix = Vec::new();
    let mut reader = csv::Reader::from_path(path)?;
    let mut codes = Vec::new();
    for result in reader.records() {
        let record = result?;
        let code = record.get(0).unwrap().to_owned();
        codes.push(code);
        let row: Result<Vec<u32>, _> = (1..record.len())
            .map(|i| record.get(i).unwrap().parse::<u32>())
            .collect();
        adjacency_matrix.push(row?);
    }
    Ok((codes, Graph::from_adjacency_matrix(adjacency_matrix)))
}

#[allow(dead_code)]
pub fn write_nodes_to_areas(
    path: &Path,
    codes: &[String],
    nodes_to_areas: &std::collections::HashMap<usize, usize>,
) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record(["code", "area"])?;
    for (node, area) in nodes_to_areas {
        writer.write_record([codes[*node].as_str(), codes[*area].as_str()])?;
    }
    Ok(())
}
