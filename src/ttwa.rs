use ndarray::Array2;
use std::collections::HashMap;

const MIN_SIZE: f64 = 3500.0;
const TARGET_SIZE: f64 = 25000.0;
const MIN_CONTAINMENT: f64 = 0.667;
const TARGET_CONTAINMENT: f64 = 0.75;
const THRESHOLD: f64 = 0.5;

pub struct TravelToWorkAreas {
    pub adjacency_matrix: Array2<i32>,     // Adjacency matrix
    pub areas: HashMap<usize, Vec<usize>>, // Nodes in each area
    pub flow_to_node: Vec<i32>,
    pub flow_from_node: Vec<i32>,
    pub self_containment: HashMap<usize, f64>,
}

impl TravelToWorkAreas {
    pub fn new(adjacency_matrix: Vec<Vec<i32>>) -> TravelToWorkAreas {
        let array2 = Array2::from_shape_vec(
            (adjacency_matrix.len(), adjacency_matrix.len()),
            adjacency_matrix.into_iter().flatten().collect(),
        )
        .expect("Could not create adjacency matrix");

        let flow_to_node: Vec<i32> = (0..array2.shape()[0])
            .map(|node_index| array2.column(node_index).sum())
            .collect();

        let flow_from_node: Vec<i32> = (0..array2.shape()[0])
            .map(|node_index| array2.row(node_index).sum())
            .collect();

        TravelToWorkAreas {
            adjacency_matrix: array2,
            areas: HashMap::new(),
            flow_to_node,
            flow_from_node,
            self_containment: HashMap::new(),
        }
    }

    fn flow_to_area(&self, area_index: &usize) -> i32 {
        let area_nodes = &self.areas.get(area_index).unwrap();
        area_nodes.iter().map(|&node| self.flow_to_node[node]).sum()
    }
    
    fn flow_from_area(&self, area_index: &usize) -> i32 {
        let area_nodes = &self.areas.get(area_index).unwrap();
        area_nodes.iter().map(|&node| self.flow_from_node[node]).sum()
    }
    

    fn self_containment_of_area(&self, area_index: &usize) -> f64 {
        let area_nodes = &self.areas.get(area_index).unwrap();
    
        // Calculate the total flow within the area
        let self_containment: i32 = area_nodes.iter().map(|&i| {
            area_nodes.iter().map(|&j| {
                self.adjacency_matrix[(i, j)]
            }).sum::<i32>()
        }).sum();
    
        self_containment as f64 / self.flow_to_area(area_index) as f64
    }

    fn x_equation(&self, area_index: &usize) -> f64 {
        let size = self.flow_from_area(area_index) as f64;
        let self_containment = *self.self_containment.get(area_index).unwrap();

    
        if (size > MIN_SIZE) && (self_containment > TARGET_CONTAINMENT) {
            1.0
        } else if (size > TARGET_SIZE) && (self_containment > MIN_CONTAINMENT) {
            1.0
        } else {
            (size - MIN_SIZE)/(TARGET_SIZE - MIN_SIZE) + (self_containment - MIN_CONTAINMENT)/(TARGET_CONTAINMENT - MIN_CONTAINMENT)
        }
    }

    fn tij2_index(&self, node: usize, area_index: &usize) -> f64 {
        let area_nodes = &self.areas.get(area_index).unwrap();
    
        // Calculate flow from the node to the area
        let flow_node_to_area: i32 = area_nodes.iter()
            .map(|&area_node| self.adjacency_matrix[(node, area_node)])
            .sum();
    
        // Calculate flow from the area to the node
        let flow_area_to_node: i32 = area_nodes.iter()
            .map(|&area_node| self.adjacency_matrix[(area_node, node)])
            .sum();
    
        let a = flow_node_to_area as f64 / self.flow_from_node[node] as f64;
        let b = flow_node_to_area as f64 / self.flow_to_area(area_index) as f64;
        let c = flow_area_to_node as f64 / self.flow_from_area(area_index) as f64;
        let d = flow_area_to_node as f64 / self.flow_to_node[node] as f64;
    
        (a * b) + (c * d)
    }
    

    fn find_best_fit_area(&self, node: usize) -> usize {
        let mut best_fit_area = 0;
        let mut best_fit_value = 0.0;

        for area_index in self.areas.keys() {
            let tij2 = self.tij2_index(node, &area_index);

            if tij2 > best_fit_value {
                best_fit_area = *area_index;
                best_fit_value = tij2;
            }
        }

        best_fit_area
    }

    pub fn fit_travel_to_work_areas(&mut self) {
        // Create an area for each node
        for node in 0..self.adjacency_matrix.shape()[0] {
            self.areas.insert(node, vec![node]);
        }

        for area in self.areas.keys() {
            self.self_containment.insert(*area, self.self_containment_of_area(area));
        }
    
        let mut iter = 0;
    
        // While the worst scoring area is below the threshold
        loop {
            let mut worst_area_score = f64::INFINITY; // Initialize to a very large value
            let mut worst_area_index = 0;
    
            // Calculate the score for each area and find the worst score and its index
            for area_index in self.areas.keys() {
                let score = self.x_equation(area_index);
                if score < worst_area_score {
                    worst_area_score = score;
                    worst_area_index = *area_index;
                }
            }
    
            if iter % 100 == 0 {
                println!("Iteration {}: Worst area score: {}", iter, worst_area_score);
            }
    
            // If the worst area is above the threshold, we are done
            if worst_area_score > THRESHOLD {
                break;
            }
    
            // Remove the worst area, capturing its nodes
            let worst_area = self.areas.remove(&worst_area_index).unwrap();
            self.self_containment.remove(&worst_area_index);
    
            // For each node in the worst area, add it to the best fit area
            for node in worst_area {
                let best_fit_area = self.find_best_fit_area(node);
                self.areas.get_mut(&best_fit_area).unwrap().push(node);
                self.self_containment.insert(best_fit_area, self.self_containment_of_area(&best_fit_area));
            }
    
            iter += 1;
        }
    }    
}
