use crate::graph::{Graph, Node};
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
        graph.nodes[node_id].area_id = self.id;
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

            graph.nodes[node_id].area_id = usize::MAX;
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

    fn remove_area(&mut self, area: &Area) {
        area.node_ids.iter().for_each(|node_id| {
            self.graph.nodes[*node_id].area_id = usize::MAX;
        });

        self.areas[area.id] = None;
    }

    fn combined_flow(&self, node: &Node, area: &Area) -> (i32, i32) {
        let mut node_to_area = 0;
        let mut area_to_node = 0;

        for edge in &node.out_edges {
            if self.graph.nodes[edge.target].area_id == area.id {
                node_to_area += edge.weight;
            }
        }

        for edge in &node.in_edges {
            if self.graph.nodes[edge.source].area_id == area.id {
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
            if iter % 1000 == 0 {
                println!("Iteration: {}, worst score {}", iter, worst_score);
            }

            let worst_area = worst_area.unwrap();

            // Remove worst area, capturing its nodes
            let area_nodes = worst_area.node_ids.clone();
            for area_node in area_nodes.iter() {
                if let Some(area) = &mut self.areas[worst_area.id] {
                    area.remove_node(*area_node, &mut self.graph);
                }
            }

            // Find the best tij2 for each node

            for node_idx in area_nodes.iter() {
                let mut best_area_index = None;
                let mut best_score = f64::MIN;

                // Find relevant areas, i.e. areas whose nodes are connected to this node
                let mut relevant_areas = Vec::new();
                let node = &self.graph.nodes[*node_idx];

                
                for edge in node.out_edges.iter().chain(node.in_edges.iter()) {
                    if self.graph.nodes[edge.target].area_id != usize::MAX {
                        relevant_areas.push(self.graph.nodes[edge.target].area_id);
                    }
                    
                    if self.graph.nodes[edge.source].area_id != usize::MAX {
                        relevant_areas.push(self.graph.nodes[edge.source].area_id);
                    }
                    
                }

                relevant_areas.sort_unstable();
                relevant_areas.dedup();
                

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

                    if *best_idx != worst_area.id {
                        self.remove_area(&worst_area);
                    }
                    
                }
            }

            
            
            

            iter += 1;

            if iter >= max_iter {
                break;
            }
        }
    }
}
