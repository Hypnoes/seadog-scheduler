use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::task::Task;

// pub type Task = fn() -> Result<(), String>;

pub struct TaskNode {
    id: String,
    pub name: String,
    task: Arc<dyn Task>,
}

impl TaskNode {
    pub fn new<T: Task + 'static>(name: String, task: T) -> Self {
        TaskNode {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            task: Arc::new(task),
        }
    }

    pub fn execute(&self) -> Result<(), String> {
        (self.task).execute()
    }
}

impl PartialEq for TaskNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TaskNode {}

impl Hash for TaskNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Clone for TaskNode {
    fn clone(&self) -> Self {
        TaskNode {
            id: self.id.clone(),
            name: self.name.clone(),
            task: self.task.clone(),
        }
    }
}

impl Debug for TaskNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node({})", self.name)
    }
}

pub struct Dag {
    name: String,
    node_table: HashMap<TaskNode, Vec<TaskNode>>,
    reverse_table: HashMap<TaskNode, Vec<TaskNode>>,
    indegree: HashMap<TaskNode, usize>,
}

impl Dag {
    pub fn new(name: String) -> Self {
        Dag {
            name,
            node_table: HashMap::new(),
            reverse_table: HashMap::new(),
            indegree: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task_node: TaskNode) {
        self.node_table.insert(task_node.clone(), Vec::new());
        self.reverse_table.insert(task_node.clone(), Vec::new());
        self.indegree.insert(task_node, 0);
    }

    pub fn add_task_relation(&mut self, from: TaskNode, to: TaskNode) {
        if !self.node_table.contains_key(&from) {
            self.node_table.insert(from.clone(), Vec::new());
            self.reverse_table.insert(from.clone(), Vec::new());
            self.indegree.insert(from.clone(), 0);
        }

        if !self.node_table.contains_key(&to) {
            self.node_table.insert(to.clone(), Vec::new());
            self.reverse_table.insert(to.clone(), Vec::new());
            self.indegree.insert(to.clone(), 0);
        }

        self.node_table
            .entry(from.clone())
            .or_insert(Vec::new())
            .push(to.clone());

        self.reverse_table
            .entry(to.clone())
            .or_insert(Vec::new())
            .push(from.clone());

        self.indegree
            .entry(to.clone())
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    pub fn get_all_tasks(&self) -> Vec<TaskNode> {
        self.node_table.keys().cloned().collect()
    }

    /// Topological sort of the DAG
    ///
    /// Uses Kahn's algorithm to perform a topological sort on the DAG.
    /// First finds all nodes with an indegree of 0 and adds them to the queue.
    /// Then, iteratively removes nodes from the queue, reducing the indegree of their neighbors.
    /// If a neighbor's indegree reaches 0, it is added to the queue.
    /// Until the queue is empty, the nodes are added to the result vector.
    ///
    /// Returns a vector of nodes in topological order.
    pub fn resolve_execution_order(&self) -> Result<Vec<TaskNode>, String> {
        if self.node_table.is_empty() {
            return Err("No nodes found".into());
        }

        let mut result: Vec<TaskNode> = Vec::with_capacity(self.get_all_tasks().len());

        // Initialize indegree map
        let mut indegree: HashMap<&TaskNode, usize> = self
            .indegree
            .iter()
            .map(|(node, &deg)| (node, deg))
            .collect();

        // Find all nodes with an indegree of 0 and add them to the queue
        let mut queue: VecDeque<&TaskNode> = indegree
            .iter()
            .filter_map(|(&node, &deg)| if deg == 0 { Some(node) } else { None })
            .collect();

        // Start topological sort
        while let Some(current_node) = queue.pop_front() {
            // Clone only when pushing into the final result
            result.push(current_node.clone());
            if let Some(neighbors) = self.node_table.get(current_node) {
                for neighbor in neighbors {
                    // Remove current node (indegree = 0) and update indegree count
                    if let Some(d) = indegree.get_mut(neighbor) {
                        if *d > 0 {
                            // Decrement indegree count (cause we removed current node)
                            *d -= 1;
                            if *d == 0 {
                                // Add node to queue if indegree count reaches zero
                                queue.push_back(neighbor);
                            }
                        }
                    }
                }
            }
        }

        if result.len() != indegree.len() {
            return Err("Graph has at least one cycle".into());
        }

        Ok(result)
    }

    pub fn execute(&self) -> Result<(), String> {
        self.resolve_execution_order()?
            .into_iter()
            .try_for_each(|node| node.execute())
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
        let n = TaskNode::new("ok_task".to_string(), ok_task);
        assert_eq!(n.execute().unwrap(), ());
    }

    #[test]
    fn test_node_execute_err() {
        let n = TaskNode::new("err_task".to_string(), err_task);
        assert!(n.execute().is_err());
    }

    #[test]
    fn test_node_ops() {
        let mut dag = Dag::new("g".to_string());
        let n = TaskNode::new("n".to_string(), ok_task);
        dag.add_task(n.clone());
        let nodes = dag.get_all_tasks();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_edge_ops() {
        let mut dag = Dag::new("g".to_string());
        let a = TaskNode::new("a".to_string(), ok_task);
        let b = TaskNode::new("b".to_string(), ok_task);
        dag.add_task_relation(a.clone(), b.clone());
        assert_eq!(dag.get_all_tasks().len(), 2);
        assert_eq!(dag.node_table.get(&a).unwrap().len(), 1);
        assert_eq!(dag.node_table.get(&b).unwrap().len(), 0);
    }

    #[test]
    fn test_dag_topo_sort_ok() {
        let mut dag = Dag::new("g".into());
        let a = TaskNode::new("a".to_string(), ok_task);
        let b = TaskNode::new("b".to_string(), ok_task);
        let c = TaskNode::new("c".to_string(), ok_task);
        let d = TaskNode::new("d".to_string(), ok_task);
        let e = TaskNode::new("e".to_string(), ok_task);
        dag.add_task_relation(a.clone(), b.clone());
        dag.add_task_relation(a.clone(), c.clone());
        dag.add_task_relation(c.clone(), d.clone());
        dag.add_task_relation(b.clone(), e.clone());
        dag.add_task_relation(d.clone(), e.clone());
        let order = dag
            .resolve_execution_order()
            .unwrap()
            .iter()
            .map(|node| node.name.clone())
            .collect::<Vec<_>>();
        assert_eq!(order, vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn test_dag_execute() {
        let mut dag = Dag::new("g".into());
        let a = TaskNode::new("a".to_string(), ok_task);
        let b = TaskNode::new("b".to_string(), ok_task);
        let c = TaskNode::new("c".to_string(), ok_task);
        let d = TaskNode::new("d".to_string(), ok_task);
        let e = TaskNode::new("e".to_string(), ok_task);
        dag.add_task_relation(a.clone(), b.clone());
        dag.add_task_relation(a.clone(), c.clone());
        dag.add_task_relation(c.clone(), d.clone());
        dag.add_task_relation(b.clone(), e.clone());
        dag.add_task_relation(d.clone(), e.clone());
        let order = dag
            .resolve_execution_order()
            .unwrap()
            .iter()
            .map(|node| node.name.clone())
            .collect::<Vec<_>>();
        assert_eq!(order, vec!["a", "b", "c", "d", "e"]);

        let result = dag.execute();
        assert_eq!(result, Ok(()));
    }
}
