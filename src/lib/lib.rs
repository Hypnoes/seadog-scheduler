use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

/// A node in the DAG that represents a task to be executed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node {
    pub id: String,
}

impl Node {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// Task function type - takes a node ID and returns a Result
pub type TaskFn = fn(&str) -> Result<(), String>;

/// Represents a Directed Acyclic Graph (DAG) for task scheduling
pub struct Dag {
    nodes: HashSet<Node>,
    edges: HashMap<String, Vec<String>>, // node_id -> list of dependent node_ids
    tasks: HashMap<String, TaskFn>,
}

impl Dag {
    /// Creates a new empty DAG
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    /// Adds a node to the DAG
    pub fn add_node(&mut self, node: Node, task: TaskFn) -> Result<(), String> {
        if self.nodes.contains(&node) {
            return Err(format!("Node '{}' already exists", node.id));
        }
        self.nodes.insert(node.clone());
        self.tasks.insert(node.id.clone(), task);
        self.edges.insert(node.id, Vec::new());
        Ok(())
    }

    /// Checks if a node exists by ID
    fn has_node(&self, id: &str) -> bool {
        self.nodes.iter().any(|node| node.id == id)
    }

    /// Adds a directed edge from `from` to `to` (from must complete before to)
    pub fn add_edge(&mut self, from: &str, to: &str) -> Result<(), String> {
        if !self.has_node(from) {
            return Err(format!("Source node '{}' does not exist", from));
        }
        if !self.has_node(to) {
            return Err(format!("Target node '{}' does not exist", to));
        }

        self.edges.get_mut(from).unwrap().push(to.to_string());

        Ok(())
    }

    /// Performs topological sort to get execution order
    /// Returns Err if the graph has a cycle
    fn topological_sort(&self) -> Result<impl Iterator<Item = String>, String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Initialize in-degree for all nodes
        for node in &self.nodes {
            in_degree.insert(node.id.clone(), 0);
        }

        // Calculate in-degrees
        for deps in self.edges.values() {
            for dep in deps {
                *in_degree.get_mut(dep).unwrap() += 1;
            }
        }

        // Queue for nodes with no dependencies
        let mut queue: VecDeque<String> = VecDeque::new();
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        let mut sorted = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            sorted.push(node_id.clone());

            if let Some(dependents) = self.edges.get(&node_id) {
                for dependent in dependents {
                    let degree = in_degree.get_mut(dependent).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        // If we haven't processed all nodes, there's a cycle
        if sorted.len() != self.nodes.len() {
            return Err("DAG contains a cycle".to_string());
        }

        Ok(sorted.into_iter())
    }

    /// Gets the task function for a node
    fn get_task(&self, node_id: &str) -> Option<&TaskFn> {
        self.tasks.get(node_id)
    }
}

impl Default for Dag {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Dag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dag")
            .field("nodes", &self.nodes)
            .field("edges", &self.edges)
            .finish()
    }
}

/// Scheduler that executes DAG nodes in topological order
pub struct Scheduler {
    dag: Dag,
}

impl Scheduler {
    /// Creates a new scheduler with the given DAG
    pub fn new(dag: Dag) -> Self {
        Self { dag }
    }

    /// Executes all tasks in the DAG in topological order
    /// Returns Ok(()) if all tasks succeed, or Err with details if any task fails
    pub fn execute(&self) -> Result<(), String> {
        // Get execution order
        let execution_order = self.dag.topological_sort()?;

        // Execute tasks in order
        for node_id in execution_order {
            if let Some(task) = self.dag.get_task(&node_id) {
                task(&node_id).map_err(|e| format!("Task '{}' failed: {}", node_id, e))?;
            }
        }

        Ok(())
    }

    /// Gets the execution order without running tasks
    pub fn get_execution_order(&self) -> Result<Vec<String>, String> {
        self.dag.topological_sort().map(|iter| iter.collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sample task functions for testing
    fn task_a(_node_id: &str) -> Result<(), String> {
        Ok(())
    }

    fn task_b(_node_id: &str) -> Result<(), String> {
        Ok(())
    }

    fn task_c(_node_id: &str) -> Result<(), String> {
        Ok(())
    }

    fn task_fail(_node_id: &str) -> Result<(), String> {
        Err("Task failed intentionally".to_string())
    }

    #[test]
    fn test_create_empty_dag() {
        let dag = Dag::new();
        assert_eq!(dag.nodes.len(), 0);
    }

    #[test]
    fn test_add_single_node() {
        let mut dag = Dag::new();
        let result = dag.add_node(Node::new("task1"), task_a);
        assert!(result.is_ok());
        assert_eq!(dag.nodes.len(), 1);
    }

    #[test]
    fn test_add_duplicate_node() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("task1"), task_a).unwrap();
        let result = dag.add_node(Node::new("task1"), task_a);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_edge_between_existing_nodes() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("task1"), task_a).unwrap();
        dag.add_node(Node::new("task2"), task_b).unwrap();
        let result = dag.add_edge("task1", "task2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_edge_with_nonexistent_source() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("task2"), task_b).unwrap();
        let result = dag.add_edge("task1", "task2");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_edge_with_nonexistent_target() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("task1"), task_a).unwrap();
        let result = dag.add_edge("task1", "task2");
        assert!(result.is_err());
    }

