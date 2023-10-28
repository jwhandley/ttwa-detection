use anyhow::Result;
use std::path::Path;
mod graph;
mod io;
// mod ttwa_naive;
mod ttwa_v2;
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

    let ttwas = ttwa_v2::travel_to_work_areas(&graph);
    println!("Found {} TTWAs", ttwas.len());

    let mut nodes = Vec::new();
    let mut areas = Vec::new();

    // Print the results
    for (area_id, area) in ttwas.iter().enumerate() {
        for node in area.nodes.iter() {
            nodes.push(*node);
            areas.push(area_id);
        }
        println!(
            "Area {} has {} self containment, {} population, {} workforce",
            area_id, area.self_containment, area.flow_from_area, area.flow_to_area
        );
    }
    let area_metadata = ttwas
        .iter()
        .enumerate()
        .map(|(area_id, area)| {
            (
                area_id,
                [
                    area.self_containment,
                    area.flow_from_area,
                    area.flow_to_area,
                ],
            )
        })
        .collect::<HashMap<usize, [f64; 3]>>();
    // Write the results to a file
    if let Some(output) = args.output {
        io::write_nodes_to_areas(Path::new(&output), &codes, &nodes, &areas, &area_metadata)?;
    }

    Ok(())
}
