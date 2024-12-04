// import modules
use std::fs;

use procfs::process::*;
use crate::process::data::fs::File;
//use std::fs::File;
use std::io::{self, BufRead};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use chrono::DateTime;
use chrono::Local;
use std::collections::HashSet;
use nix::unistd::Pid;
use nix::sys::signal::{kill, Signal};


pub const PAGE_SIZE: u64 = 4096;

#[derive(Clone)]
pub struct ProcessUsage {
    pub pid: i32,
    pub ppid: i32,
    pub name: String,
    pub cpu_usage: f64,
    pub virtual_memory_usage: f64,  // Virtual memory usage percentage
    pub resident_memory_usage: f64, // Resident memory usage percentage
    pub state: String,
    pub start_time: String,
    pub priority: String,
    pub num_threads: i64, // Number of threads
    pub voluntary_ctxt_switches: u64,
    pub nonvoluntary_ctxt_switches: u64,
}

/// Function to kill a process by its PID.
pub fn kill_process(pid: i32) -> Result<(), String> {
    let process_pid = Pid::from_raw(pid);

    match kill(process_pid, Signal::SIGKILL) {
        Ok(_) => {
            println!("Successfully killed process with PID: {}", pid);
            Ok(())
        }
        Err(err) => {
            eprintln!("Failed to kill process with PID: {}. Error: {:?}", pid, err);
            Err(format!("Error killing process: {:?}", err))
        }
    }
}
// filters a list of processes based on their state.
pub fn filter_process_info(processes: &[ProcessUsage], filter_by_states: &HashSet<char>) -> Vec<ProcessUsage> {
    processes
        .iter()
        .filter(|process| filter_by_states.contains(&process.state.chars().next().unwrap_or_default()))
        .cloned()
        .collect()
}

pub fn convert_state(state: char) -> String {
    match state {
        'R' => "Running".to_string(),
        'S' => "Sleeping".to_string(),
        'D' => "Disk Sleep".to_string(),
        'Z' => "Zombie".to_string(),
        'T' => "Stopped".to_string(),
        'I' => "Idle".to_string(),
        _ => "Unknown".to_string(),
    }
}

pub fn convert_priority(priority: i64) -> String {
    match priority {
        p if p <= 0 => "High".to_string(),
        p if p <= 20 => "Normal".to_string(),
        _ => "Low".to_string(),
    }
}

fn format_start_time(start_time_ticks: u64) -> String {
    // Assuming each tick is 1/100th of a second (adjust if needed for your system)
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::new(0, 0))
        .as_secs();

    let start_time_secs = uptime - start_time_ticks / 100;
    let start_time = UNIX_EPOCH + Duration::from_secs(start_time_secs);

    // Convert SystemTime to DateTime and format it
    let datetime: DateTime<Local> = start_time.into();
    datetime.format("%H:%M:%S").to_string()
}

fn calculate_virtual_memory_usage_percentage(process_memory: u64, total_used_memory: u64) -> f64 {
    if total_used_memory == 0 {
        0.0
    } else {
        (process_memory as f64 / total_used_memory as f64) * 100.0
    }
}

fn calculate_resident_memory_usage_percentage(process_memory: u64, total_used_memory: u64) -> f64 {
    if total_used_memory == 0 {
        0.0
    } else {
        (process_memory as f64 / total_used_memory as f64) * 100.0
    }
}

fn calculate_cpu_usage_percentage(process_cpu_ticks: u64, total_cpu_ticks: u64) -> f64 {
    if total_cpu_ticks == 0 {
        0.0
    } else {
        (process_cpu_ticks as f64 / total_cpu_ticks as f64) * 100.0
    }
}

fn get_total_cpu_ticks() -> u64 {
    if let Ok(file) = File::open("/proc/stat") {
        let reader = io::BufReader::new(file);
        if let Some(Ok(line)) = reader.lines().next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts[0] == "cpu" {
                return parts.iter().skip(1).filter_map(|v| v.parse::<u64>().ok()).sum();
            }
        }
    }
    0
}

pub fn parse_status_file(pid: u32) -> io::Result<(u64, u64)> {
    let status_path = format!("/proc/{}/status", pid);
    let file = fs::File::open(&status_path)?;
    let reader = io::BufReader::new(file);

    let mut voluntary_ctxt_switches = 0;
    let mut nonvoluntary_ctxt_switches = 0;

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("voluntary_ctxt_switches:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                voluntary_ctxt_switches = parts[1].parse::<u64>().unwrap_or(0);
            }
        } else if line.starts_with("nonvoluntary_ctxt_switches:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                nonvoluntary_ctxt_switches = parts[1].parse::<u64>().unwrap_or(0);
            }
        }
    }

    Ok((voluntary_ctxt_switches, nonvoluntary_ctxt_switches))
}

pub fn get_processes() -> Vec<ProcessUsage> {
    let mut processes = Vec::new();
    let mut total_virtual_used_memory: u64 = 0;
    let mut total_resident_used_memory: u64 = 0;
    let total_cpu_ticks = get_total_cpu_ticks();

    for process_result in all_processes().unwrap() {
        if let Ok(process) = process_result {
            if let Ok(stat) = process.stat() {
                let virtual_memory_usage = stat.vsize / 1024; // KB
                let resident_memory_usage = (stat.rss * PAGE_SIZE) / 1024; // KB

                total_virtual_used_memory += virtual_memory_usage;
                total_resident_used_memory += resident_memory_usage;

                let (voluntary_ctxt_switches, nonvoluntary_ctxt_switches) = parse_status_file(stat.pid as u32).unwrap_or((0, 0));

                processes.push(ProcessUsage {
    pid: stat.pid,
    ppid: stat.ppid,
    name: stat.comm.clone(),
    cpu_usage: calculate_cpu_usage_percentage((stat.utime + stat.stime) as u64, total_cpu_ticks),
    virtual_memory_usage: virtual_memory_usage as f64,
    resident_memory_usage: resident_memory_usage as f64,
    state: convert_state(stat.state), // Use converted state
    start_time: format_start_time(stat.starttime),
    priority: convert_priority(stat.priority), // Use converted priority
    num_threads: stat.num_threads,
    voluntary_ctxt_switches,
    nonvoluntary_ctxt_switches,
});

            }
        }
    }

    for process in &mut processes {
        process.virtual_memory_usage = calculate_virtual_memory_usage_percentage(
            process.virtual_memory_usage as u64,
            total_virtual_used_memory,
        );
        process.resident_memory_usage = calculate_resident_memory_usage_percentage(
            process.resident_memory_usage as u64,
            total_resident_used_memory,
        );
    }

    processes
}
