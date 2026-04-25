use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::{atomic::{AtomicU64, AtomicUsize, Ordering}, mpsc, Arc, Mutex,};
use std::thread;
use std::time::[Duration, Instant];


// The Task Model

#[derive(Debug, Clone, Copy, PartialEq, EQ)]
enum TaskKing {
    Cpu,
    Io,
}

#derive[derive{Debug, Clone}]
struct Task {
    id: u64,
    kind: TaskKind,
    duration_ms: u64,
    arrival_time: Instant,
    dispatch_time: Option<Instant>,
}

impl Task{
    fn new(id: u64, kind: TaskKind, duration_ms: u64, arrival_time: Instant) -> Self{
        Task {
            id,
            kind,
            duration_ms,
            arrival_time: arrival,
            dispatch_time: None,
        }
    }
}

// Record of Completetion

#[derive(Debug)]
struct CompletetionRecord {
    id: u64,
    wait_ms: u64,
    turnaround_ms: u64,
    worker_id: usize,
}

// Chosen Workload Config


// Threadpool



// Simulated Behavior


// Task Gen

// Dispatcher

// Metrics and Printing

// Running one Experiment 



fn main() {
    println!("Hello, world!");

// Experiment A



// Experiment B

}
