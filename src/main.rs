use anyhow::Result;
use std::path::Path;
mod io;
mod ttwa;
use clap::Parser;

#[derive(Parser)]
struct Args {
    input: String,
    output: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.input;

    println!("Reading data from {}", path);
    let (codes, adjacency_matrix) = io::read_adjacency_matrix(Path::new(&path))?;

    println!(
        "Read {} nodes and {}x{} adjacency matrix",
        codes.len(),
        adjacency_matrix.len(),
        adjacency_matrix[0].len()
    );

    // Create a new TTWA instance
    let mut ttwa = ttwa::TravelToWorkAreas::new(adjacency_matrix);
    // Fit the travel to work areas
    ttwa.fit_travel_to_work_areas();

    // Invert areas to get a map of nodes to areas
    let mut nodes_to_areas: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (area_index, area_nodes) in ttwa.areas.iter() {
        for &node in area_nodes {
            nodes_to_areas.insert(node, *area_index);
        }
    }

    // Write result to csv
    io::write_nodes_to_areas(Path::new(&args.output), &codes, &nodes_to_areas)?;

    Ok(())
}
