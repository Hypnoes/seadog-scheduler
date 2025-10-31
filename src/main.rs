/*--------------------------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See https://go.microsoft.com/fwlink/?linkid=2090316 for license information.
 *-------------------------------------------------------------------------------------------------------------*/

use seadog_scheduler::{Dag, Node, Scheduler};

// Example task functions
fn download_data(node_id: &str) -> Result<(), String> {
    println!("Executing task: {} - Downloading data...", node_id);
    Ok(())
}

fn process_data(node_id: &str) -> Result<(), String> {
    println!("Executing task: {} - Processing data...", node_id);
    Ok(())
}

fn upload_results(node_id: &str) -> Result<(), String> {
    println!("Executing task: {} - Uploading results...", node_id);
    Ok(())
}

fn main() {
    println!("=== Seadog Scheduler - DAG Execution Example ===\n");

    // Create a DAG
    let mut dag = Dag::new();

    // Add nodes with their tasks
    dag.add_node(Node::new("download"), download_data).unwrap();
    dag.add_node(Node::new("process"), process_data).unwrap();
    dag.add_node(Node::new("upload"), upload_results).unwrap();

    // Define dependencies (edges)
    dag.add_edge("download", "process").unwrap();
    dag.add_edge("process", "upload").unwrap();

    // Create scheduler
    let scheduler = Scheduler::new(dag);

    // Show execution order
    match scheduler.get_execution_order() {
        Ok(order) => {
            println!("Execution order: {:?}\n", order);
        }
        Err(e) => {
            eprintln!("Error getting execution order: {}", e);
            return;
        }
    }

    // Execute the DAG
    match scheduler.execute() {
        Ok(_) => {
            println!("\n✓ All tasks completed successfully!");
        }
        Err(e) => {
            eprintln!("\n✗ Execution failed: {}", e);
        }
    }
}
