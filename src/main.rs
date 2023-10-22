use anyhow::Result;
use std::path::Path;
mod graph;
mod io;
mod ttwa_naive;
use clap::Parser;
use std::collections::HashMap;

use crate::io::read_adjacency_matrix_to_graph;

#[derive(Parser)]
struct Args {
    input: String,
    output: Option<String>,
    max_iter: Option<usize>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.input;

    let (codes, graph) = read_adjacency_matrix_to_graph(Path::new(&path))?;

    // Create a TTWA structure
    let mut ttwa = ttwa_naive::AreaCollection::new(graph);
    ttwa.fit(args.max_iter.unwrap_or(usize::MAX));

    let mut node_to_area: HashMap<usize, usize> = HashMap::new();

    // Print the results
    for area in ttwa.areas.iter().flatten() {
        for node in area.node_ids.iter() {
            node_to_area.insert(*node, area.id);
        }
        // println!(
        //     "Area {} has {} self containment, {} population, {} workforce",
        //     codes[area.id], area.self_containment, area.flow_from_area, area.flow_to_area
        // );
    }

    // Write the results to a file
    if let Some(output) = args.output {
        io::write_nodes_to_areas(Path::new(&output), &codes, &node_to_area)?;
    }

    Ok(())
}
