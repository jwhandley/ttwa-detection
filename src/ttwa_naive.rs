use std::collections::HashSet;

use crate::graph::{EdgeDirection, Graph};
const TARGET_SIZE: f64 = 25000.0;
const MIN_SIZE: f64 = 3500.0;
const TARGET_CONTAINMENT: f64 = 0.75;
const MIN_CONTAINMENT: f64 = 0.667;
const TRADEOFF: f64 = (MIN_CONTAINMENT - TARGET_CONTAINMENT) / (TARGET_SIZE - MIN_SIZE);
const INTERCEPT: f64 = TARGET_CONTAINMENT - TRADEOFF * MIN_SIZE;
const THRESHOLD: f64 = -1e-2;

#[derive(PartialEq, Clone, Debug)]
pub struct Area {
    pub id: usize,
    pub node_ids: HashSet<usize>,
    pub flow_to_area: u32,
    pub flow_from_area: u32,
    pub self_containment: u32,
}

#[allow(dead_code)]
impl Area {
    fn new(id: usize) -> Area {
        Area {
            id,
            node_ids: HashSet::new(),
            flow_to_area: 0,
            flow_from_area: 0,
            self_containment: 0,
        }
    }

    fn add_node(&mut self, node_id: usize, graph: &Graph) {
        if !self.node_ids.insert(node_id) {
            return;
        }

        self.flow_to_area += graph.nodes[node_id].in_degree;

        self.flow_from_area += graph.nodes[node_id].out_degree;

        // All out edges where the target is in the area (including self-loops)
        let a = graph
            .get_edges(node_id, EdgeDirection::Out)
            .filter(|&e| self.node_ids.contains(&e.target))
            .map(|edge| edge.weight)
            .sum::<u32>();

        // All in edges where the source is in the area (excluding self-loops)
        let b = graph
            .get_edges(node_id, EdgeDirection::In)
            .filter(|&e| self.node_ids.contains(&e.source) && e.source != e.target)
            .map(|edge| edge.weight)
            .sum::<u32>();

        self.self_containment += a + b;

        // dbg!(
        //     self.flow_to_area,
        //     self.flow_from_area,
        //     self.self_containment
        // );
    }

    fn remove_node(&mut self, node_id: usize, graph: &Graph) {
        if !self.node_ids.remove(&node_id) {
            return;
        };

        self.flow_to_area -= graph.nodes[node_id].in_degree;
        self.flow_from_area -= graph.nodes[node_id].out_degree;

        // All out edges where the target is in the area (including self-loops)
        let a = graph
            .get_edges(node_id, EdgeDirection::Out)
            .filter(|&e| self.node_ids.contains(&e.target))
            .map(|edge| edge.weight)
            .sum::<u32>();

        // All in edges where the source is in the area (excluding self-loops)
        let b = graph
            .get_edges(node_id, EdgeDirection::In)
            .filter(|&e| self.node_ids.contains(&e.source) && e.source != e.target)
            .map(|edge| edge.weight)
            .sum::<u32>();

        self.self_containment -= a + b;
    }

    fn x_equation(&self) -> f64 {
        let size = self.flow_from_area as f64;
        assert!(self.self_containment <= self.flow_to_area);
        assert!(self.self_containment <= self.flow_from_area);

        // dbg!(self.flow_to_area, self.flow_from_area, self.self_containment);

        let demand_self_containment = self.self_containment as f64 / self.flow_to_area as f64;
        let supply_self_containment = self.self_containment as f64 / self.flow_from_area as f64;
        let self_containment = demand_self_containment.max(supply_self_containment);

        if size >= TARGET_SIZE && self_containment >= TARGET_CONTAINMENT {
            1.0 / 12.0
        } else if self_containment >= TARGET_CONTAINMENT {
            TRADEOFF * (MIN_SIZE - size)
        } else if size >= TARGET_SIZE {
            self_containment - MIN_CONTAINMENT
        } else {
            self_containment - TRADEOFF * size - INTERCEPT
        }
    }
}
#[derive(Debug)]
pub struct AreaCollection {
    pub areas: Vec<Option<Area>>,
    pub node_to_area: Vec<usize>,
    graph: Graph,
}

impl AreaCollection {
    pub fn new(graph: Graph) -> AreaCollection {
        AreaCollection {
            areas: Vec::new(),
            node_to_area: vec![usize::MAX; graph.nodes.len()],
            graph,
        }
    }

    fn add_area(&mut self, area: Area) {
        self.areas.push(Some(area));
    }

    fn remove_area(&mut self, area_id: usize) -> HashSet<usize> {
        let area_nodes = self.areas[area_id].clone().unwrap().node_ids.clone();

        for node_id in area_nodes.iter() {
            self.remove_node_from_area(*node_id, area_id);
        }
        self.areas[area_id] = None;

        area_nodes
    }

