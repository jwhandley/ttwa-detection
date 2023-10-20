use crate::graph::{Graph, Node};
use std::collections::HashSet;
const THRESHOLD: f64 = 0.0;
const MIN_SIZE: i32 = 3500;
const TARGET_SIZE: i32 = 25000;
const MIN_CONTAINMENT: f64 = 0.667;
const TARGET_CONTAINMENT: f64 = 0.75;

#[derive(PartialEq, Clone, Debug)]
pub struct Area {
    pub id: usize,
    pub node_ids: Vec<usize>,
    flow_to_area: i32,
    flow_from_area: i32,
    self_containment: i32,
}

#[allow(dead_code)]
impl Area {
    fn new(id: usize) -> Area {
        Area {
            id,
            node_ids: Vec::new(),
            flow_to_area: 0,
            flow_from_area: 0,
            self_containment: 0,
        }
    }

    fn add_node(&mut self, node_id: usize, graph: &mut Graph) {
        graph.nodes[node_id].area_id = Some(self.id);
        let node = &graph.nodes[node_id];
        self.flow_to_area += node.in_degree;
        self.flow_from_area += node.out_degree;
        self.self_containment += node
            .out_edges
            .iter()
            .filter_map(|e| {
                if self.node_ids.iter().any(|n| n == &e.target) {
                    Some(e.weight)
                } else {
                    None
                }
            })
            .sum::<i32>();
        self.node_ids.push(node.id);
    }

    fn remove_node(&mut self, node_id: usize, graph: &mut Graph) {
        let node = &graph.nodes[node_id];
        if self.node_ids.contains(&node_id) {
            self.node_ids.retain(|&n| n != node_id);
            self.flow_to_area -= node.in_degree;
            self.flow_from_area -= node.out_degree;
            self.self_containment -= node
                .out_edges
                .iter()
                .filter_map(|e| {
                    if self.node_ids.iter().any(|n| n == &e.target) {
                        Some(e.weight)
                    } else {
                        None
                    }
                })
                .sum::<i32>();

            graph.nodes[node_id].area_id = None;
        };
    }
}

#[derive(Debug)]
pub struct AreaCollection {
    pub areas: Vec<Option<Area>>,
    graph: Graph,
}

impl AreaCollection {
    pub fn new(graph: Graph) -> AreaCollection {
        AreaCollection {
            areas: Vec::new(),
            graph,
        }
    }

    fn add_area(&mut self, area: Area) {
        self.areas.push(Some(area));
    }

    fn remove_area(&mut self, area: &Area) -> Vec<usize> {
        
        let nodes = area.node_ids
            .iter()
            .map(|node_id| {
                self.graph.nodes[*node_id].area_id = None;
                *node_id
            })
            .collect();

        self.areas[area.id] = None;
        nodes

    }

    fn combined_flow(&self, node: &Node, area: &Area) -> (i32, i32) {
        let (mut node_to_area, mut area_to_node) = (0, 0);
    
        for edge in node.out_edges.iter().chain(&node.in_edges) {
            if self.graph.nodes[edge.target].area_id == Some(area.id) {
                node_to_area += edge.weight;
            } else if self.graph.nodes[edge.source].area_id == Some(area.id) {
                area_to_node += edge.weight;
            }
        }
    
        (node_to_area, area_to_node)
    }
    
    

    fn tij2(&mut self, node_id: usize, area_id: usize) -> f64 {
        let area = self.areas[area_id].clone().expect("Area not found");
        let node = &self.graph.nodes[node_id];
    
        let (node_to_area, area_to_node) = self.combined_flow(node, &area);

        let a = node_to_area as f64 / self.graph.nodes[node_id].out_degree as f64;
        let b = node_to_area as f64 / area.flow_to_area as f64;
        let c = area_to_node as f64 / area.flow_from_area as f64;
        let d = area_to_node as f64 / self.graph.nodes[node_id].in_degree as f64;

        (a * b) + (c * d)
    }

    fn x_equation(&self, area: &Area) -> f64 {
        let self_containment = area.self_containment as f64 / area.flow_from_area as f64;
        let size = area.flow_from_area;

        if ((size > MIN_SIZE) && (self_containment > TARGET_CONTAINMENT))
            || ((size > TARGET_SIZE) && (self_containment > MIN_CONTAINMENT))
        {
            1.0
        } else {
            (size - MIN_SIZE) as f64 / (TARGET_SIZE - MIN_SIZE) as f64
                + (self_containment - MIN_CONTAINMENT) / (TARGET_CONTAINMENT - MIN_CONTAINMENT)
        }
    }

    

    pub fn fit(&mut self, max_iter: usize) {
        // Add all nodes to their own area
        for node_id in 0..self.graph.nodes.len() {
            let mut area = Area::new(node_id);
            area.add_node(node_id, &mut self.graph);
            self.add_area(area);
        }

        let mut iter = 0;

        loop {
            // Find worst x_equation
            let mut worst_area: Option<Area> = None;
            let mut worst_score = f64::MAX;

            for area in self.areas.iter().flatten() {
                let score = self.x_equation(&area);
                if score < worst_score {
                    worst_area = Some(area.clone());
                    worst_score = score;
                }
            }

            // If x_equation for worst area is above threshold, stop
            if worst_score > THRESHOLD {
                break;
            }
            if iter % 100 == 0 {
                println!("Iteration: {}, worst score {}", iter, worst_score);
            }

            let worst_area = worst_area.unwrap();

            // Remove worst area, capturing its nodes
            let area_nodes = self.remove_area(&worst_area);

            // Find the best tij2 for each node

            for node_idx in area_nodes.iter() {
                

                let mut best_area_index = None;
                let mut best_score = f64::MIN;

                // Find relevant areas, i.e. areas whose nodes are connected to this node
                let mut relevant_areas = HashSet::new();

                for edge in self.graph.nodes[*node_idx].out_edges.iter() {
                    if let Some(area_id) = self.graph.nodes[edge.target].area_id {
                        if area_id != worst_area.id {
                            relevant_areas.insert(area_id);
                
                        }
                    }
                }

                for edge in self.graph.nodes[*node_idx].in_edges.iter() {
                    if let Some(area_id) = self.graph.nodes[edge.source].area_id {
                        if area_id != worst_area.id {
                            relevant_areas.insert(area_id);
                
                        }
                    }
                }


                // Now, compute the tij2 score only for the relevant areas
                for area_idx in relevant_areas.iter() {
                    let score = self.tij2(*node_idx, *area_idx);
                    if score > best_score {
                        best_area_index = Some(area_idx);
                        best_score = score;
                    }
                }

                if let Some(best_idx) = best_area_index {
                    self.areas[*best_idx]
                        .as_mut()
                        .unwrap()
                        .add_node(*node_idx, &mut self.graph);
                    
                }
            }

            iter += 1;

            if iter >= max_iter {
                break;
            }
        }
    }
}
