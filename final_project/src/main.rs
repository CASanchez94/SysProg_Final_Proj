use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::{atomic::{AtomicU64, AtomicUsize, Ordering}, mpsc, Arc, Mutex,};
use std::thread;
use std::time::{Duration, Instant};


// The Task Model

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskKind {
    Cpu,
    Io,
}

#[derive(Debug, Clone)]
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
    duration_ms: u64,
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
    sender: mpsc::SyncSender<Message>, 
    queued_jobs: Arc<AtomicUsize>,
    active_workers: Arc<AtomicUsize>,
}

impl ThreadPool{
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0);


        let (sender, receiver) = mpsc::sync_channel(100);
        let receiver = Arc::new(Mutex::new(receiver));
        let queued_jobs = Arc::new(AtomicUsize::new(0));
        let active_workers = Arc::new(AtomicUsize::new(0));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                Arc::clone(&queued_jobs),
                Arc::clone(&active_workers),

            ));
        }

        ThreadPool {
            workers,
            sender,
            queued_jobs,
            active_workers,
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

                if let Some(thread) = worker.thread.take(){
                    thread.join().unwrap();
                }
            }
        }
    }

    struct Worker {
        #[allow(dead_code)]
        id:usize,
        thread: Option<thread::JoinHandle<()>>,
    }

    impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        queued_jobs: Arc<AtomicUsize>,
        active_workers: Arc<AtomicUsize>,
    ) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    queued_jobs.fetch_sub(1, Ordering::SeqCst);
                    active_workers.fetch_add(1, Ordering::SeqCst);
                    job(id);
                    active_workers.fetch_sub(1, Ordering::SeqCst);
                    
                }
                Message::Terminate => {
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
fn run_dispatcher(
    task_rx: mpsc::Receiver<Task>,
    pool: Arc<ThreadPool>,
    done_tx: mpsc::Sender<CompletionRecord>,
    submitted_count: Arc<AtomicU64>,
) {
    for mut task in task_rx {
    task.dispatch_time = Some(Instant::now());
    submitted_count.fetch_add(1, Ordering::SeqCst);

    let done_tx = done_tx.clone();

    pool.execute(move |worker_id| {
        let start = Instant::now();
        let dispatch_time = task.dispatch_time.unwrap_or(task.arrival_time);
        let wait_ms = start.duration_since(dispatch_time).as_millis() as u64;

        match task.kind {
            TaskKind::Cpu => simulate_cpu_work(task.duration_ms),
            TaskKind::Io => thread::sleep(Duration::from_millis(task.duration_ms)),
        }

        let finish = Instant::now();
        let turnaround_ms = finish.duration_since(task.arrival_time).as_millis() as u64;

        let rec = CompletionRecord {
            id: task.id,
            wait_ms,
            turnaround_ms,
            duration_ms: task.duration_ms,
            worker_id,
        };

        let _ = done_tx.send(rec);
    });
}
}
// Metrics and Printing
 
fn print_summary(
    label: &str,
    completions: &[CompletionRecord],
    worker_busy_ms: &[u64],
    makespan_ms: u64,
    last_finished_task_id: Option<u64>,
    peak_available: usize,
    num_workers: usize,
) {
    let total = completions.len();
 
    let avg_wait = if total > 0 {
        completions.iter().map(|r| r.wait_ms).sum::<u64>() / total as u64
    } else {
        0
    };
 
    let avg_turn = if total > 0 {
        completions
            .iter()
            .map(|r| r.turnaround_ms)
            .sum::<u64>()
            / total as u64
    } else {
        0
    };
 
    let max_wait = completions.iter().map(|r| r.wait_ms).max().unwrap_or(0);
 
    println!();
    println!("---------------------------------------------------------");
    println!(" RESULTS: {:<46}|", label);
    println!("---------------------------------------------------------");
    println!("  Total tasks completed : {:>6}                       ", total);
    println!(
        "  Last task finished    : {:>6}                       ",
        last_finished_task_id.unwrap_or(0)
    );
    println!("|  Makespan              : {:>6} ms                     |", makespan_ms);
    println!("|  Avg wait time         : {:>6} ms                     |", avg_wait);
    println!("|  Avg turnaround time   : {:>6} ms                     |", avg_turn);
    println!("|  Max wait time         : {:>6} ms                     |", max_wait);
    println!("|  Min available workers : {:>6} / {}                   |", peak_available, num_workers);
    println!("|  Worker utilization:                                   |");
 
    for i in 0..num_workers {
        let pct = if makespan_ms > 0 {
            worker_busy_ms[i] * 100 / makespan_ms
        } else {
            0
        };
 
        println!(
            "|    Worker {:>2}  : {:>3}%                                   |",
            i, pct
        );
    }
 
    println!("---------------------------------------------------------");
}
 
// Running one Experiment
 
fn run_experiment(label: &str, cfg: WorkloadConfig, num_workers: usize) {
    println!();
    println!("▶  Starting: {}", label);
    println!(
        "   Tasks: {}  |  Workers: {}  |  CPU fraction: {:.0}%  |  Burst: {}",
        cfg.num_tasks,
        num_workers,
        cfg.cpu_fraction * 100.0,
        cfg.burst_mode
    );
 
    let pool = Arc::new(ThreadPool::new(num_workers));
 
    let (task_tx, task_rx) = mpsc::channel::<Task>();
    let (done_tx, done_rx) = mpsc::channel::<CompletionRecord>();
 
    let submitted_count = Arc::new(AtomicU64::new(0));
 
    let active_workers_ref = Arc::clone(&pool.active_workers);
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let monitor_handle = thread::spawn(move || {
        let mut min_available = num_workers;
        loop {
            let active = active_workers_ref.load(Ordering::SeqCst);
            let available = num_workers.saturating_sub(active);
            if available < min_available {
                min_available = available;
            }
            if stop_rx.try_recv().is_ok() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        min_available
    });

    let dispatcher_handle = {
        let pool = Arc::clone(&pool);
        let done_tx = done_tx.clone();
        let submitted = Arc::clone(&submitted_count);
        thread::spawn(move || run_dispatcher(task_rx, pool, done_tx, submitted))
    };
 
    let generator_handle = thread::spawn(move || generate_tasks(cfg, task_tx));
 
    let wall_start = Instant::now();
 
    generator_handle.join().unwrap();
    dispatcher_handle.join().unwrap();

    let _ = stop_tx.send(());
    let peak_available = monitor_handle.join().unwrap();

 
    drop(done_tx);
 
    let expected = submitted_count.load(Ordering::SeqCst);
    let mut completions = Vec::with_capacity(expected as usize);
    let mut worker_busy_ms = vec![0u64; pool.size()];
    let mut last_finished_task_id = None;
 
    for _ in 0..expected {
        let rec = done_rx.recv().unwrap();
        worker_busy_ms[rec.worker_id] += rec.duration_ms;
        last_finished_task_id = Some(rec.id);
        completions.push(rec);
    }
 
    let makespan_ms = wall_start.elapsed().as_millis() as u64;
 
    print_summary(
        label,
        &completions,
        &worker_busy_ms,
        makespan_ms,
        last_finished_task_id,
        peak_available,
        num_workers,
    );
}
 
 
fn main() {
    println!("--------------------------------------------------------");
    println!("  Final Project : Concurrent Task Dispatcher");
    println!("  Architecture  : Central Dispatcher");
    println!("  Policy        : FIFO - Simple, Fair, Starvation Free");
    println!("  Workers       : 8");
    println!("--------------------------------------------------------");
 
    // Experiment A
    run_experiment(
        "A: Balanced Workload",
        WorkloadConfig {
            num_tasks: 500,
            seed: 42,
            cpu_fraction: 0.50,
            cpu_dur_min: 5,
            cpu_dur_max: 30,
            io_dur_min: 10,
            io_dur_max: 40,
            burst_mode: false,
            max_arrival_gap_ms: 3,
        },
        8,
    );
 
    // Experiment B
    run_experiment(
        "B: Stressed Workload",
        WorkloadConfig {
            num_tasks: 600,
            seed: 99,
            cpu_fraction: 0.80,
            cpu_dur_min: 20,
            cpu_dur_max: 80,
            io_dur_min: 2,
            io_dur_max: 10,
            burst_mode: true,
            max_arrival_gap_ms: 20,
        },
        8,
    );
 
    println!("All experiments complete.");
}
 
