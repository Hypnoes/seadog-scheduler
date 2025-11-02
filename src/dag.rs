use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub type Task = fn() -> Result<(), String>;

pub fn default_task() -> Result<(), String> {
    Ok(())
}

pub struct Node {
    id: String,
    name: String,
    task: Task,
}

impl Node {
    pub fn new(name: String, task: Task) -> Self {
        Node {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            task,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn execute(&self) -> Result<(), String> {
        (self.task)()
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }

    fn hash_slice<H: std::hash::Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for piece in data {
            piece.hash(state)
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Node {
            id: self.id.clone(),
            name: self.name.clone(),
            task: self.task.clone(),
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Node {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            task: default_task,
        }
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node").field("name", &self.name).finish()
    }
}

/// Enum representing the direction of traversal in a graph.
///
/// This enum is used to specify the direction of traversal in a graph.
/// It can be used to traverse the graph in either breadth-first or depth-first order.
enum Direction {
    BFS,
    DFS,
}

pub struct Dag {
    name: String,
    node_table: HashMap<Node, Vec<Node>>,
}

impl Dag {
    pub fn new(name: String) -> Self {
        Dag {
            name,
            node_table: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.node_table.insert(node, Vec::new());
    }

    pub fn add_edge(&mut self, from: Node, to: Node) {
        if !self.node_table.contains_key(&from) {
            self.node_table.insert(from.clone(), Vec::new());
        }

        if !self.node_table.contains_key(&to) {
            self.node_table.insert(to.clone(), Vec::new());
        }

        self.node_table
            .get_mut(&from)
            .map(|neighbors| neighbors.push(to));
    }

    pub fn get_nodes(&self) -> Vec<Node> {
        self.node_table.keys().cloned().collect()
    }

    pub fn reset(&mut self) {
        self.node_table.clear();
    }

    fn in_degree(&self, node: &Node) -> usize {
        self.node_table
            .values()
            .filter(|neighbors| neighbors.contains(node))
            .count()
    }

    /// Iterates over the nodes in the specified direction.
    ///
    /// Returns a vector of nodes in the specified direction.
    fn iter_nodes(&self, direction: Direction) -> Result<Vec<Node>, String> {
        if self.node_table.is_empty() {
            return Err("DAG is empty".to_string());
        }

        let mut queue: VecDeque<Node> = VecDeque::new();
        let mut visited: Vec<Node> = Vec::new();

        // initialize queue with the first node
        let mut in_degrees = HashMap::new();
        for node in self.node_table.keys() {
            in_degrees.insert(node.clone(), self.in_degree(node));
        }
        for node in self.node_table.keys() {
            if in_degrees[&node] == 0 {
                queue.push_back(node.clone());
            }
        }

        // iterate over nodes in the specified direction
        let pop = match direction {
            Direction::BFS => VecDeque::pop_front,
            Direction::DFS => VecDeque::pop_back,
        };
        while let Some(node) = pop(&mut queue) {
            if !visited.contains(&node) {
                visited.push(node.clone());
            }

            if let Some(neighbors) = self.node_table.get(&node) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        Ok(visited)
    }

    pub fn bfs(&self) -> Result<Vec<Node>, String> {
        self.iter_nodes(Direction::BFS)
    }

    pub fn dfs(&self) -> Result<Vec<Node>, String> {
        self.iter_nodes(Direction::DFS)
    }

    pub fn topo_sort(&self) -> Result<Vec<Node>, String> {
        if self.node_table.is_empty() {
            return Err("No nodes found".into());
        }

        // Build indegree and adjacency (Kahn's algorithm)
        let mut indegree: HashMap<&Node, usize> = HashMap::new();
        let mut adj: HashMap<&Node, Vec<&Node>> = HashMap::new();

        for (from, neighbors) in &self.node_table {
            indegree.entry(from).or_insert(0);
            let entry = adj.entry(from).or_insert_with(Vec::new);
            for to in neighbors {
                indegree.entry(to).and_modify(|d| *d += 1).or_insert(1);
                entry.push(to);
            }
        }

        // Ensure every node exists in adjacency map
        let all_nodes: Vec<&Node> = indegree.keys().cloned().collect();
        for n in all_nodes {
            adj.entry(n).or_insert_with(Vec::new);
        }

        let mut queue: VecDeque<&Node> = indegree
            .iter()
            .filter_map(|(n, &deg)| if deg == 0 { Some(*n) } else { None })
            .collect();

        let mut result: Vec<&Node> = Vec::with_capacity(indegree.len());

        while let Some(n) = queue.pop_front() {
            result.push(n);
            if let Some(neighbors) = adj.get(n) {
                for m in neighbors {
                    if let Some(d) = indegree.get_mut(m) {
                        if *d > 0 {
                            *d -= 1;
                            if *d == 0 {
                                queue.push_back(m);
                            }
                        }
                    }
                }
            }
        }

        if result.len() != indegree.len() {
            return Err("Graph has at least one cycle".into());
        }

        Ok(result.into_iter().cloned().collect())
    }

    pub fn execute(&self) -> Vec<Result<(), String>> {
        match self.topo_sort() {
            Ok(task_order) => task_order.into_iter().map(|node| node.execute()).collect(),
            Err(err) => vec![Err(err)],
        }
    }
}

impl Debug for Dag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dag(name={}, nodes={:?})", self.name, self.node_table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_task() -> Result<(), String> {
        Ok(())
    }
    fn err_task() -> Result<(), String> {
        Err("boom".into())
    }

    #[test]
    fn test_node_execute_ok() {
        let n = Node::new("ok_task".to_string(), ok_task);
        assert_eq!(n.execute().unwrap(), ());
    }

    #[test]
    fn test_node_execute_err() {
        let n = Node::new("err_task".to_string(), err_task);
        assert!(n.execute().is_err());
    }

    #[test]
    fn test_node_ops() {
        let mut dag = Dag::new("g".to_string());
        let n = Node::new("n".to_string(), ok_task);
        dag.add_node(n.clone());
        let nodes = dag.get_nodes();
        assert_eq!(nodes.len(), 1);
        dag.reset();
        assert!(dag.get_nodes().is_empty());
    }

    #[test]
    fn test_edge_ops() {
        let mut dag = Dag::new("g".to_string());
        let a = Node::new("a".to_string(), ok_task);
        let b = Node::new("b".to_string(), ok_task);
        dag.add_edge(a.clone(), b.clone());
        assert_eq!(dag.get_nodes().len(), 2);
        assert_eq!(dag.node_table.get(&a).unwrap().len(), 1);
        assert_eq!(dag.node_table.get(&b).unwrap().len(), 0);
        dag.reset();
        assert!(dag.get_nodes().is_empty());
    }

    #[test]
    fn test_dag_bfs() {
        let mut dag = Dag::new("g".to_string());
        let a = Node::new("a".to_string(), ok_task);
        let b = Node::new("b".to_string(), ok_task);
        let c = Node::new("c".to_string(), ok_task);
        let d = Node::new("d".to_string(), ok_task);
        dag.add_edge(a.clone(), b.clone());
        dag.add_edge(a.clone(), c.clone());
        dag.add_edge(b.clone(), d.clone());
        dag.add_edge(c.clone(), d.clone());
        let bfs_result = dag
            .bfs()
            .unwrap()
            .iter()
            .map(|node| node.name.clone())
            .collect::<Vec<_>>();
        assert!(bfs_result == vec!["a", "b", "c", "d"] || bfs_result == vec!["a", "c", "b", "d"]);
    }

    #[test]
    fn test_dag_dfs() {
        let mut dag = Dag::new("g".to_string());
        let a = Node::new("a".to_string(), ok_task);
        let b = Node::new("b".to_string(), ok_task);
        let c = Node::new("c".to_string(), ok_task);
        let d = Node::new("d".to_string(), ok_task);
        dag.add_edge(a.clone(), b.clone());
        dag.add_edge(a.clone(), c.clone());
        dag.add_edge(b.clone(), d.clone());
        dag.add_edge(c.clone(), d.clone());
        let dfs_result = dag
            .dfs()
            .unwrap()
            .iter()
            .map(|node| node.name.clone())
            .collect::<Vec<_>>();
        assert!(dfs_result == vec!["a", "b", "d", "c"] || dfs_result == vec!["a", "c", "d", "b"]);
    }

    #[test]
    fn test_dag_iter_empty() {
        let dag = Dag::new("g".into());
        assert!(dag.bfs().is_err());
        assert!(dag.dfs().is_err());
    }

    #[test]
    fn test_dag_topo_sort_ok() {
        let mut dag = Dag::new("g".into());
        let a = Node::new("a".to_string(), ok_task);
        let b = Node::new("b".to_string(), ok_task);
        let c = Node::new("c".to_string(), ok_task);
        let d = Node::new("d".to_string(), ok_task);
        let e = Node::new("e".to_string(), ok_task);
        dag.add_edge(a.clone(), b.clone());
        dag.add_edge(a.clone(), c.clone());
        dag.add_edge(c.clone(), d.clone());
        dag.add_edge(b.clone(), e.clone());
        dag.add_edge(d.clone(), e.clone());
        let order = dag
            .topo_sort()
            .unwrap()
            .iter()
            .map(|node| node.name.clone())
            .collect::<Vec<_>>();
        assert_eq!(order, vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn test_dag_execute() {
        let mut dag = Dag::new("g".into());
        let a = Node::new("a".to_string(), ok_task);
        let b = Node::new("b".to_string(), ok_task);
        let c = Node::new("c".to_string(), ok_task);
        let d = Node::new("d".to_string(), ok_task);
        let e = Node::new("e".to_string(), ok_task);
        dag.add_edge(a.clone(), b.clone());
        dag.add_edge(a.clone(), c.clone());
        dag.add_edge(c.clone(), d.clone());
        dag.add_edge(b.clone(), e.clone());
        dag.add_edge(d.clone(), e.clone());
        let order = dag
            .topo_sort()
            .unwrap()
            .iter()
            .map(|node| node.name.clone())
            .collect::<Vec<_>>();
        assert_eq!(order, vec!["a", "b", "c", "d", "e"]);

        let result = dag.execute();
        assert_eq!(result, vec![Ok(()), Ok(()), Ok(()), Ok(()), Ok(())]);
    }
}
