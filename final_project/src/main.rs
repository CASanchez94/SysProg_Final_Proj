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
#[derive(Debug, Clone, Copy)]
struct WorkloadConfig {
    num_tasks: u64,
    seed: u64,
    cpu_fraaction: f64,
    cpu_dur_min: u64,
    io_dur_min: u64,
    burst_mode: bool,
    max_arrival_gap_ms: u64,
}

// Threadpool

enum Message {
    NewJob(Job),
    Terminate,
}

type  Job = Box<dyn fnOnce(usize) + Send + 'static>;

struct Threadpool {

    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    queued_jobs: Arc<AtomicUsize>,
}

impl Threadpool{
    
}




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
