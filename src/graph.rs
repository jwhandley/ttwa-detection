use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Node {
    pub id: usize,
    pub in_degree: i32,
    pub out_degree: i32,
    pub out_edges: Vec<Edge>,
    pub in_edges: Vec<Edge>,
}

#[allow(dead_code)]
impl Node {
    fn new(id: usize) -> Node {
        Node {
            id,
            in_degree: 0,
            out_degree: 0,
            out_edges: Vec::new(),
            in_edges: Vec::new(),
        }
    }
}

#[derive(PartialEq, Clone, Debug, Eq, Hash)]
pub struct Edge {
    pub source: usize,
    pub target: usize,
    pub weight: i32,
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: HashMap<usize, Node>,
    pub edges: Vec<Edge>,
}

#[allow(dead_code)]
impl Graph {
    fn new() -> Graph {
        Graph {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }
    pub fn from_adjacency_matrix(adjacency: Vec<Vec<i32>>) -> Graph {
        let mut graph = Graph::new();

        for index in 0..adjacency.len() {
            graph.add_node(&Node::new(index));
        }

        for (i, row) in adjacency.iter().enumerate() {
            for (j, &weight) in row.iter().enumerate() {
                if weight > 0 {
                    graph.add_edge(Edge {
                        source: i,
                        target: j,
                        weight,
                    });
                }
            }
        }

        graph
    }

    fn add_node(&mut self, node: &Node) {
        self.nodes.insert(node.id,node.clone());
    }

    fn remove_node(&mut self, node: &Node) {
        // Update nodes
        self.nodes.remove(&node.id);

        // Update all edges
        self.edges.retain(|e| e.source != node.id && e.target != node.id);
    }

    fn add_edge(&mut self, edge: Edge) {
        // Update degrees
        // This won't work if we've removed nodes

        self.nodes.get_mut(&edge.source).unwrap().out_degree += edge.weight;
        self.nodes.get_mut(&edge.target).unwrap().in_degree += edge.weight;

        // Update edges
        self.nodes.get_mut(&edge.source)
            .unwrap()
            .out_edges
            .push(edge.clone());

        self.nodes.get_mut(&edge.target)
            .unwrap()
            .in_edges
            .push(edge.clone());

        self.edges.push(edge);
    }

    fn remove_edge(&mut self, edge: Edge) {
        // Update degrees
        self.nodes.get_mut(&edge.source).unwrap().out_degree -= edge.weight;
        self.nodes.get_mut(&edge.target).unwrap().in_degree -= edge.weight;

        // Update edges
        self.nodes.get_mut(&edge.source)
            .unwrap()
            .out_edges
            .retain(|e| e != &edge);
        self.nodes.get_mut(&edge.target)
            .unwrap()
            .in_edges
            .retain(|e| e != &edge);

        self.edges.retain(|e| e != &edge);
    }
}