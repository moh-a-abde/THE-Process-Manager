use procfs::process::Process;
use crate::process::data::{PAGE_SIZE, parse_status_file, ProcessUsage};

pub fn get_process_info(pid: i32) -> String {
    match Process::new(pid) {
        Ok(process) => {
            match process.stat() {
                Ok(stat) => {
                    let (voluntary_ctxt_switches, nonvoluntary_ctxt_switches) =
                        parse_status_file(stat.pid as u32).unwrap_or((0, 0));

                    // Construct a ProcessUsage instance for better abstraction
                    let process_usage = ProcessUsage {
                        pid: stat.pid,
                        ppid: stat.ppid,
                        name: stat.comm.clone(),
                        cpu_usage: (stat.utime + stat.stime) as f64,
                        virtual_memory_usage: (stat.vsize / 1024) as f64, // KB
                        resident_memory_usage: ((stat.rss * PAGE_SIZE) / 1024) as f64, // KB
                        state: stat.state.to_string(),
                        start_time: "N/A".to_string(), // Update with actual start time if required
                        priority: stat.priority.to_string(),
                        num_threads: stat.num_threads,
                        voluntary_ctxt_switches,
                        nonvoluntary_ctxt_switches,
                    };

                    format!(
                        "PID: {}\nCommand: {}\nState: {}\nCPU Usage: {} ticks\nVirtual Memory: {} KB\nResident Memory: {} KB\nThreads: {}\nVoluntary Context Switches: {}\nNonvoluntary Context Switches: {}",
                        process_usage.pid,
                        process_usage.name,
                        process_usage.state,
                        process_usage.cpu_usage,
                        process_usage.virtual_memory_usage,
                        process_usage.resident_memory_usage,
                        process_usage.num_threads,
                        process_usage.voluntary_ctxt_switches,
                        process_usage.nonvoluntary_ctxt_switches,
                    )
                }
                Err(e) => format!("Failed to get stat for process {}: {:?}", pid, e),
            }
        }
        Err(e) => format!("Failed to find process with PID {}: {:?}", pid, e),
    }
}

