# Final Project - Concurrent Task Dispatcher
The goal of this project is to implement a concurrent task dispatcher in Rust.
The dispatcher will distribute tasks to the worker threads and processes them according to the schedule policy used.

---
## Build Instructions

1. Install Rust using the official installer:

https://rustup.rs

2. Clone the repository:
```text
git clone <your_repo_url>
```
3. Navigate into the project folder:
```text
cd final_project
```
4. Build the project:
```text
cargo build
```

---

## Run Instructions

Run the dispatcher with:
```text
cargo run
```
---

## Example Command

Example:
```text
cargo run
```
Example output:
```text
Final Project : Concurrent Task Dispatcher
  Architecture  : Central Dispatcher
  Policy        : FIFO - Simple, Fair, Starvation Free
  Workers       : 8

Starting: A: Balanced Workload
   Tasks: 500  |  Workers: 8  |  CPU fraction: 50%  |  Burst: false

  Total tasks completed :    500
  Makespan              :   1430 ms
  Avg wait time         :    148 ms
  Avg turnaround time   :    173 ms
  Max wait time         :    316 ms
```
---

## Design Summary

This project implements a concurrent task dispatcher using a central dispatcher
architecture with four major components.
The generator thread creates tasks using a fixed random seed, assigns each
one a kind (CPU or IO), a duration, and an arrival timestamp, and sends them
one at a time over an unbounded mpsc channel with a configurable inter-arrival
gap.
The dispatcher thread receives tasks from the generator channel, stamps
each with a dispatch time, and forwards jobs to the thread pool. It exits
automatically when the generator channel closes.
The thread pool holds eight worker threads, each pulling from a shared
bounded sync_channel (capacity 100). CPU tasks are simulated with a busy
counter loop; IO tasks use thread::sleep. Each worker sends a CompletionRecord
back to the main thread on finish.
The main thread joins the generator and dispatcher, drains the completion
channel for exactly as many records as a shared AtomicU64 submitted counter
reports, and prints the summary.
Shared state is minimal. The pool's receiver is wrapped in
Arc<Mutex<Receiver<Message>>> so all eight workers compete for the same
queue; the Mutex is held only during recv, not during execution. The submitted
count is an Arc<AtomicU64> shared between the dispatcher and main thread.
The scheduling policy is FIFO. Tasks pass through two ordered channels in
arrival order with no reordering or prioritization. FIFO was chosen because it
matches the channel architecture naturally, requires no decision logic in the
dispatcher, and guarantees that no task is skipped in favor of another.

# Experiment Summary
Experiment A — Balanced Workload

With an even 50/50 split of CPU and IO tasks and no burst arrivals, the
workload distributed evenly across all eight workers. Wait times were moderate
and turnaround stayed close to wait plus execution time, showing the system was
keeping up with arrivals. Worker utilization exceeded 100% due to a timestamp
accounting issue where turnaround time was used instead of raw duration — the
utilization logic was identified as a known bug and the fix is described in the
written report.

Experiment B — Stressed Workload
With 80% CPU-heavy tasks, burst arrivals, and durations up to 80 ms, the
system was under sustained pressure throughout. Average wait time rose to
200 ms and max wait reached 410 ms. FIFO's head-of-line blocking was the main
cause: short IO tasks as fast as 2 ms were forced to wait behind long CPU tasks
ahead of them in the queue with no way to skip forward. The makespan nearly
doubled compared to Experiment A, and the sync_channel's capacity of 100
added additional back-pressure when workers were slow to drain. Workers stayed
at near-full utilization for the duration, confirming the pool was the
bottleneck rather than the dispatcher.

---
## Tools Disclosure

**Tools used:** Claude.ai (claude.ai)

**Kind of help provided:** Claude was used to review code correctness,
check whether the project met grading requirements, and suggest fixes
for bugs and warnings encountered during development.

**Example of advice accepted:** Claude identified that the worker
utilization calculation was inflated because it used `turnaround_ms -
wait_ms` instead of the task's actual `duration_ms`. Accepting this fix
brought utilization values into a realistic range.

**Example of advice rejected or fixed:** Claude suggested removing the
`id` field from the `Worker` struct entirely to resolve a dead_code
warning. This would have broken the code because `id` is passed into
`job(id)` inside the worker thread closure. The fix applied instead was
`#[allow(dead_code)]` on the field, which silences the warning without
breaking anything.
