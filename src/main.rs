mod dag;
mod task;

use dag::{Dag, TaskNode};

fn example_task_a() -> Result<(), String> {
    println!("Task A");
    Ok(())
}

fn example_task_b() -> Result<(), String> {
    println!("Task B");
    Ok(())
}

fn main() -> Result<(), String> {
    let mut dag = Dag::new("example_dag".to_string());
    let task_a = TaskNode::new("example_task_a".to_string(), example_task_a);
    let task_b = TaskNode::new("example_task_b".to_string(), example_task_b);
    dag.add_task_relation(task_a.clone(), task_b.clone());

    let result = dag.execute();

    match result {
        Ok(_) => println!("All tasks completed successfully"),
        Err(err) => println!("Error: {}", err),
    }

    Ok(())
}