    fn add_node_to_area(&mut self, node_id: usize, area_id: usize) {
        let area = self.areas[area_id].as_mut().unwrap();
        area.add_node(node_id, &self.graph);
        self.node_to_area[node_id] = area_id;
    }

    fn remove_node_from_area(&mut self, node_id: usize, area_id: usize) {
        let area = self.areas[area_id].as_mut().unwrap();
        area.remove_node(node_id, &self.graph);
        self.node_to_area[node_id] = usize::MAX;
    }

    fn flow_from_node_to_area(&self, node_id: usize, area_id: usize) -> u32 {
        self.graph
            .get_edges(node_id, EdgeDirection::Out)
            .filter(|&e| self.node_to_area[e.target] == area_id)
            .map(|edge| edge.weight)
            .sum::<u32>()
    }

    fn flow_from_area_to_node(&mut self, node_id: usize, area_id: usize) -> u32 {
        self.graph
            .get_edges(node_id, EdgeDirection::In)
            .filter(|&e| self.node_to_area[e.source] == area_id)
            .map(|edge| edge.weight)
            .sum::<u32>()
    }

    fn tij2(&mut self, node_id: usize, area_id: usize) -> f64 {
        let node_to_area = self.flow_from_node_to_area(node_id, area_id);
        let area_to_node = self.flow_from_area_to_node(node_id, area_id);
        let area = self.areas[area_id].as_ref().unwrap();

        let a = node_to_area as f64 / self.graph.nodes[node_id].out_degree as f64;
        let b = node_to_area as f64 / area.flow_to_area as f64;
        let c = area_to_node as f64 / area.flow_from_area as f64;
        let d = area_to_node as f64 / self.graph.nodes[node_id].in_degree as f64;

        (a * b) + (c * d)
    }

    pub fn fit(&mut self, max_iter: usize) {
        // Add all nodes to their own area
        for node_id in 0..self.graph.nodes.len() {
            let area = Area::new(node_id);
            self.add_area(area);
            self.add_node_to_area(node_id, node_id);
        }

        let mut iter = 0;

        loop {
            // Find the worst area
            let mut worst_area: Option<usize> = None;
            let mut worst_score = f64::MAX;

            for area in self.areas.iter().flatten() {
                if area.x_equation() < worst_score {
                    worst_area = Some(area.id);
                    worst_score = area.x_equation();
                }
            }

            // If x_equation for worst area is above threshold, stop
            if worst_score >= THRESHOLD {
                println!(
                    "Iteration: {}, worst score {:.03}, {} areas remaining",
                    iter,
                    worst_score,
                    self.areas.iter().flatten().count()
                );
                break;
            }
            if iter % 1000 == 0 {
                println!(
                    "Iteration: {}, worst score {:.03}, {} areas remaining",
                    iter,
                    worst_score,
                    self.areas.iter().flatten().count()
                );
            }

            let worst_area_index = worst_area.unwrap();
            // Remove the area, capturing its nodes
            let area_nodes = self.remove_area(worst_area_index);

            // Find the best tij2 for each node
            // let relevant_areas: Vec<usize> = self.areas.iter().flatten().map(|a| a.id).collect();
            for node_idx in area_nodes.iter() {
                let mut best_area_index = None;
                let mut best_tij2 = f64::MIN;

                // Find relevant areas, i.e. areas whose nodes are connected to this node
                let mut relevant_areas: HashSet<usize> = HashSet::new();

                // Loop over in edges
                for edge in self.graph.get_edges(*node_idx, EdgeDirection::In) {
                    let source_area = self.node_to_area[edge.source];
                    if source_area != worst_area_index && source_area != usize::MAX {
                        relevant_areas.insert(source_area);
                    }
                }

                // Loop over out edges
                for edge in self.graph.get_edges(*node_idx, EdgeDirection::Out) {
                    let target_area = self.node_to_area[edge.target];
                    if target_area != worst_area_index && target_area != usize::MAX {
                        relevant_areas.insert(target_area);
                    }
                }

                // Now, compute the tij2 score only for the relevant areas
                for area_idx in relevant_areas.iter() {
                    let score = self.tij2(*node_idx, *area_idx);
                    if score > best_tij2 {
                        best_area_index = Some(area_idx);
                        best_tij2 = score;
                    }
                }

                // dbg!(best_tij2, best_area_index, worst_area_index, node_idx);
                let best_area_idx = best_area_index.unwrap();

                // println!("Inserting node {} into area {} after removing area {}", node_idx, best_area_idx, worst_area.id);
                self.add_node_to_area(*node_idx, *best_area_idx);
            }

            iter += 1;

            if iter >= max_iter {
                break;
            }
        }
    }
}
