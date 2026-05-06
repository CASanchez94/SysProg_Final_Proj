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
---

### Experiment A: Balanced Workload

---

## Appendix — Experiment Output

### Experiment A: Balanced Workload

**Configuration:**
- Tasks: 500 | Workers: 8 | CPU fraction: 50% | Burst: false
- CPU duration: 5–30 ms | IO duration: 10–40 ms
- Arrival gap: 0–3 ms | Seed: 42

```text
---------------------------------------------------------
 RESULTS: A: Balanced Workload
---------------------------------------------------------
|  Total tasks completed :    500                       |
|  Last task finished    :    494                       |
|  Makespan              :   1401 ms                     |
|  Avg wait time         :    119 ms                     |
|  Avg turnaround time   :    141 ms                     |
|  Max wait time         :    294 ms                     |
|  Min available workers :      0 / 8                   |
|  Worker utilization:                                   |
|    Worker  0  :  94%                                   |
|    Worker  1  :  95%                                   |
|    Worker  2  :  93%                                   |
|    Worker  3  :  94%                                   |
|    Worker  4  :  92%                                   |
|    Worker  5  :  93%                                   |
|    Worker  6  :  93%                                   |
|    Worker  7  :  93%                                   |
---------------------------------------------------------
```

---

### Experiment B: Stressed Workload

**Configuration:**
- Tasks: 600 | Workers: 8 | CPU fraction: 80% | Burst: true
- CPU duration: 20–80 ms | IO duration: 2–10 ms
- Arrival gap: 0–20 ms | Seed: 99

```text
---------------------------------------------------------
 RESULTS: B: Stressed Workload
---------------------------------------------------------
|  Total tasks completed :    600                       |
|  Last task finished    :    599                       |
|  Makespan              :   3472 ms                     |
|  Avg wait time         :    455 ms                     |
|  Avg turnaround time   :    763 ms                     |
|  Max wait time         :    692 ms                     |
|  Min available workers :      0 / 8                   |
|  Worker utilization:                                   |
|    Worker  0  :  91%                                   |
|    Worker  1  :  90%                                   |
|    Worker  2  :  90%                                   |
|    Worker  3  :  90%                                   |
|    Worker  4  :  91%                                   |
|    Worker  5  :  90%                                   |
|    Worker  6  :  90%                                   |
|    Worker  7  :  91%                                   |
---------------------------------------------------------
```

---

### Comparison

| Metric | Experiment A | Experiment B | Change |
|---|---|---|---|
| Tasks | 500 | 600 | +20% |
| Makespan | 1401 ms | 3472 ms | +148% |
| Avg wait time | 119 ms | 455 ms | +282% |
| Avg turnaround | 141 ms | 763 ms | +441% |
| Max wait time | 294 ms | 692 ms | +135% |
| Avg utilization | ~93% | ~90% | −3% |

Experiment B took 148% longer to complete despite only 20% more tasks.
The shift to 80% CPU-heavy tasks with longer durations (20–80 ms) and
burst arrivals caused avg wait to rise nearly 4x and avg turnaround to
rise over 5x compared to Experiment A. FIFO's head-of-line blocking is
the primary cause — short IO tasks of 2–10 ms were forced to wait behind
CPU tasks ahead of them with no ability to skip forward. Worker
utilization stayed near-identical in both experiments (~90–95%),
confirming the workers were never idle — the bottleneck was task ordering,
not pool capacity.

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
