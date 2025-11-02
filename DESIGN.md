# Seadog Scheduler - Phase 1: Minimal Scheduler Design

## Overview

The Seadog Scheduler is a minimal task scheduler that executes tasks organized as a Directed Acyclic Graph (DAG). This Phase 1 implementation focuses on core functionality without distributed execution, complex scheduling plans, or other advanced features.

## Architecture

### Core Components

#### 1. Node
- Represents a single task in the DAG
- Contains a unique identifier (string ID)
- Each node is associated with a task function

#### 2. DAG (Directed Acyclic Graph)
- Container for nodes and their dependencies
- Maintains:
  - Set of nodes
  - Adjacency list for edges (dependencies)
  - Map of node IDs to task functions
- Provides operations to:
  - Add nodes with their task functions
  - Add edges (dependencies) between nodes
  - Perform topological sort for execution ordering
  - Detect cycles in the graph

#### 3. Scheduler
- Orchestrates the execution of DAG tasks
- Uses topological sort to determine execution order
- Executes tasks sequentially in dependency order
- Provides error handling for:
  - Cyclic dependencies
  - Task execution failures

### Task Functions

Task functions follow a simple signature:
```rust
fn task(node_id: &str) -> Result<(), String>
```

- Takes the node ID as input
- Returns `Ok(())` on success
- Returns `Err(String)` with error message on failure
- Functions are Rust functions (no external processes in Phase 1)

## Key Algorithms

### Topological Sort (Kahn's Algorithm)

The scheduler uses Kahn's algorithm for topological sorting:

1. Calculate in-degree for each node
2. Initialize queue with nodes having zero in-degree
3. Process nodes from queue:
   - Add to sorted list
   - Reduce in-degree of dependent nodes
   - Add newly zero-degree nodes to queue
4. If sorted list length ≠ total nodes, graph has a cycle

Time Complexity: O(V + E) where V = vertices, E = edges

### Execution Flow

1. **Build DAG**: Add nodes and define dependencies
2. **Validate**: Check for cycles during topological sort
3. **Execute**: Run tasks in sorted order
4. **Error Handling**: Stop on first failure, report error

## API Usage Example

```rust
use seadog_scheduler::{Dag, Node, Scheduler};

// Define task functions
fn task_a(node_id: &str) -> Result<(), String> {
    println!("Running {}", node_id);
    Ok(())
}

fn task_b(node_id: &str) -> Result<(), String> {
    println!("Running {}", node_id);
    Ok(())
}

// Build DAG
let mut dag = Dag::new();
dag.add_node(Node::new("task1"), task_a)?;
dag.add_node(Node::new("task2"), task_b)?;
dag.add_edge("task1", "task2")?;

// Execute
let scheduler = Scheduler::new(dag);
scheduler.execute()?;
```

## Features

### Implemented ✓
- DAG data structure with nodes and edges
- Topological sort for execution ordering
- Cycle detection
- Sequential task execution
- Comprehensive error handling
- Task failure propagation

### Not Implemented (Out of Scope for Phase 1)
- Distributed execution
- Parallel task execution
- Scheduling plans/strategies
- Task retry mechanisms
- Task persistence/state management
- External process execution
- Dynamic DAG modification
- Task priority or scheduling policies

## Error Handling

The implementation provides clear error messages for:
- Duplicate node addition
- Non-existent nodes when adding edges
- Cyclic dependencies in the DAG
- Task execution failures

## Testing

The implementation includes comprehensive unit tests covering:
- Basic DAG operations (add nodes, add edges)
- Edge cases (duplicate nodes, invalid edges)
- Topological sort correctness
- Cycle detection
- Scheduler execution with success and failure cases
- Complex DAG structures (parallel branches, independent tasks)

### Test Coverage
- 14 unit tests
- All tests passing
- Coverage includes:
  - DAG construction
  - Edge validation
  - Topological sorting
  - Cycle detection
  - Scheduler execution
  - Error handling

## Future Enhancements (Phase 2+)

Potential improvements for future phases:
1. **Parallel Execution**: Execute independent tasks concurrently
2. **Distributed Execution**: Run tasks across multiple nodes
3. **Scheduling Policies**: Priority-based, resource-aware scheduling
4. **State Management**: Persist DAG state and task results
5. **Dynamic DAGs**: Modify DAG structure during execution
6. **Advanced Error Handling**: Retry logic, partial failures
7. **Task Dependencies**: Data passing between tasks
8. **Monitoring**: Task metrics, execution time tracking

## Dependencies

The implementation uses only Rust standard library:
- `std::collections::HashMap` - Task and edge storage
- `std::collections::HashSet` - Node storage
- `std::collections::VecDeque` - Queue for topological sort

No external dependencies required for Phase 1.

## Performance Characteristics

- **Time Complexity**: O(V + E) for execution planning
- **Space Complexity**: O(V + E) for DAG storage
- **Execution**: Sequential, synchronous
- **Scalability**: Limited by single-process, single-threaded execution

## Limitations

1. **No Parallelism**: Tasks execute sequentially
2. **No Distribution**: Single process only
3. **No Persistence**: DAG exists only in memory
4. **Fixed Structure**: DAG cannot be modified after creation
5. **Simple Functions**: Task functions must be Rust functions (no external processes)

## Conclusion

This Phase 1 implementation provides a solid foundation for a DAG-based task scheduler. It implements core functionality with clean APIs, robust error handling, and comprehensive testing. The architecture is designed to be extensible for future enhancements while keeping the initial implementation minimal and maintainable.
