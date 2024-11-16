#![allow(unused_assignments)]
pub mod layout;
pub mod render;
pub mod event;

use tui::backend::CrosstermBackend;
use tui::Terminal;
use std::io::{self, Stdout};
use tui::layout::{Layout, Constraint};
use crate::process::data::{get_processes};
use crate::process::display::get_process_info;

use sysinfo::{System, SystemExt};
use std::collections::HashSet;

use crate::process::data::filter_process_info;



pub fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn main_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::new_all(); // Initialize the System struct for gathering system info
    let mut scroll_offset = 0;
    let mut input = String::new();
    let mut command_output = String::new();
    let mut processes = get_processes();
    let mut filtered_processes = processes.clone();
    let mut sort_by = String::from("none");
    let mut active_filter: Option<HashSet<char>> = None;
    let mut is_collapsed = false;

    loop {
        // refresh system information
        system.refresh_all();
        processes = get_processes(); // Reload processes to get updated data

        // apply sorting to processes
        match sort_by.as_str() {
            "cpu" => processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)),
            "memory" => processes.sort_by(|a, b| b.memory_usage.partial_cmp(&a.memory_usage).unwrap_or(std::cmp::Ordering::Equal)),
            "ppid" => processes.sort_by(|a, b| a.ppid.cmp(&b.ppid)),
            "state" => processes.sort_by(|a, b| a.state.cmp(&b.state)),
            "start_time" => processes.sort_by(|a, b| a.start_time.cmp(&b.start_time)),
            "priority" => processes.sort_by(|a, b| a.priority.cmp(&b.priority)),
            _ => {}
        }

        // apply filtering if an active filter exists
        if let Some(filter_by_states) = &active_filter {
            filtered_processes = filter_process_info(&processes, filter_by_states);
        } else {
            filtered_processes = processes.clone(); // No filter; display all processes
        }

        // draw TUI layout
        terminal.draw(|f| {
    // Adjust layout constraints based on `is_collapsed`
    let layout_constraints = if is_collapsed {
        vec![
            Constraint::Percentage(10), // System Info Header
            Constraint::Percentage(50), // Processes Table
            Constraint::Percentage(10), // Input
            Constraint::Percentage(20), // Command Output
            Constraint::Percentage(0),  // Status Bar (collapsed)
        ]
    } else {
        vec![
            Constraint::Percentage(10), // System Info Header
            Constraint::Percentage(50), // Processes Table
            Constraint::Percentage(10), // Input
            Constraint::Percentage(20), // Command Output
            Constraint::Percentage(10), // Status Bar (expanded)
        ]
    };

    // Split the layout dynamically
    let chunks = Layout::default()
        .direction(tui::layout::Direction::Vertical)
        .constraints(layout_constraints)
        .split(f.size());

    // Render components
    render::render_system_info(f, chunks[0], &system); // Render the system info header
    render::render_layout(f, &chunks[1..], scroll_offset, &input, &command_output, &filtered_processes, true);

    // Only render the status bar if not collapsed
    if !is_collapsed {
        render::render_status_bar(f, chunks[4], is_collapsed);
    }
})?;

        // handle events
        match event::handle_events(&mut input)? {
            event::EventAction::Quit => break,
            event::EventAction::ScrollDown => scroll_offset += 1,
            event::EventAction::ScrollUp => {
                if scroll_offset > 0 {
                    scroll_offset -= 1;
                }
            }
            event::EventAction::ToggleStatusBar => {

                is_collapsed = !is_collapsed;

            }
            event::EventAction::ExecuteCommand(command) => {
                if command == "cpu" || command == "memory" || command == "ppid" || command == "state" 
                    || command == "start_time" || command == "priority" {
                    
                    // sort by specified field
                    sort_by = command.clone();
                    command_output = format!("Sorting processes by {}", command);

                } else if command.starts_with("/") {
                    // filtering processes by state using `/` prefix
                    let filter_input = command[1..].to_uppercase(); // Remove the `/` and convert to uppercase
                    let filter_by_states: HashSet<char> = filter_input
                        .chars()
                        .filter(|&state| matches!(state, 'I' | 'S' | 'R' | 'Z')) // only keep valid states
                        .collect();

                    if filter_by_states.is_empty() {
                        command_output = "Invalid input! Please enter valid states like '/IS' or '/RZ'.".to_string();
                        active_filter = None; // clear any active filter
                    } else {
                    
                        // set active filter
                        active_filter = Some(filter_by_states);
                        command_output = format!("Filtered processes by states: {:?}", active_filter);
                    }
                } else if let Ok(pid) = command.parse::<i32>() {
                    // display details for process with specified PID
                    command_output = get_process_info(pid);
                } else {
                    command_output = "Invalid command. Please enter 'cpu', 'memory', 'ppid', 'state', 'start_time', 'priority', or '/<states>' for filtering, or valid PID.".to_string();
                }
                
                input.clear();
            }
            event::EventAction::None => {}
        }
    }

    Ok(())
}

