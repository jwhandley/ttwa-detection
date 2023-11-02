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

#[derive(Clone)]
pub struct Area {
    pub nodes: FxHashSet<NodeIndex>,
    pub flow_to_area: f64,
    pub flow_from_area: f64,
    pub self_containment: f64,
}
type TravelToWorkAreas = Vec<Area>;

pub fn travel_to_work_areas(graph: &Graph) -> TravelToWorkAreas {
    // Assign each node to an area
    let mut node2area = Vec::new();
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
        node2area.push(areas.len() - 1);
    }
    let mut iter = 0;

    loop {
        assert_eq!(node2area.len(), graph.nodes.len());
        // Find worst x_equation
        let mut worst_area = None;
        let mut worst_x_equation = f64::INFINITY;

        for (area_index, area) in areas.iter().enumerate().filter(|(_,a)| a.nodes.len() > 0) {
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
                areas.iter().filter(|a| a.nodes.len() > 0).count()
            );
        }

        if worst_x_equation >= THRESHOLD {
            break;
        }

        let worst_area = worst_area.unwrap();
        let worst_area_nodes = areas[worst_area].nodes.clone();

        // Clear nodes from worst area
        for node in worst_area_nodes.iter() {
            node2area[*node] = usize::MAX;
            areas[worst_area].nodes.remove(node);
            areas[worst_area].flow_from_area -= graph.nodes[*node].out_degree as f64;
            areas[worst_area].flow_to_area -= graph.nodes[*node].in_degree as f64;
            let a = graph
                .get_edges(*node, EdgeDirection::Out)
                .filter(|&e| areas[worst_area].nodes.contains(&e.target))
                .map(|edge| edge.weight)
                .sum::<u32>() as f64;

            let b = graph
                .get_edges(*node, EdgeDirection::In)
                .filter(|&e| areas[worst_area].nodes.contains(&e.source) && e.source != e.target)
                .map(|edge| edge.weight)
                .sum::<u32>() as f64;

            areas[worst_area].self_containment -= a + b;
        }

        for &node in worst_area_nodes.iter() {
            let relevant_areas = graph
                .get_neighbors(node)
                .filter_map(|neighbor| if node2area[neighbor] != usize::MAX {Some(node2area[neighbor])} else {None}
                )
                .map(|area| area)
                .filter(|&area| area != worst_area);

            let mut best_area = None;
            let mut best_tij2 = 0.0;

            for area_index in relevant_areas {
                assert_ne!(area_index, worst_area);
                assert_ne!(area_index, usize::MAX);

                let tij2 = tij2(graph, node, &areas, area_index, &node2area);

                

                if tij2 > best_tij2 {
                    best_tij2 = tij2;
                    best_area = Some(area_index);
                }
            }
            let best_area = best_area.expect("tij2 should have been > 0");
            node2area[node] = best_area;

            areas[best_area].nodes.insert(node);
            areas[best_area].flow_to_area += graph.nodes[node].in_degree as f64;
            areas[best_area].flow_from_area += graph.nodes[node].out_degree as f64;

            let a = graph
                .get_edges(node, EdgeDirection::Out)
                .filter(|&e| areas[best_area].nodes.contains(&e.target))
                .map(|edge| edge.weight)
                .sum::<u32>() as f64;

            let b = graph
                .get_edges(node, EdgeDirection::In)
                .filter(|&e| areas[best_area].nodes.contains(&e.source) && e.source != e.target)
                .map(|edge| edge.weight)
                .sum::<u32>() as f64;

            areas[best_area].self_containment += a + b;
        }

        iter += 1;
    }

    // Remove empty areas before returning
    areas.iter().filter(|&a| a.nodes.len()>0).cloned().collect::<TravelToWorkAreas>()
}

fn x_equation(area: &Area) -> f64 {
    let size = area.flow_from_area;
    let containment = area.self_containment;

    let supply_self_containment = containment / area.flow_from_area;
    let demand_self_containment = containment / area.flow_to_area;
    // The methodology is unclear about how these to area combined to create a single index
    let self_containment = supply_self_containment.min(demand_self_containment);

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

fn tij2(graph: &Graph, node: NodeIndex, areas: &TravelToWorkAreas, area: usize, node2area: &Vec<usize>) -> f64 {
    let area_to_node = flow_area_to_node(graph, node, area, node2area);
    let node_to_area = flow_node_to_area(graph, node, area, node2area);

    let to_area = areas[area].flow_to_area;
    let from_area = areas[area].flow_from_area;

    let a = node_to_area / graph.nodes[node].out_degree as f64;
    let b = node_to_area / to_area;
    let c = area_to_node / from_area;
    let d = area_to_node / graph.nodes[node].in_degree as f64;

    a * b + c * d
}

fn flow_area_to_node(graph: &Graph, node: NodeIndex, area: usize, node2area: &Vec<usize>) -> f64 {
    graph
        .get_edges(node, EdgeDirection::In)
        .filter(|e| node2area[e.source] == area)
        .map(|e| e.weight)
        .sum::<u32>() as f64
}

fn flow_node_to_area(graph: &Graph, node: NodeIndex, area: usize, node2area: &Vec<usize>) -> f64 {
    graph
        .get_edges(node, EdgeDirection::Out)
        .filter(|e| node2area[e.target] == area)
        .map(|e| e.weight)
        .sum::<u32>() as f64
}
