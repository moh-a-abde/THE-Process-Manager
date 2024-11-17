// src/process/display.rs
use procfs::process::Process;
use crate::process::data::PAGE_SIZE;
use crate::process::data::parse_status_file;

pub fn get_process_info(pid: i32) -> String {
    match Process::new(pid) {
        Ok(process) => {
            match process.stat() {
                Ok(stat) => {
                    let (voluntary_ctxt_switches, nonvoluntary_ctxt_switches) =
                        parse_status_file(stat.pid as u32).unwrap_or((0, 0));

                    format!(
                        "PID: {}\nCommand: {}\nState: {}\nCPU Usage: {} ticks\nVirtual Memory: {} KB\nResident Memory: {} KB\nThreads: {}\nVoluntary Context Switches: {}\nNonvoluntary Context Switches: {}",
                        stat.pid,
                        stat.comm,
                        stat.state,
                        stat.utime + stat.stime,
                        stat.vsize / 1024,
                        (stat.rss * PAGE_SIZE) / 1024,
                        stat.num_threads,
                        voluntary_ctxt_switches,
                        nonvoluntary_ctxt_switches
                    )
                }
                Err(e) => format!("Failed to get stat for process {}: {:?}", pid, e),
            }
        }
        Err(e) => format!("Failed to find process with PID {}: {:?}", pid, e),
    }
}

