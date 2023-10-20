#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Node {
    pub id: usize,
    pub in_degree: i32,
    pub out_degree: i32,
    pub out_edges: Vec<Edge>,
    pub in_edges: Vec<Edge>,
    pub area_id: usize,
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
            area_id: usize::MAX,
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
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[allow(dead_code)]
impl Graph {
    fn new() -> Graph {
        Graph {
            nodes: Vec::new(),
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
        self.nodes.insert(node.id, node.clone());
    }

    fn add_edge(&mut self, edge: Edge) {
        // Update degrees
        // This won't work if we've removed nodes

        self.nodes[edge.source].out_degree += edge.weight;
        self.nodes[edge.target].in_degree += edge.weight;

        // Update edges
        self.nodes[edge.source].out_edges.push(edge.clone());

        self.nodes[edge.target].in_edges.push(edge.clone());

        self.edges.push(edge);
    }

    fn remove_edge(&mut self, edge: Edge) {
        // Update degrees
        self.nodes[edge.source].out_degree -= edge.weight;
        self.nodes[edge.target].in_degree -= edge.weight;

        // Update edges
        self.nodes[edge.source].out_edges.retain(|e| e != &edge);
        self.nodes[edge.target].in_edges.retain(|e| e != &edge);

        self.edges.retain(|e| e != &edge);
    }
}
