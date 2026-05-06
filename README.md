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
  Avg wait time          : 9125 ms
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

This project implements a concurrent task dispatcher using a central manager architecture with five major components. The generator thread creates tasks using a fixed random seed, assigns each one a kind (CPU or IO) and an arrival timestamp, and sends them one at a time over an unbounded mpsc channel with a fixed 20ms inter-arrival gap. The manager runs on the main experiment thread, holds tasks in an internal Vec queue, and makes the dispatch decision for each task before it reaches a worker. Before dispatching, the manager checks two things: whether a free worker is available and whether the task's CPU cost would push the global cpu_pct over 100 percent. If either check fails the task stays in the queue until a worker finishes and sends a release signal. Eight worker threads share a bounded sync_channel with capacity 32. CPU tasks are simulated with a busy counter loop and IO tasks use thread::sleep, both for 200ms. Each worker decrements the shared state when it finishes and sends a CompletionRecord back to the main thread. A monitor thread runs independently, sampling active worker count and CPU percentage every 10ms into a log that is used to compute averages at the end. Shared state is managed through a single Arc<Mutex<SharedState>> struct containing active_workers, cpu_pct, and queue_len. A separate Arc<Mutex<u64>> submitted counter tracks how many tasks the manager has dispatched so the main thread knows exactly how many completion records to collect. Two scheduling policies are implemented. FIFO holds all tasks in a single Vec and always dispatches from the front, preserving strict arrival order. The Optimized policy splits tasks into separate CPU and IO queues and fills available CPU headroom greedily — preferring CPU tasks when at least 35 percent is free and falling back to IO tasks when headroom is tighter. This keeps more workers busy simultaneously at the cost of strict arrival-order fairness.

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
  Avg wait time          : 9125 ms
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
  Avg wait time          : 5402 ms
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

FIFO produced a shorter makespan by 6.7%, meaning it finished all 1000 tasks faster overall. This is because the workload is IO-dominated at 70%, so the CPU cap rarely becomes a bottleneck and FIFO's simple send-the-next-task approach keeps workers busy without hesitation. The Optimized policy is more conservative — it pauses to check headroom before dispatching, which occasionally leaves workers idle when FIFO would have just sent the next IO task immediately. However the Optimized policy reduced average wait time by 40.8% and average turnaround by 40.4%, meaning individual tasks spent significantly less time in the system. The trade-off is clear: FIFO wins on total runtime for IO-heavy workloads, while the Optimized packing strategy would show its advantage on CPU-heavy workloads where the 100% cap is frequently hit and smarter dispatch decisions matter more.


---
## Tools Disclosure

**Tools used:** Claude.ai (claude.ai)

**Kind of help provided:** Claude was used to review code correctness,
check whether the project met grading requirements, and suggest fixes
for bugs and warnings encountered during development.

Example of advice accepted:
Claude identified that the simulate_cpu_work function originally ended with let _ = counter, which gives the compiler permission to discard the loop entirely since the result is never used. Accepting the fix to use counter.wrapping_add(1) inside the loop ensures the compiler keeps the work, so CPU tasks consistently run for the full 200ms as intended.

Example of advice rejected or fixed:
Claude suggested adding a duration_ms field to CompletionRecord and using it for worker utilization calculations instead of turnaround_ms - wait_ms. While this is technically the more correct approach, the current code measures wait time from dispatch time rather than arrival time, so turnaround_ms - wait_ms produces accurate execution time in this implementation. The suggestion was noted but not applied since the existing calculation already gives correct results given how 
