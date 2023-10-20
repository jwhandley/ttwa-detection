use anyhow::Result;
use std::path::Path;
mod graph;
mod io;
mod ttwa_naive;
use clap::Parser;

#[derive(Parser)]
struct Args {
    input: String,
    max_iter: Option<usize>,
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

    // Create graph from adjacency matrix
    let graph = graph::Graph::from_adjacency_matrix(adjacency_matrix);

    // Create a TTWA structure
    let mut ttwa = ttwa_naive::AreaCollection::new(graph);
    ttwa.fit(args.max_iter.unwrap_or(usize::MAX));

    // Print the results
    for area in ttwa.areas.iter().flatten() {
        println!("Area {} has {} nodes", area.id, area.node_ids.len());
    }

    Ok(())
}
