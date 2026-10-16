use std::collections::VecDeque;
use std::fs;

struct Edge {
    to: usize,
    capacity: i64,
    flow: i64,
    reverse: usize,
}

impl Edge {
    fn residual_capacity(&self) -> i64 {
        self.capacity - self.flow
    }
}

struct Graph {
    edges: Vec<Edge>,
    adj: Vec<Vec<usize>>, // adj[node] = indices into `edges` for edges out of the node
}

impl Graph {
    fn new(node_count: usize) -> Self {
        Graph { edges: Vec::new(), adj: vec![Vec::new(); node_count] }
    }

    fn add_edge(&mut self, from: usize, to: usize, capacity: i64) {
        let i = self.edges.len();
        self.edges.push(Edge { to, capacity, flow: 0, reverse: i + 1 });
        self.adj[from].push(i);

        self.edges.push(Edge { to: from, capacity: 0, flow: 0, reverse: i });
        self.adj[to].push(i + 1);
    }
}

fn read_flow_graph(filename: &str) -> Graph {
    let content = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Finner ikke filen {}", filename));

    let mut numbers = content.split_whitespace().map(|s| {
        s.parse::<usize>()
            .unwrap_or_else(|_| panic!("feil ved tall {}: {}", filename, s))
    });

    let node_count = numbers.next().expect("mangler node antall");
    let edge_count = numbers.next().expect("mangler kant antall");

    let mut graph = Graph::new(node_count);
    for _ in 0..edge_count {
        let from = numbers.next().expect("mangler fra node");
        let to = numbers.next().expect("mangler til node");
        let capacity = numbers.next().expect("mangler kapasitet") as i64;
        graph.add_edge(from, to, capacity);
    }

    graph
}

fn find_augmenting_path(graph: &Graph, source: usize, sink: usize) -> Option<Vec<usize>> {
    let mut predecessor: Vec<Option<(usize, usize)>> = vec![None; graph.adj.len()];
    let mut visited = vec![false; graph.adj.len()];
    visited[source] = true;

    let mut queue = VecDeque::new();
    queue.push_back(source);

    while let Some(node) = queue.pop_front() {
        if node == sink {
            break;
        }
        for &e in &graph.adj[node] {
            let edge = &graph.edges[e];
            if !visited[edge.to] && edge.residual_capacity() > 0 {
                visited[edge.to] = true;
                predecessor[edge.to] = Some((node, e));
                queue.push_back(edge.to);
            }
        }
    }

    if !visited[sink] {
        return None;
    }

    let mut path = Vec::new();
    let mut node = sink;
    while let Some((from, e)) = predecessor[node] {
        path.push(e);
        node = from;
    }
    path.reverse();
    Some(path)
}


fn edmonds_karp(graph: &mut Graph, source: usize, sink: usize) -> i64 {
    println!("Maksimum flyt fra {} til {} med Edmond-Karp", source, sink);
    println!("Økning : Flytøkende vei");

    let mut max_flow = 0;
    while let Some(path) = find_augmenting_path(graph, source, sink) {
        let increase = path
            .iter()
            .map(|&e| graph.edges[e].residual_capacity())
            .min()
            .expect("ugyldig graf");

        for &e in &path {
            graph.edges[e].flow += increase;
            let reverse = graph.edges[e].reverse;
            graph.edges[reverse].flow -= increase;
        }
        max_flow += increase;

        let mut node_path = vec![source.to_string()];
        for &e in &path {
            node_path.push(graph.edges[e].to.to_string());
        }
        println!("{:>6} {}", increase, node_path.join(" "));
    }

    println!("Maksimal flyt ble {}", max_flow);
    max_flow
}

fn main() {
    let source = 0;
    let sink = 1;

    let mut graph = read_flow_graph("flytgraf3.txt");
    edmonds_karp(&mut graph, source, sink);
}
