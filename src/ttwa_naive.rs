use crate::graph::{Graph, Node};
const THRESHOLD: f64 = 0.0;
const MIN_SIZE: i32 = 3500;
const TARGET_SIZE: i32 = 25000;
const MIN_CONTAINMENT: f64 = 0.667;
const TARGET_CONTAINMENT: f64 = 0.75;

#[derive(PartialEq, Clone, Debug)]
pub struct Area {
    id: usize,
    nodes: Vec<Node>,
    flow_to_area: i32,
    flow_from_area: i32,
    self_containment: i32,
}

#[allow(dead_code)]
impl Area {
    fn new(id: usize) -> Area {
        Area {
            id,
            nodes: Vec::new(),
            flow_to_area: 0,
            flow_from_area: 0,
            self_containment: 0,
        }
    }

    fn add_node(&mut self, node: &Node) {
        self.flow_to_area += node.in_degree;
        self.flow_from_area += node.out_degree;
        self.self_containment += node
            .out_edges
            .iter()
            .filter_map(|e| {
                if self.nodes.iter().any(|n| n.id == e.target) {
                    Some(e.weight)
                } else {
                    None
                }
            })
            .sum::<i32>();
        // TODO: avoid cloning here
        self.nodes.push(node.clone());
    }

    fn remove_node(&mut self, node: &Node) {
        // Maybe make this return Option<Node>?
        if self.nodes.contains(node) {
            self.nodes.retain(|n| n != node);
            self.flow_to_area -= node.in_degree;
            self.flow_from_area -= node.out_degree;
            self.self_containment -= node
                .out_edges
                .iter()
                .filter_map(|e| {
                    if self.nodes.iter().any(|n| n.id == e.target) {
                        Some(e.weight)
                    } else {
                        None
                    }
                })
                .sum::<i32>();
        };
    }
}



#[derive(Debug)]
pub struct AreaCollection {
    pub areas: Vec<Area>,
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
        self.areas.push(area);
    }

    fn remove_area(&mut self, area: &Area) -> Vec<Node> {
        self.areas.retain(|a| a != area);
        area.nodes.clone()
    }

    fn flow_from_node_to_area(&self, node: &Node, area: &Area) -> i32 {
        node.out_edges
            .iter()
            .filter(|&edge| area.nodes.iter().any(|n| n.id == edge.target))
            .map(|edge| edge.weight)
            .sum()
    }

    fn flow_from_area_to_node(&self, area: &Area, node: &Node) -> i32 {
        node.in_edges
            .iter()
            .filter(|&edge| area.nodes.iter().any(|n| n.id == edge.source))
            .map(|edge| edge.weight)
            .sum()
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

    fn tij2(&self, node: &Node, area: &Area) -> f64 {
        let node_to_area = self.flow_from_node_to_area(node, area) as f64;
        let area_to_node = self.flow_from_area_to_node(area, node) as f64;

        let a = node_to_area / node.out_degree as f64;
        let b = node_to_area / area.flow_to_area as f64;
        let c = area_to_node / area.flow_from_area as f64;
        let d = area_to_node / node.in_degree as f64;

        (a * b) + (c * d)
    }

    pub fn fit(&mut self, max_iter: usize) {
        // Add all nodes to their own area
        let nodes_clone = self.graph.nodes.clone();
        for node in nodes_clone.values() {
            let mut area = Area::new(self.areas.len());
            area.add_node(node);
            self.add_area(area);
        }

        let mut iter = 0;

        loop {
            // Find worst x_equation
            let mut worst_area = None;
            let mut worst_score = f64::MAX;

            for area in self.areas.iter() {
                let score = self.x_equation(area);
                if score < worst_score {
                    worst_area = Some(area);
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

            let worst_area = worst_area.unwrap().clone();

            // Remove worst area, capturing its nodes
            let area_nodes = self.remove_area(&worst_area);

            // Find the best tij2 for each node

            for node in area_nodes.iter() {
                let mut best_area_index = None;
                let mut best_score = f64::MIN;
                for i in 0..self.areas.len() {
                    let score = self.tij2(node, &self.areas[i]);
                    if score > best_score {
                        best_area_index = Some(i);
                        best_score = score;
                    }
                }
                self.areas[best_area_index.unwrap()].add_node(node);
            }

            iter += 1;

            if iter >= max_iter {
                break;
            }
        }
    }
}
