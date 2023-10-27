use std::rc::Rc;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Node {
    pub id: usize,
    pub in_degree: u32,
    pub out_degree: u32,
}

#[allow(dead_code)]
impl Node {
    fn new(id: usize) -> Node {
        Node {
            id,
            in_degree: 0,
            out_degree: 0,
        }
    }
}

#[derive(PartialEq, Clone, Debug, Eq, Hash)]
pub struct Edge {
    pub source: usize,
    pub target: usize,
    pub weight: u32,
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Rc<Edge>>,
    node_to_in_edges: Vec<Vec<Rc<Edge>>>,
    node_to_out_edges: Vec<Vec<Rc<Edge>>>,
}

pub enum EdgeDirection {
    In,
    Out,
}

#[allow(dead_code)]
impl Graph {
    fn new() -> Graph {
        Graph {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_to_in_edges: Vec::new(),
            node_to_out_edges: Vec::new(),
        }
    }
    pub fn from_adjacency_matrix(adjacency: Vec<Vec<u32>>) -> Graph {
        let mut graph = Graph::new();

        for index in 0..adjacency.len() {
            graph.add_node(Node::new(index));
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
        // Node's out_degree should be equal to the row-sum for that node in the adjacency matrix
        // Node's in_degree should be equal to the column-sum for that node in the adjacency matrix
        assert_eq!(
            graph
                .nodes
                .iter()
                .map(|node| node.out_degree)
                .collect::<Vec<u32>>(),
            adjacency
                .iter()
                .map(|row| row.iter().sum::<u32>())
                .collect::<Vec<u32>>()
        );
        assert_eq!(
            graph
                .nodes
                .iter()
                .map(|node| node.in_degree)
                .collect::<Vec<u32>>(),
            (0..adjacency.len())
                .map(|j| adjacency.iter().map(|row| row[j]).sum::<u32>())
                .collect::<Vec<u32>>()
        );

        // Sum of all edge weights should equal the sum of all values in the adjacency matrix
        assert_eq!(
            graph.edges.iter().map(|edge| edge.weight).sum::<u32>(),
            adjacency
                .iter()
                .map(|row| row.iter().sum::<u32>())
                .sum::<u32>()
        );

        graph
    }

    fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
        self.node_to_in_edges.push(Vec::new());
        self.node_to_out_edges.push(Vec::new());
    }

    fn add_edge(&mut self, edge: Edge) {
        let edge = Rc::new(edge);
        // Update degrees
        // This won't work if we've removed nodes
        let target = edge.target;
        let source = edge.source;
        self.nodes[source].out_degree += edge.weight;
        self.nodes[target].in_degree += edge.weight;

        self.edges.push(edge.clone());
        self.node_to_in_edges[target].push(edge.clone());
        self.node_to_out_edges[source].push(edge.clone());
    }

    pub fn get_edges(
        &self,
        node_index: usize,
        direction: EdgeDirection,
    ) -> impl Iterator<Item = &'_ Rc<Edge>> {
        match direction {
            EdgeDirection::In => 
                self.node_to_in_edges[node_index]
                    .iter()
            ,
            EdgeDirection::Out => 
                self.node_to_out_edges[node_index]
                    .iter()
            ,
        }
    }
}
