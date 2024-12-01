use gtk::prelude::*; // Import GTK prelude to make the GTK types available
use gtk::{Application, ApplicationWindow, Button, Label, ScrolledWindow, TextView, Dialog, ResponseType, Entry, CheckButton}; // Add CheckButton to the import
use std::collections::HashSet; // Import HashSet for filtering states
mod process;
use crate::process::data::*;

// Function to create the welcome message dialog
fn show_welcome_dialog() {
    let dialog = Dialog::new();
    dialog.set_title("Welcome");
    dialog.set_default_size(300, 100);

    let label = Label::new(Some("Welcome Dr Amr El-Kadi To Our Process Manager!\nWe hope you enjoy it :)"));
    dialog.content_area().pack_start(&label, true, true, 5);

    dialog.add_button("OK", ResponseType::Ok);

    dialog.show_all();
    dialog.run();
    dialog.close();
}

// Function to filter processes by selected states
fn filter_process_info(processes: &[ProcessUsage], selected_states: &HashSet<String>) -> Vec<ProcessUsage> {
    processes.iter().filter(|&process| selected_states.contains(&process.state)).cloned().collect()
}

// Function to build the UI
fn build_ui(application: &Application) {
    show_welcome_dialog();

    let window = ApplicationWindow::new(application);
    window.set_title("Process Manager");
    window.set_default_size(600, 400);

    // Create the text view and clone it for later use
    let text_view = TextView::new();
    text_view.set_editable(false);
    let text_view_clone = text_view.clone(); // Clone for the closure
    let second_text_view_clone = text_view.clone(); 

    let scroll_window = ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    scroll_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll_window.set_min_content_height(200);
    scroll_window.set_min_content_width(400);
    scroll_window.add(&text_view);

    // Button to fetch process info for a given PID
    let button_pid = Button::with_label("Get Process Info");
    button_pid.connect_clicked({
        let text_view = text_view.clone(); // Clone the text view for the closure
        move |_| {
            let dialog = Dialog::new();
            dialog.set_title("Enter PID");
            dialog.set_default_size(300, 100);

            let entry = Entry::new();
            entry.set_placeholder_text(Some("Enter PID"));
            dialog.content_area().pack_start(&entry, false, false, 5);
            dialog.add_button("OK", ResponseType::Ok);
            dialog.add_button("Cancel", ResponseType::Cancel);
            dialog.show_all();

            if dialog.run() == ResponseType::Ok {
                let pid_text = entry.text().to_string();
                if let Ok(pid) = pid_text.parse::<i32>() {
                    let process_info = get_process_info(pid);
                    let buffer = text_view.buffer().expect("Failed to get TextBuffer");
                    buffer.set_text("");
                    buffer.insert_at_cursor(&process_info);
                } else {
                    let buffer = text_view.buffer().expect("Failed to get TextBuffer");
                    buffer.set_text("Invalid PID. Please enter a valid number.");
                }
            }

            dialog.close();
        }
    });

    // Button to fetch all processes info
    let button_all = Button::with_label("Get All Processes");
    button_all.connect_clicked(move |_| {
        let processes = get_processes();
        let mut all_processes_info = String::new();

        for process in processes {
            all_processes_info.push_str(&format!(
                "PID: {}, Name: {}, State: {}, CPU: {:.2}%, Virtual Mem: {:.2} KB, Resident Mem: {:.2} KB\n",
                process.pid,
                process.name,
                process.state,
                process.cpu_usage,
                process.virtual_memory_usage,
                process.resident_memory_usage,
            ));
        }

        if let Some(buffer) = text_view.buffer() {
            buffer.set_text(&all_processes_info);
        }
    });

    // Sort processes button
    let sort_button = Button::with_label("Sort Processes");
    sort_button.connect_clicked(move |_| {
        let dialog = Dialog::new();
        dialog.set_title("Sort Processes");
        dialog.set_default_size(300, 100);

        let entry = Entry::new();
        entry.set_placeholder_text(Some("Enter 'cpu' or 'memory'"));
        dialog.content_area().pack_start(&entry, false, false, 5);
        dialog.add_button("Sort", ResponseType::Ok);
        dialog.add_button("Cancel", ResponseType::Cancel);
        dialog.show_all();

        if dialog.run() == ResponseType::Ok {
            let user_input = entry.text().to_lowercase();
            let sorted_processes: Vec<ProcessUsage>;

            if user_input == "cpu" {
                sorted_processes = sort_processes_by_cpu();
            } else if user_input == "memory" {
                sorted_processes = sort_processes_by_memory();
            } else {
                sorted_processes = get_processes();
            }

            let mut all_processes_info = String::new();
            for process in sorted_processes {
                all_processes_info.push_str(&format!(
                    "PID: {}, Name: {}, State: {}, CPU: {:.2}%, Virtual Mem: {:.2} KB, Resident Mem: {:.2} KB\n",
                    process.pid,
                    process.name,
                    process.state,
                    process.cpu_usage,
                    process.virtual_memory_usage,
                    process.resident_memory_usage,
                ));
            }

            if let Some(buffer) = text_view_clone.buffer() {
                buffer.set_text(&all_processes_info);
            }
        }

        dialog.close();
    });


    // Button to filter processes based on state
    let filter_state_button = Button::with_label("Filter Based on State");
    filter_state_button.connect_clicked(move |_| {
        let dialog = Dialog::new();
        dialog.set_title("Filter Processes by State");
        dialog.set_default_size(300, 400);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);

        // List of possible process states
        let states = vec!["Running", "Sleeping", "Disk Sleep", "Zombie", "Stopped", "Idle"];
        let mut checkboxes = Vec::new();
        
        // Create checkboxes for each state
        for state in &states {
            let checkbox = CheckButton::with_label(*state);
            vbox.pack_start(&checkbox, false, false, 5);
            checkboxes.push(checkbox);
        }

        dialog.content_area().pack_start(&vbox, true, true, 5);
        dialog.add_button("OK", ResponseType::Ok);
        dialog.add_button("Cancel", ResponseType::Cancel);
        dialog.show_all();

        if dialog.run() == ResponseType::Ok {
            let mut selected_states = HashSet::new();
            for (i, checkbox) in checkboxes.iter().enumerate() {
                if checkbox.is_active() {
                    let state_str = match i {
                        0 => "Running".to_string(),
                        1 => "Sleeping".to_string(),
                        2 => "Disk Sleep".to_string(),
                        3 => "Zombie".to_string(),
                        4 => "Stopped".to_string(),
                        5 => "Idle".to_string(),
                        _ => continue,
                    };
                    selected_states.insert(state_str);
                }
            }

            // Fetch processes filtered by the selected states
            let processes = get_processes();
            let filtered_processes = filter_process_info(&processes, &selected_states);

            // Display filtered processes
            let mut filtered_processes_info = String::new();
            for process in filtered_processes {
                filtered_processes_info.push_str(&format!(
                    "PID: {}, Name: {}, State: {}, CPU: {:.2}%, Virtual Mem: {:.2} KB, Resident Mem: {:.2} KB\n",
                    process.pid,
                    process.name,
                    process.state,
                    process.cpu_usage,
                    process.virtual_memory_usage,
                    process.resident_memory_usage,
                ));
            }

            if let Some(buffer) = second_text_view_clone.buffer() {
                buffer.set_text(&filtered_processes_info);
            }
        }

        dialog.close();
    });

    // Set up the layout
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.pack_start(&button_pid, false, false, 0);
    vbox.pack_start(&button_all, false, false, 0);
    vbox.pack_start(&sort_button, false, false, 0); // Add the "Sort Processes" button
    vbox.pack_start(&filter_state_button, false, false, 0);
    vbox.pack_start(&scroll_window, true, true, 0);

    window.add(&vbox);
    window.show_all();
}

fn main() {
    let application = Application::new(
        Some("com.malak.process_manager"),
        Default::default(),
    );

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run();
}
