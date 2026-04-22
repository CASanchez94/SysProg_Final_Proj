# Final Project - Concurrent Task Dispatcher
The goal of this project is to implement a concurrent task dispatcher in Rust.
The dispatcher will distribute tasks to the worker threads and processes them according to the schedule policy used.

---
## Build Instructions

1. Install Rust using the official installer:

https://rustup.rs

2. Clone the repository:

git clone <your_repo_url>

3. Navigate into the project folder:

cd final_project

4. Build the project:

cargo build

---

## Run Instructions

Run the dispatcher with:

cargo run

---

## Example Command

Example:

cargo run

Example output:

Concurrent Task Dispatcher
Policy: FIFO
Workers: 4
Tasks processed: 20

---

## Summary

## Design Summary

This project implements a concurrent task dispatcher using a central dispatcher architecture.
Tasks are stored in a shared queue and distributed to worker threads for execution.

The dispatcher coordinates task assignment while worker threads process tasks concurrently.

The system ensures safe concurrent execution while maintaining the selected scheduling policy.

---

Will be updated with tools used once started.