    #[test]
    fn test_topological_sort_linear_chain() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("task1"), task_a).unwrap();
        dag.add_node(Node::new("task2"), task_b).unwrap();
        dag.add_node(Node::new("task3"), task_c).unwrap();
        dag.add_edge("task1", "task2").unwrap();
        dag.add_edge("task2", "task3").unwrap();

        let order: Vec<String> = dag.topological_sort().unwrap().collect();
        assert_eq!(order, vec!["task1", "task2", "task3"]);
    }

    #[test]
    fn test_topological_sort_with_parallel_branches() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("task1"), task_a).unwrap();
        dag.add_node(Node::new("task2"), task_b).unwrap();
        dag.add_node(Node::new("task3"), task_c).unwrap();
        dag.add_node(Node::new("task4"), task_a).unwrap();

        dag.add_edge("task1", "task2").unwrap();
        dag.add_edge("task1", "task3").unwrap();
        dag.add_edge("task2", "task4").unwrap();
        dag.add_edge("task3", "task4").unwrap();

        let order: Vec<String> = dag.topological_sort().unwrap().collect();
        assert_eq!(order[0], "task1");
        assert_eq!(order[3], "task4");
        assert!(order[1] == "task2" || order[1] == "task3");
        assert!(order[2] == "task2" || order[2] == "task3");
    }

    #[test]
    fn test_topological_sort_detects_cycle() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("task1"), task_a).unwrap();
        dag.add_node(Node::new("task2"), task_b).unwrap();
        dag.add_node(Node::new("task3"), task_c).unwrap();

        dag.add_edge("task1", "task2").unwrap();
        dag.add_edge("task2", "task3").unwrap();
        dag.add_edge("task3", "task1").unwrap(); // Creates a cycle

        let result = dag.topological_sort();
        assert!(result.is_err());
    }

    #[test]
    fn test_scheduler_execute_simple_dag() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("task1"), task_a).unwrap();
        dag.add_node(Node::new("task2"), task_b).unwrap();
        dag.add_edge("task1", "task2").unwrap();

        let scheduler = Scheduler::new(dag);
        let result = scheduler.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_scheduler_execute_with_failure() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("task1"), task_a).unwrap();
        dag.add_node(Node::new("task2"), task_fail).unwrap();
        dag.add_edge("task1", "task2").unwrap();

        let scheduler = Scheduler::new(dag);
        let result = scheduler.execute();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("task2"));
    }

    #[test]
    fn test_scheduler_get_execution_order() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("task1"), task_a).unwrap();
        dag.add_node(Node::new("task2"), task_b).unwrap();
        dag.add_node(Node::new("task3"), task_c).unwrap();
        dag.add_edge("task1", "task2").unwrap();
        dag.add_edge("task2", "task3").unwrap();

        let scheduler = Scheduler::new(dag);
        let order = scheduler.get_execution_order().unwrap();
        assert_eq!(order, vec!["task1", "task2", "task3"]);
    }

    #[test]
    fn test_scheduler_detects_cycle() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("task1"), task_a).unwrap();
        dag.add_node(Node::new("task2"), task_b).unwrap();
        dag.add_edge("task1", "task2").unwrap();
        dag.add_edge("task2", "task1").unwrap();

        let scheduler = Scheduler::new(dag);
        let result = scheduler.execute();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cycle"));
    }

    #[test]
    fn test_independent_tasks() {
        let mut dag = Dag::new();
        dag.add_node(Node::new("task1"), task_a).unwrap();
        dag.add_node(Node::new("task2"), task_b).unwrap();
        dag.add_node(Node::new("task3"), task_c).unwrap();
        // No edges - all tasks are independent

        let scheduler = Scheduler::new(dag);
        let result = scheduler.execute();
        assert!(result.is_ok());

        let order = scheduler.get_execution_order().unwrap();
        assert_eq!(order.len(), 3);
    }
}
