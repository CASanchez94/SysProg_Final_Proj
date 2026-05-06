use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::{Arc, Mutex, mpsc};
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
    arrival_time: Instant,
    #[allow(dead_code)]
    dispatch_time: Option<Instant>,
}
 
impl Task {
    fn new(id: u64, kind: TaskKind) -> Self {
        Task {
            id,
            kind,
            arrival_time: Instant::now(),
            dispatch_time: None,
        }
    }
 
    
    fn cpu_cost(&self) -> f64 {
        match self.kind {
            TaskKind::Cpu => 35.0,
            TaskKind::Io  => 10.0,
        }
    }
}
 
 
// Record of Completion
 
#[derive(Debug)]
struct CompletionRecord {
    #[allow(dead_code)]
    id: u64,
    kind: TaskKind,
    wait_ms: u64,
    turnaround_ms: u64,
    #[allow(dead_code)]
    worker_id: usize,
}
 
 
// Workload Config
 
#[derive(Debug, Clone, Copy)]
struct WorkloadConfig {
    num_tasks: u64,
    seed: u64,
    io_fraction: f64,
    arrival_gap_ms: u64,   
}
 
 
// Shared State 
 
struct SharedState {
    active_workers: usize,
    cpu_pct: f64,       
    queue_len: usize,   
}
 
impl SharedState {
    fn new() -> Self {
        SharedState { active_workers: 0, cpu_pct: 0.0, queue_len: 0 }
    }
}
 
 
// Workers
 
fn worker_thread(
    id: usize,
    task_rx: Arc<Mutex<mpsc::Receiver<Task>>>,
    done_tx: mpsc::Sender<CompletionRecord>,
    state: Arc<Mutex<SharedState>>,
    release_tx: mpsc::Sender<()>,
) {
    loop {
        let msg = {
            let rx = task_rx.lock().unwrap();
            rx.recv()
        };
 
        match msg {
            Err(_) => break, 
            Ok(task) => {
                let exec_start = Instant::now();
                let wait_ms = exec_start.duration_since(task.arrival_time).as_millis() as u64;
 
              
                match task.kind {
                    TaskKind::Io  => thread::sleep(Duration::from_millis(200)),
                    TaskKind::Cpu => simulate_cpu_work(200),
                }
 
                let turnaround_ms = Instant::now()
                    .duration_since(task.arrival_time)
                    .as_millis() as u64;
 
               
                {
                    let mut s = state.lock().unwrap();
                    s.active_workers -= 1;
                    s.cpu_pct -= task.cpu_cost();
                    if s.cpu_pct < 0.0 { s.cpu_pct = 0.0; }
                }
 
                let _ = release_tx.send(());
 
                let _ = done_tx.send(CompletionRecord {
                    id: task.id,
                    kind: task.kind,
                    wait_ms,
                    turnaround_ms,
                    worker_id: id,
                });
            }
        }
    }
}
 
 
// Simulated Behavior
 
fn simulate_cpu_work(duration_ms: u64) {
    let start = Instant::now();
    let mut counter: u64 = 0;
    while start.elapsed().as_millis() < duration_ms as u128 {
        counter = counter.wrapping_add(1);
    }
    let _ = counter;
}
 
 
// Task Generator
 
