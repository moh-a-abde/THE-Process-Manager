// import modules
use procfs::process::*;
use std::fs::File;
use std::io::{self, BufRead};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use chrono::DateTime;
use chrono::Local;
use std::collections::HashSet;

#[derive(Clone)]
pub struct ProcessUsage {
    pub pid: i32,
    pub ppid: i32,           // parent PID
    pub name: String,
    pub cpu_usage: f64,      // stores CPU percentage as f64
    pub memory_usage: f64,   // stores memory percentage as f64
    pub state: String,       // process state
    pub start_time: String,    
    pub priority: String,       
}

// filters a list of processes based on their state.
pub fn filter_process_info(processes: &[ProcessUsage], filter_by_states: &HashSet<char>) -> Vec<ProcessUsage> {
    processes
        .iter()
        .filter(|process| filter_by_states.contains(&process.state.chars().next().unwrap_or_default()))
        .cloned()
        .collect()
}

fn convert_state(state: char) -> String {
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

fn convert_priority(priority: i64) -> String {
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

fn calculate_cpu_usage_percentage(process_cpu_ticks: u64, total_cpu_ticks: u64) -> f64 {
    if total_cpu_ticks == 0 {
        0.0
    } else {
        (process_cpu_ticks as f64 / total_cpu_ticks as f64) * 100.0
    }
}

fn calculate_memory_usage_percentage(process_memory: u64, total_used_memory: u64) -> f64 {
    if total_used_memory == 0 {
        0.0
    } else {
        (process_memory as f64 / total_used_memory as f64) * 100.0
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

pub fn get_processes() -> Vec<ProcessUsage> {
    let mut processes = Vec::new();
    let mut total_used_memory: u64 = 0;

    for process_result in all_processes().unwrap() {
        if let Ok(process) = process_result {
            if let Ok(stat) = process.stat() {
                let cpu_usage = stat.utime + stat.stime;
                let memory_usage = stat.vsize / 1024;  // Convert to KB
                total_used_memory += memory_usage;
                
                // Convert fields to meaningful values
                let ppid = stat.ppid;
                let state = convert_state(stat.state);
                let start_time = format_start_time(stat.starttime);
                let priority = convert_priority(stat.priority);

                processes.push(ProcessUsage {
                    pid: stat.pid,
                    ppid,
                    name: stat.comm.clone(),
                    cpu_usage: cpu_usage as f64,  // Temporarily store raw CPU ticks
                    memory_usage: memory_usage as f64,  // Temporarily store raw memory usage
                    state,
                    start_time,
                    priority,
                });
            }
        }
    }

    let total_cpu_ticks = get_total_cpu_ticks();

    // Calculate CPU and memory percentages
    for process in &mut processes {
        process.cpu_usage = calculate_cpu_usage_percentage(process.cpu_usage as u64, total_cpu_ticks);
        process.memory_usage = calculate_memory_usage_percentage(process.memory_usage as u64, total_used_memory);
    }

    processes
}

