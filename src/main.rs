mod dag;

use dag::{Dag, Node};

fn example_task_a() -> Result<(), String> {
    println!("Task A");
    Ok(())
}

fn example_task_b() -> Result<(), String> {
    println!("Task B");
    Ok(())
}

fn main() {
    let mut dag = Dag::new("example_dag".to_string());
    let task_a = Node::new("example_task_a".to_string(), example_task_a);
    let task_b = Node::new("example_task_b".to_string(), example_task_b);
    dag.add_edge(task_a.clone(), task_b.clone());

    let results = dag.execute();
    match result {
        Ok(()) => println!("Task completed successfully"),
        Err(err) => println!("Task failed with error: {}", err),
    }
    println!("Execution completed");
}
