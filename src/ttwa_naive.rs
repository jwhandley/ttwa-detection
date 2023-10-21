use crate::graph::{Graph, Node};
// const THRESHOLD: f64 = 1.0;
// const MIN_SIZE: i32 = 3500;
// const TARGET_SIZE: i32 = 25000;
// const MIN_CONTAINMENT: f64 = 0.667;
// const TARGET_CONTAINMENT: f64 = 0.75;

#[derive(PartialEq, Clone, Debug)]
pub struct Area {
    pub id: usize,
    pub node_ids: Vec<usize>,
    pub flow_to_area: i32,
    pub flow_from_area: i32,
    pub self_containment: i32,
    pub x_score: f64,
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
            x_score: 0.0,
        }
    }

    fn add_node(&mut self, node_id: usize, graph: &mut Graph) {
        graph.nodes[node_id].area_id = self.id;

        let node = &graph.nodes[node_id];
        self.flow_to_area += node
            .edges
            .iter()
            .filter(|&edge| graph.nodes[edge.target].area_id == self.id)
            .map(|edge| edge.weight)
            .sum::<i32>();
        self.flow_from_area += node
            .edges
            .iter()
            .filter(|&edge| graph.nodes[edge.source].area_id == self.id)
            .map(|edge| edge.weight)
            .sum::<i32>();

        self.self_containment += node
            .edges
            .iter()
            .filter(|&edge| {
                graph.nodes[edge.source].area_id == self.id
                    && graph.nodes[edge.target].area_id == self.id
            })
            .map(|edge| edge.weight)
            .sum::<i32>();
        self.node_ids.push(node.id);

        self.x_score = self.x_equation();
    }

    fn remove_node(&mut self, node_id: usize, graph: &mut Graph) {
        let node = &graph.nodes[node_id];
        if self.node_ids.contains(&node_id) {
            self.node_ids.retain(|&n| n != node_id);

            self.flow_to_area -= node
                .edges
                .iter()
                .filter(|&edge| graph.nodes[edge.target].area_id == self.id)
                .map(|edge| edge.weight)
                .sum::<i32>();
            self.flow_from_area -= node
                .edges
                .iter()
                .filter(|&edge| graph.nodes[edge.source].area_id == self.id)
                .map(|edge| edge.weight)
                .sum::<i32>();

            self.self_containment -= node
                .edges
                .iter()
                .filter(|&edge| {
                    graph.nodes[edge.source].area_id == self.id
                        && graph.nodes[edge.target].area_id == self.id
                })
                .map(|edge| edge.weight)
                .sum::<i32>();

            graph.nodes[node_id].area_id = usize::MAX;
            self.x_score = self.x_equation();
        };
    }

    fn x_equation(&self) -> f64 {
        0.25*(0.667*self.flow_to_area as f64).ln() + 0.75*(self.self_containment as f64).ln() - 10.0
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

        for edge in &node.edges {
            if self.graph.nodes[edge.target].area_id == area.id {
                node_to_area += edge.weight;
            } else if self.graph.nodes[edge.source].area_id == area.id {
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

    pub fn fit(&mut self, max_iter: usize) {
        // Add all nodes to their own area
        for node_id in 0..self.graph.nodes.len() {
            let mut area = Area::new(node_id);
            area.add_node(node_id, &mut self.graph);
            self.add_area(area);
        }

        let mut iter = 0;

        loop {
            // Find the worst area
            let mut worst_area: Option<Area> = None;
            let mut worst_score = f64::MAX;

            for area in self.areas.iter().flatten() {
                if area.x_score < worst_score {
                    worst_area = Some(area.clone());
                    worst_score = area.x_score;
                }
            }

            // If x_equation for worst area is above threshold, stop
            if self.areas.iter().flatten().count() <= 170 {
                println!("Iteration: {}, worst score {}", iter, worst_score);
                dbg!(self.areas.iter().flatten().count());
                break;
            }
            if iter % 1000 == 0 {
                println!("Iteration: {}, worst score {}", iter, worst_score);
                dbg!(self.areas.iter().flatten().count());
            }

            let worst_area = worst_area.unwrap();

            // Remove worst area, capturing its nodes
            let area_nodes = worst_area.node_ids.clone();
            self.remove_area(&worst_area);

            // Find the best tij2 for each node

            for node_idx in area_nodes.iter() {
                let mut best_area_index = None;
                let mut best_score = f64::MIN;

                // Find relevant areas, i.e. areas whose nodes are connected to this node
                let mut relevant_areas = Vec::new();
                let node = &self.graph.nodes[*node_idx];

                for edge in node.edges.iter() {
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
                }
            }

            iter += 1;

            if iter >= max_iter {
                break;
            }
        }
    }
}
