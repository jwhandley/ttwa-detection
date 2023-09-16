use anyhow::Result;
use std::path::Path;
mod io;
mod ttwa;

fn main() -> Result<()> {
    // Accept path as command line argument
    let path = std::env::args().nth(1).expect("No path provided");
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

    // Print the results
    println!("Areas:");
    for area in ttwa.areas {
        println!("{:?}", area.1.iter().map(|&i| codes[i].clone()).collect::<Vec<_>>());
    }

    Ok(())
}
