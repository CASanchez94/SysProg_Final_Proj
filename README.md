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
---------------------------------------------------------
 RESULTS: A: FIFO, 70% IO / 30% CPU
---------------------------------------------------------
  Total tasks completed  : 1000
  CPU tasks completed    : 299
  IO tasks completed     : 701
  Makespan               : 39165 ms
  Avg wait time          : 0 ms
  Avg turnaround time    : 9328 ms
---------------------------------------------------------
  Monitor (sampled every 10ms):
  Avg CPU usage          : 89.4%
  Peak CPU usage         : 100%
  Avg active workers     : 5.1 / 8
---------------------------------------------------------
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
---
### Experiment A: FIFO, 70% IO / 30% CPU

**Configuration:**
Tasks: 1000 | Workers: 8 | IO fraction: 70% | Scheduler: FIFO
IO task: sleep(200ms), 10% CPU cost | CPU task: spin(200ms), 35% CPU cost
Arrival gap: 20ms | Seed: 42

```text
---------------------------------------------------------
 RESULTS: A: FIFO, 70% IO / 30% CPU
---------------------------------------------------------
  Total tasks completed  : 1000
  CPU tasks completed    : 299
  IO tasks completed     : 701
  Makespan               : 39165 ms
  Avg wait time          : 0 ms
  Avg turnaround time    : 9328 ms
---------------------------------------------------------
  Monitor (sampled every 10ms):
  Avg CPU usage          : 89.4%
  Peak CPU usage         : 100%
  Avg active workers     : 5.1 / 8
---------------------------------------------------------
```

### Experiment B: Optimized, 70% IO / 30% CPU

**Configuration:**
Tasks: 1000 | Workers: 8 | IO fraction: 70% | Scheduler: Optimized
IO task: sleep(200ms), 10% CPU cost | CPU task: spin(200ms), 35% CPU cost
Arrival gap: 20ms | Seed: 42

```text
---------------------------------------------------------
 RESULTS: B: Optimized, 70% IO / 30% CPU
---------------------------------------------------------
  Total tasks completed  : 1000
  CPU tasks completed    : 299
  IO tasks completed     : 701
  Makespan               : 41784 ms
  Avg wait time          : 0 ms
  Avg turnaround time    : 5557 ms
---------------------------------------------------------
  Monitor (sampled every 10ms):
  Avg CPU usage          : 83.7%
  Peak CPU usage         : 100%
  Avg active workers     : 4.8 / 8
---------------------------------------------------------
```

### Comparison

| Metric | Exp A (FIFO) | Exp B (Optimized) | Change |
|---|---|---|---|
| Tasks | 1000 | 1000 | — |
| Makespan | 39165 ms | 41784 ms | +6.7% |
| Avg wait time | 0 ms | 0 ms | — |
| Avg turnaround | 9328 ms | 5557 ms | −40.4% |
| Avg CPU usage | 89.4% | 83.7% | −5.7% |
| Peak CPU usage | 100% | 100% | — |
| Avg active workers | 5.1 / 8 | 4.8 / 8 | −5.9% |

Both schedulers completed all 1000 tasks with zero wait time, meaning tasks were dispatched
as fast as they arrived in both cases. The key difference shows up in avg turnaround: the
optimized scheduler reduced it by 40.4% (9328ms → 5557ms) by prioritizing CPU tasks first
and packing the CPU budget greedily, so individual tasks spent less time in the system overall.
The tradeoff is a slightly longer makespan (+6.7%) — the optimized scheduler was more
conservative about which tasks to run simultaneously, leaving workers idle when no task fit
the remaining CPU budget cleanly, while FIFO just sent whatever was next regardless.


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
