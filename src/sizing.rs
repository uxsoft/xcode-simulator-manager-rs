use std::process::Command;
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::simctl::Udid;

pub struct SizeUpdate {
    pub udid: Udid,
    pub bytes: u64,
}

const WORKERS: usize = 6;

pub fn spawn_size_scan(jobs: Vec<(Udid, String)>) -> Receiver<SizeUpdate> {
    let (result_tx, result_rx) = channel::<SizeUpdate>();
    let (job_tx, job_rx) = channel::<(Udid, String)>();
    for job in jobs {
        let _ = job_tx.send(job);
    }
    drop(job_tx);

    let job_rx = Arc::new(Mutex::new(job_rx));
    for _ in 0..WORKERS {
        let result_tx = result_tx.clone();
        let job_rx = Arc::clone(&job_rx);
        thread::spawn(move || {
            loop {
                let job = {
                    let guard = job_rx.lock().unwrap();
                    guard.recv()
                };
                let Ok((udid, path)) = job else { break };
                if let Some(bytes) = du_bytes(&path) {
                    let _ = result_tx.send(SizeUpdate { udid, bytes });
                }
            }
        });
    }
    result_rx
}

fn du_bytes(path: &str) -> Option<u64> {
    let output = Command::new("du").args(["-sk", path]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let kb: u64 = stdout.split_whitespace().next()?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}
