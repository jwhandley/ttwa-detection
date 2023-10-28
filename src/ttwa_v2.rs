// use std::collections::HashSet;
use rustc_hash::FxHashSet;

use crate::graph::{EdgeDirection, Graph};
const TARGET_SIZE: f64 = 25000.0;
const MIN_SIZE: f64 = 3500.0;
const TARGET_CONTAINMENT: f64 = 0.75;
const MIN_CONTAINMENT: f64 = 0.667;
const TRADEOFF: f64 = (MIN_CONTAINMENT - TARGET_CONTAINMENT) / (TARGET_SIZE - MIN_SIZE);
const INTERCEPT: f64 = TARGET_CONTAINMENT - TRADEOFF * MIN_SIZE;
const THRESHOLD: f64 = 0.0;

type NodeIndex = usize;
pub struct Area {
    pub nodes: FxHashSet<NodeIndex>,
    pub flow_to_area: f64,
    pub flow_from_area: f64,
    pub self_containment: f64,
}
type TravelToWorkAreas = Vec<Area>;

pub fn travel_to_work_areas(graph: &Graph) -> TravelToWorkAreas {
    // Assign each node to an area
    let mut areas = TravelToWorkAreas::new();
    for node in graph.nodes.iter() {
        let mut area = Area {
            nodes: FxHashSet::default(),
            flow_to_area: 0.0,
            flow_from_area: 0.0,
            self_containment: 0.0,
        };
        area.nodes.insert(node.id);
        area.flow_to_area += node.in_degree as f64;
        area.flow_from_area += node.out_degree as f64;
        area.self_containment += graph
            .get_edges(node.id, EdgeDirection::In)
            .filter(|&e| e.source == node.id)
            .map(|e| e.weight)
            .sum::<u32>() as f64;

        areas.push(area);
    }
    let mut iter = 0;

    loop {
        // Find worst x_equation
        let mut worst_area = None;
        let mut worst_x_equation = f64::INFINITY;

        for (area_index, area) in areas.iter().enumerate() {
            let x_equation = x_equation(area);
            if x_equation < worst_x_equation {
                worst_x_equation = x_equation;
                worst_area = Some(area_index);
            }
        }

        if iter % 1000 == 0 {
            println!(
                "Iteration {}: {}, {} areas remaining",
                iter,
                worst_x_equation,
                areas.len()
            );
        }

        if worst_x_equation > THRESHOLD {
            break;
        }

        let worst_area = worst_area.unwrap();
        let worst_area_nodes = areas[worst_area].nodes.clone();
        areas.remove(worst_area);

        for &node in worst_area_nodes.iter() {
            let mut best_area = None;
            let mut best_tij2 = 0.0;

            for (area_index, area) in areas.iter().enumerate() {
                let tij2 = tij2(graph, node, area);
                if tij2 > best_tij2 {
                    best_tij2 = tij2;
                    best_area = Some(area_index);
                }
            }

            if let Some(area_index) = best_area {
                areas[area_index].nodes.insert(node);
                areas[area_index].flow_to_area += graph.nodes[node].in_degree as f64;
                areas[area_index].flow_from_area += graph.nodes[node].out_degree as f64;

                let a = graph
                    .get_edges(node, EdgeDirection::Out)
                    .filter(|&e| areas[area_index].nodes.contains(&e.target))
                    .map(|edge| edge.weight)
                    .sum::<u32>() as f64;

                let b = graph
                    .get_edges(node, EdgeDirection::In)
                    .filter(|&e| {
                        areas[area_index].nodes.contains(&e.source) && e.source != e.target
                    })
                    .map(|edge| edge.weight)
                    .sum::<u32>() as f64;

                areas[area_index].self_containment += a + b;
            }
        }

        iter += 1;
    }

    areas
}

fn x_equation(area: &Area) -> f64 {
    let size = area.flow_from_area;
    let containment = area.self_containment;

    let supply_self_containment = containment / area.flow_from_area;
    let demand_self_containment = containment / area.flow_to_area;
    // The methodology is unclear about how these to area combined to create a single index
    let self_containment = (supply_self_containment*demand_self_containment).sqrt();

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

fn tij2(graph: &Graph, node: NodeIndex, area: &Area) -> f64 {
    let area_to_node = flow_area_to_node(graph, node, area);
    let node_to_area = flow_node_to_area(graph, node, area);

    let to_area = area.flow_to_area;
    let from_area = area.flow_from_area;

    let a = node_to_area / graph.nodes[node].out_degree as f64;
    let b = node_to_area / to_area;
    let c = area_to_node / from_area;
    let d = area_to_node / graph.nodes[node].in_degree as f64;

    a * b + c * d
}

fn flow_area_to_node(graph: &Graph, node: NodeIndex, area: &Area) -> f64 {
    graph
        .get_edges(node, EdgeDirection::In)
        .filter(|e| area.nodes.contains(&e.source))
        .map(|e| e.weight)
        .sum::<u32>() as f64
}

fn flow_node_to_area(graph: &Graph, node: NodeIndex, area: &Area) -> f64 {
    graph
        .get_edges(node, EdgeDirection::Out)
        .filter(|e| area.nodes.contains(&e.target))
        .map(|e| e.weight)
        .sum::<u32>() as f64
}