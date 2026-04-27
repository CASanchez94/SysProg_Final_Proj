use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::{atomic::{AtomicU64, AtomicUsize, Ordering}, mpsc, Arc, Mutex,};
use std::thread;
use std::time::{Duration, Instant};


// The Task Model

#[derive(Debug, Clone, Copy, PartialEq, EQ)]
enum TaskKind {
    Cpu,
    Io,
}

#{derive{Debug, Clone}]
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
            arrival_time: arrival_time,
            dispatch_time: None,
        }
    }
}

// Record of Completetion

#[derive(Debug)]
struct CompletionRecord {
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
    cpu_fraction: f64,
    cpu_dur_min: u64,
    cpu_dur_max: u64,
    io_dur_min: u64,
    io_dur_max: u64,
    burst_mode: bool,
    max_arrival_gap_ms: u64,
}

// Threadpool

enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnOnce(usize) + Send + 'static>;

struct ThreadPool {

    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    queued_jobs: Arc<AtomicUsize>,
}

impl ThreadPool{
    fn new(size: usize) -> Threadpool {
        assert!(size > 0);


        let (sender, receiver) = mpsc::channel::<Message>();
        let receiver = Arc::new(Mutex::new(receiver));
        let queued_jobs = Arc::new(AtomicUsize::new(0));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                Arc::clone(&queued_jobs),

            ));
        }

        ThreadPool {
            workers,
            sender,
            queued_jobs
        }
    }
    
    fn execute<F>(&self, f: F)
    where
        F: FnOnce(usize) + Send + 'static, 
        {
            let job = Box::new(f);
            self.queued_jobs.fetch_add(1, Ordering::SeqCst);
            self.sender.send(Message::NewJob(job)).unwrap();
        }

        fn size(&self) -> usize {
            self.workers.len()
        }
    }

    impl Drop for ThreadPool {

        fn drop(&mut self) {
            for _ in &self.workers {
                self.sender.send(Message::Terminate).unwrap();
            }

            for worker in &mut self.workers {
                println!("Shutting down worker {}", worker.id);

                if let Some(thread) = worker.thread.take(){
                    thread.join().unwrap();
                }
            }
        }
    }

    struct Worker {
        id:usize,
        hread: Option<thread::JoinHandle<()>>,
    }

    impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        queued_jobs: Arc<AtomicUsize>,
    ) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    queued_jobs.fetch_sub(1, Ordering::SeqCst);
                    job(id);
                }
                Message::Terminate => {
                    println!("Worker {} is terminating.", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// Simulated Behavior

fn simulate_cpu_work(duration_ms: u64) {
    let start = Instant::now();
    let mut counter: u64 = 0;

    while start.elapsed().as_millis() < duration_ms as u128 {
        counter += 1;
    }

    if counter == 0 {
        panic!("Counter should not be at 0");
    }

}
// Task Gen
fn generate_tasks(cfg: WorkloadConfig, tx: mpsc::Sender<Task>) {
    let mut rng = StdRng::seed_from_u64(cfg.seed);

    for i in 0 ..cfg.num_tasks {
        let kind = if rng.r#gen::<f64>() < cfg.cpu_fraction {
            TaskKind::Cpu
        } else {
            TaskKind::Io
        };

        let duration_ms = match kind {
            TaskKind::Cpu => rng.gen_range(cfg.cpu_dur_min..= cfg.cpu_dur_max),
            TaskKind::Io => rng.gen_range(cfg.io_dur_min..=cfg.io_dur_max),
        };

        let task = Task::new(i, kind, duration_ms, Instant::now());

        if tx.send(task).is_err() {
            break;
        }

        let gap = if cfg.burst_mode {
            if i % 20 == 19 {
                rng.gen_range(20..=cfg.max_arrival_gap_ms)
            } else {
                rng.gen_range(0..=2)
            }
        } else {
            rng.gen_range(0..=cfg.max_arrival_gap_ms)
        };

        if gap > 0 {
            thread::sleep(Duration::from_millis(gap));
        }
     }
    
}

// Dispatcher

// Metrics and Printing

// Running one Experiment 



fn main() {
    println!("Hello, world!");

// Experiment A



// Experiment B

}
