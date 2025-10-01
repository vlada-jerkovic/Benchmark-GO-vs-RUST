use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::Instant; // removed Duration


fn parse_arg(name: &str, default: u64) -> u64 {
    let mut it = env::args().skip(1);
    let mut val: Option<u64> = None;
    while let Some(k) = it.next() {
        if k == format!("--{name}") {
            if let Some(v) = it.next() {
                val = v.parse::<u64>().ok();
            }
        }
    }
    val.unwrap_or(default)
}

fn parse_str(name: &str, default: &str) -> String {
    let mut it = env::args().skip(1);
    let mut val: Option<String> = None;
    while let Some(k) = it.next() {
        if k == format!("--{name}") {
            if let Some(v) = it.next() {
                val = Some(v);
            }
        }
    }
    val.unwrap_or_else(|| default.to_string())
}

// SplitMix64 step: fast 64-bit mixing function
#[inline(always)]
fn splitmix64_step(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn worker(id: usize, workers: usize, total: u64, inner: u64) -> u64 {
    // stride over the iteration space
    let mut acc: u64 = 0;
    let mut i = id as u64;
    while i < total {
        let mut x = i;
        // inner loop adds CPU work per iteration
        for _ in 0..inner {
            x = splitmix64_step(x);
        }
        acc ^= x; // stop optimizer eliminating work
        i += workers as u64;
    }
    acc
}

fn main() {
    // Parameters
    // total: number of outer iterations
    // inner: SplitMix64 steps per iteration
    // mode:  "single" | "multi"
    // workers: thread count for multi mode (ignored for single)
    let total = parse_arg("total", 10_000_000);
    let inner = parse_arg("inner", 5);
    let mode = parse_str("mode", "single"); // single | multi
    let workers_flag = parse_arg("workers", num_cpus::get() as u64) as usize;
    let csv = parse_str("csv", "true"); // "true" prints a CSV line too

    let (workers, logical_mode) = if mode == "single" {
        (1usize, "single")
    } else {
        (if workers_flag == 0 { 1 } else { workers_flag }, "multi")
    };

    let start = Instant::now();

    let (tx, rx) = mpsc::channel::<u64>();
    for w in 0..workers {
        let txc = tx.clone();
        let total_c = total;
        let inner_c = inner;
        thread::spawn(move || {
            let res = worker(w, workers, total_c, inner_c);
            txc.send(res).ok();
        });
    }
    drop(tx);

    // Combine results to prevent dead-code elimination
    let mut combined: u64 = 0;
    for _ in 0..workers {
        if let Ok(v) = rx.recv() {
            combined ^= v;
        }
    }

    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();
    let iters = total as f64;
    let ips = iters / secs;

    // Human summary
    println!(
        "[RUST] mode={logical_mode} workers={workers} total={} inner={} elapsed_ms={:.3} ips={:.0} checksum=0x{:016X}",
        total, inner, secs * 1000.0, ips, combined
    );

    if csv == "true" {
        println!(
            "lang,mode,workers,total,inner,elapsed_ms,iterations_per_sec,checksum\nrust,{},{},{},{},{:.3},{:.0},0x{:016X}",
            logical_mode, workers, total, inner, secs * 1000.0, ips, combined
        );
    }
}