fn generate_tasks(cfg: WorkloadConfig, tx: mpsc::Sender<Task>) {
    let mut rng = StdRng::seed_from_u64(cfg.seed);
 
    for i in 0..cfg.num_tasks {
        let kind = if rng.r#gen::<f64>() < cfg.io_fraction {
            TaskKind::Io
        } else {
            TaskKind::Cpu
        };
 
        let task = Task::new(i, kind);
 
        if tx.send(task).is_err() {
            break;
        }
 
        thread::sleep(Duration::from_millis(cfg.arrival_gap_ms));
    }
}
 
 

 
fn run_manager_fifo(
    task_rx:     mpsc::Receiver<Task>,
    worker_tx:   mpsc::SyncSender<Task>,
    state: Arc<Mutex<SharedState>>,
    release_rx: mpsc::Receiver<()>,
    num_workers: usize,
    submitted: Arc<Mutex<u64>>,
) {
    let mut waiting: Vec<Task> = Vec::new();
    let mut gen_done = false;
 
    loop {
       
        loop {
            match task_rx.try_recv() {
                Ok(t) => {
                    state.lock().unwrap().queue_len += 1;
                    waiting.push(t);
                }
                Err(mpsc::TryRecvError::Disconnected) => { gen_done = true; break; }
                Err(mpsc::TryRecvError::Empty) => break,
            }
        }
 
       
        let mut sent_one = false;
        while !waiting.is_empty() {
            let cpu_needed = waiting[0].cpu_cost();
            let can_go = {
                let s = state.lock().unwrap();
                s.active_workers < num_workers && s.cpu_pct + cpu_needed <= 100.0
            };
            if !can_go { break; }
 
            let task = waiting.remove(0);
            {
                let mut s = state.lock().unwrap();
                s.active_workers += 1;
                s.cpu_pct += task.cpu_cost();
                s.queue_len -= 1;
            }
            *submitted.lock().unwrap() += 1;
            let _ = worker_tx.send(task);
            sent_one = true;
        }
 
        if gen_done && waiting.is_empty() { break; }
        if !sent_one {
          
            let _ = release_rx.recv_timeout(Duration::from_millis(5));
        }
    }
}
 

 
fn run_manager_optimized(
    task_rx:     mpsc::Receiver<Task>,
    worker_tx:   mpsc::SyncSender<Task>,
    state: Arc<Mutex<SharedState>>,
    release_rx: mpsc::Receiver<()>,
    num_workers: usize,
    submitted: Arc<Mutex<u64>>,
) {
    let mut io_queue:  Vec<Task> = Vec::new();
    let mut cpu_queue: Vec<Task> = Vec::new();
    let mut gen_done = false;
 
    loop {
        loop {
            match task_rx.try_recv() {
                Ok(t) => {
                    state.lock().unwrap().queue_len += 1;
                    match t.kind {
                        TaskKind::Io  => io_queue.push(t),
                        TaskKind::Cpu => cpu_queue.push(t),
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => { gen_done = true; break; }
                Err(mpsc::TryRecvError::Empty) => break,
            }
        }
 
        let mut sent_one = false;
        loop {
            let (free_workers, free_cpu) = {
                let s = state.lock().unwrap();
                (num_workers - s.active_workers, 100.0 - s.cpu_pct)
            };
            if free_workers == 0 || free_cpu < 10.0 { break; }
 
            
            let task = if !cpu_queue.is_empty() && free_cpu >= 35.0 {
                Some(cpu_queue.remove(0))
            } else if !io_queue.is_empty() && free_cpu >= 10.0 {
                Some(io_queue.remove(0))
            } else {
                None
            };
 
            match task {
                None => break,
                Some(t) => {
                    {
                        let mut s = state.lock().unwrap();
                        s.active_workers += 1;
                        s.cpu_pct += t.cpu_cost();
                        s.queue_len -= 1;
                    }
                    *submitted.lock().unwrap() += 1;
                    let _ = worker_tx.send(t);
                    sent_one = true;
                }
            }
        }
 
        if gen_done && io_queue.is_empty() && cpu_queue.is_empty() { break; }
        if !sent_one {
            let _ = release_rx.recv_timeout(Duration::from_millis(5));
        }
    }
}
 
 
// Monitor Thread
 
fn run_monitor(
    state: Arc<Mutex<SharedState>>,
    stop: Arc<Mutex<bool>>,
    wall_start: Instant,
) -> Vec<(u64, usize, f64)> {
    let mut log: Vec<(u64, usize, f64)> = Vec::new(); // (time_ms, active_workers, cpu_pct)
    loop {
        thread::sleep(Duration::from_millis(10));
        let done = *stop.lock().unwrap();
        let s = state.lock().unwrap();
        log.push((wall_start.elapsed().as_millis() as u64, s.active_workers, s.cpu_pct));
        if done { break; }
    }
    log
}
 
 
// Metrics and Printing
 
fn print_summary(
    label: &str,
    completions: &[CompletionRecord],
    log: &[(u64, usize, f64)],
    makespan_ms: u64

) {
    let total = completions.len();
    let mut io_count  = 0usize;
    let mut cpu_count = 0usize;
 
    for r in completions {
        match r.kind {
            TaskKind::Io  => io_count  += 1,
            TaskKind::Cpu => cpu_count += 1,
        }
    }
 
    let avg_wait = if total > 0 {
        completions.iter().map(|r| r.wait_ms).sum::<u64>() / total as u64
    } else { 0 };
 
    let avg_turn = if total > 0 {
        completions.iter().map(|r| r.turnaround_ms).sum::<u64>() / total as u64
    } else { 0 };
 
    let avg_cpu  = log.iter().map(|e| e.2).sum::<f64>() / log.len().max(1) as f64;
    let peak_cpu = log.iter().map(|e| e.2 as u64).max().unwrap_or(0);
    let avg_active = log.iter().map(|e| e.1 as f64).sum::<f64>() / log.len().max(1) as f64;

 
    println!();
    println!("---------------------------------------------------------");
    println!(" RESULTS: {}", label);
    println!("---------------------------------------------------------");
    println!("  Total tasks completed  : {}", total);
    println!("  CPU tasks completed    : {}", cpu_count);
    println!("  IO tasks completed     : {}", io_count);
    println!("  Makespan               : {} ms", makespan_ms);
    println!("  Avg wait time          : {} ms", avg_wait);
    println!("  Avg turnaround time    : {} ms", avg_turn);
    println!("---------------------------------------------------------");
    println!("  Monitor (sampled every 10ms):");
    println!("  Avg CPU usage          : {:.1}%", avg_cpu);
    println!("  Peak CPU usage         : {}%", peak_cpu);
    println!("  Avg active workers     : {:.1} / 8", avg_active);
    println!("---------------------------------------------------------");
 
  
}
 
 
// Running one Experiment
 
fn run_experiment(label: &str, cfg: WorkloadConfig, num_workers: usize, optimized: bool) {
    println!();
    println!("Starting: {}", label);
    println!(
        "   Tasks: {}  |  Workers: {}  |  IO fraction: {:.0}%  |  Scheduler: {}",
        cfg.num_tasks,
        num_workers,
        cfg.io_fraction * 100.0,
        if optimized { "Optimized" } else { "FIFO" }
    );
 
    let state = Arc::new(Mutex::new(SharedState::new()));
    let stop  = Arc::new(Mutex::new(false));
    let submitted: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
 
    let (task_tx,    task_rx)    = mpsc::channel::<Task>();
    let (done_tx,    done_rx)    = mpsc::channel::<CompletionRecord>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let (worker_tx,  worker_rx)  = mpsc::sync_channel::<Task>(32);
 
    let worker_rx = Arc::new(Mutex::new(worker_rx));
 
    for id in 0..num_workers {
        let rx  = Arc::clone(&worker_rx);
        let dtx = done_tx.clone();
        let st  = Arc::clone(&state);
        let rel = release_tx.clone();
        thread::spawn(move || worker_thread(id, rx, dtx, st, rel));
    }
    drop(done_tx); 
 
  
    let wall_start = Instant::now();
    let mon_handle = {
        let st   = Arc::clone(&state);
        let stop = Arc::clone(&stop);
        thread::spawn(move || run_monitor(st, stop, wall_start))
    };
 
    
    let generator_handle = thread::spawn(move || generate_tasks(cfg, task_tx));
 
    
    if optimized {
        run_manager_optimized(task_rx, worker_tx, Arc::clone(&state), release_rx, num_workers, Arc::clone(&submitted));
    } else {
        run_manager_fifo(task_rx, worker_tx, Arc::clone(&state), release_rx, num_workers, Arc::clone(&submitted));
    }
 
    generator_handle.join().unwrap();
 
    
    let expected = *submitted.lock().unwrap();
    let mut completions: Vec<CompletionRecord> = Vec::new();
    for _ in 0..expected {
        match done_rx.recv_timeout(Duration::from_secs(120)) {
            Ok(r)  => completions.push(r),
            Err(_) => { eprintln!("timed out waiting for a worker"); break; }
        }
    }
 
    let makespan_ms = wall_start.elapsed().as_millis() as u64;
 
 
    *stop.lock().unwrap() = true;
    let log = mon_handle.join().unwrap();
 
    print_summary(label, &completions, &log, makespan_ms);
}
 
 
fn main() {

    // Experiment A - FIFO, 70/30 IO/CPU
    run_experiment(
        "A: FIFO, 70% IO / 30% CPU",
        WorkloadConfig {
            num_tasks: 1000,
            seed: 42,
            io_fraction: 0.70,
            arrival_gap_ms: 20,
        },
        8,
        false,
    );

        // Experiment B - Optimized, 70/30 IO/CPU
    run_experiment(
        ": Optimized, 70% IO / 30% CPU",
        WorkloadConfig {
            num_tasks: 1000,
            seed: 42,
            io_fraction: 0.70,
            arrival_gap_ms: 20,
        },
        8,
        true,
    );

    println!();
    println!("All experiments complete.");
}
