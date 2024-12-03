use gtk::prelude::*; // Import GTK prelude to make the GTK types available
use gtk::{Application, ApplicationWindow, Button, Label, ScrolledWindow, TextView, Dialog, ResponseType, Entry, CheckButton}; // Add CheckButton to the import
use std::collections::HashSet; // Import HashSet for filtering states
use std::rc::Rc;
use std::cell::RefCell;  // Needed to mutate `grid` safely


mod process;
use crate::process::data::*;

// Updated Function to Apply a CSS Style Based on the Color Passed
fn apply_color_style(widget: &impl IsA<gtk::Widget>, color: &str) {
    let provider = gtk::CssProvider::new();
    let css = format!("label {{ color: {}; }}", color);
    provider
        .load_from_data(css.as_bytes())
        .expect("Failed to load CSS data");
    gtk::StyleContext::add_provider(
        &widget.style_context(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );
}

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

fn filter_process_info(processes: &[ProcessUsage], selected_states: &HashSet<String>) -> Vec<ProcessUsage> {
    processes.iter().filter(|&process| selected_states.contains(&process.state)).cloned().collect()
}

fn clear_grid(grid: &Rc<RefCell<gtk::Grid>>) {
    // Borrow the grid mutably and remove all children
    let mut grid_ref = grid.borrow_mut();
    for child in grid_ref.children() {
        grid_ref.remove(&child);
    }
}
fn build_ui(application: &Application) {
    show_welcome_dialog();

    let window = ApplicationWindow::new(application);
    window.set_title("Process Manager");
    window.set_default_size(600, 400);

    let text_view = TextView::new();
    text_view.set_editable(false);
    let text_view_clone = text_view.clone();
    let second_text_view_clone = text_view.clone();

    let grid = Rc::new(RefCell::new(gtk::Grid::new())); // Wrap the grid in Rc<RefCell>
    grid.borrow_mut().set_column_spacing(10);
    grid.borrow_mut().set_row_spacing(10);

    // Create a ScrolledWindow and add the grid to it
    let scroll_window = ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    scroll_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll_window.set_min_content_height(200);
    scroll_window.set_min_content_width(400);
    
let grid_ref = grid.borrow();  // Borrow the grid once
scroll_window.add(&*grid_ref);  // Add it to the window


    // Button to fetch process info for a given PID
    let button_pid = Button::with_label("Get Process Info");
    button_pid.connect_clicked({
        let grid = Rc::clone(&grid);
        move |_| {
                    clear_grid(&grid);
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
                    let process_info = get_process_info(pid);  // Assuming this is a function that fetches process info
                    let label = Label::new(Some(&process_info));


                    grid.borrow_mut().attach(&label, 0, 1, 1, 1);  // Add to grid (you can adjust position accordingly)
                    grid.borrow_mut().show_all();  // Ensure all elements are visible
                } else {
                    let label = Label::new(Some("Invalid PID. Please enter a valid number."));
                    grid.borrow_mut().attach(&label, 0, 1, 1, 1);
                    grid.borrow_mut().show_all();
                }
            }
            dialog.close();
        }
    });
    
// Updated `button_all` Click Event
let button_all = Button::with_label("Get All Processes");
button_all.connect_clicked({
    let grid = Rc::clone(&grid);
    move |_| {
        clear_grid(&grid);
        let processes = get_processes();

        // Add headers
        grid.borrow_mut().attach(&Label::new(Some("PID")), 0, 0, 1, 1);
        grid.borrow_mut().attach(&Label::new(Some("Name")), 1, 0, 1, 1);
        grid.borrow_mut().attach(&Label::new(Some("State")), 2, 0, 1, 1);
        grid.borrow_mut().attach(&Label::new(Some("CPU %")), 3, 0, 1, 1);
        grid.borrow_mut().attach(&Label::new(Some("Virtual Mem (KB)")), 4, 0, 1, 1);
        grid.borrow_mut().attach(&Label::new(Some("Resident Mem (KB)")), 5, 0, 1, 1);

        // Populate rows with process data
        for (i, process) in processes.iter().enumerate() {
            let pid_label = Label::new(Some(&process.pid.to_string()));
            apply_color_style(&pid_label, "White"); // Default color for PID
            grid.borrow_mut().attach(&pid_label, 0, (i + 1) as i32, 1, 1);

            let name_label = Label::new(Some(&process.name));
            apply_color_style(&name_label, "white"); // Default color for name
            grid.borrow_mut().attach(&name_label, 1, (i + 1) as i32, 1, 1);

            let state_label = Label::new(Some(&process.state));
            // Apply color based on process state
            let state_color = match process.state.as_str() {
                "Running" => "green",   // Green for Running
                "Sleeping" => "gray",   // Gray for Sleeping
                "Idle" => "blue",       // Blue for Idle
                _ => "white",           // Default color
            };
            apply_color_style(&state_label, state_color);
            grid.borrow_mut().attach(&state_label, 2, (i + 1) as i32, 1, 1);

            let cpu_label = Label::new(Some(&format!("{:.2}", process.cpu_usage)));
            apply_color_style(&cpu_label, "white"); // Default color for CPU usage
            grid.borrow_mut().attach(&cpu_label, 3, (i + 1) as i32, 1, 1);

            let virtual_mem_label = Label::new(Some(&format!("{:.2}", process.virtual_memory_usage)));
            apply_color_style(&virtual_mem_label, "white"); // Default color for Virtual Memory
            grid.borrow_mut().attach(&virtual_mem_label, 4, (i + 1) as i32, 1, 1);

            let resident_mem_label = Label::new(Some(&format!("{:.2}", process.resident_memory_usage)));
            apply_color_style(&resident_mem_label, "wh"); // Default color for Resident Memory
            grid.borrow_mut().attach(&resident_mem_label, 5, (i + 1) as i32, 1, 1);
        }

        grid.borrow_mut().show_all();
    }
});


    // Sort processes button
    let sort_button = Button::with_label("Sort Processes");
    sort_button.connect_clicked({
    let grid = Rc::clone(&grid);  // Clone the Rc reference
    move |_| {
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

            // Determine the sorting criterion based on user input
            if user_input == "cpu" {
                sorted_processes = sort_processes_by_cpu();  // Assuming this function is defined elsewhere
            } else if user_input == "memory" {
                sorted_processes = sort_processes_by_memory();  // Assuming this function is defined elsewhere
            } else {
                // Handle invalid input
                return;
            }

            // Remove all children manually
            let mut grid_ref = grid.borrow_mut();  // Borrow the grid mutably
            for child in grid_ref.children() {
                grid_ref.remove(&child);
            }

            // Add headers
            grid_ref.attach(&Label::new(Some("PID")), 0, 0, 1, 1);
            grid_ref.attach(&Label::new(Some("Name")), 1, 0, 1, 1);
            grid_ref.attach(&Label::new(Some("State")), 2, 0, 1, 1);
            grid_ref.attach(&Label::new(Some("CPU %")), 3, 0, 1, 1);
            grid_ref.attach(&Label::new(Some("Virtual Mem (KB)")), 4, 0, 1, 1);
            grid_ref.attach(&Label::new(Some("Resident Mem (KB)")), 5, 0, 1, 1);

            // Populate rows with sorted process data
            for (i, process) in sorted_processes.iter().enumerate() {
                grid_ref.attach(&Label::new(Some(&process.pid.to_string())), 0, (i + 1) as i32, 1, 1);
                grid_ref.attach(&Label::new(Some(&process.name)), 1, (i + 1) as i32, 1, 1);
                grid_ref.attach(&Label::new(Some(&process.state)), 2, (i + 1) as i32, 1, 1);
                grid_ref.attach(&Label::new(Some(&format!("{:.2}", process.cpu_usage))), 3, (i + 1) as i32, 1, 1);
                grid_ref.attach(&Label::new(Some(&format!("{:.2}", process.virtual_memory_usage))), 4, (i + 1) as i32, 1, 1);
                grid_ref.attach(&Label::new(Some(&format!("{:.2}", process.resident_memory_usage))), 5, (i + 1) as i32, 1, 1);
            }

            grid_ref.show_all(); // Ensure all elements are visible
        }

        dialog.close();
    }
});

    // Button to filter processes based on state
let filter_state_button = Button::with_label("Filter Based on State");
filter_state_button.connect_clicked({
    let grid = Rc::clone(&grid);  // Clone the Rc reference
    move |_| {
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

            // Fetch processes and filter by the selected states
            let processes = get_processes();
            let filtered_processes = filter_process_info(&processes, &selected_states);

            // Remove all children from the grid before adding filtered data
            let mut grid_ref = grid.borrow_mut();
            for child in grid_ref.children() {
                grid_ref.remove(&child);
            }

            // Add headers again after clearing the grid
            grid_ref.attach(&Label::new(Some("PID")), 0, 0, 1, 1);
            grid_ref.attach(&Label::new(Some("Name")), 1, 0, 1, 1);
            grid_ref.attach(&Label::new(Some("State")), 2, 0, 1, 1);
            grid_ref.attach(&Label::new(Some("CPU %")), 3, 0, 1, 1);
            grid_ref.attach(&Label::new(Some("Virtual Mem (KB)")), 4, 0, 1, 1);
            grid_ref.attach(&Label::new(Some("Resident Mem (KB)")), 5, 0, 1, 1);

            // Populate grid with filtered process data
            for (i, process) in filtered_processes.iter().enumerate() {
                grid_ref.attach(&Label::new(Some(&process.pid.to_string())), 0, (i + 1) as i32, 1, 1);
                grid_ref.attach(&Label::new(Some(&process.name)), 1, (i + 1) as i32, 1, 1);
                grid_ref.attach(&Label::new(Some(&process.state)), 2, (i + 1) as i32, 1, 1);
                grid_ref.attach(&Label::new(Some(&format!("{:.2}", process.cpu_usage))), 3, (i + 1) as i32, 1, 1);
                grid_ref.attach(&Label::new(Some(&format!("{:.2}", process.virtual_memory_usage))), 4, (i + 1) as i32, 1, 1);
                grid_ref.attach(&Label::new(Some(&format!("{:.2}", process.resident_memory_usage))), 5, (i + 1) as i32, 1, 1);
            }

            grid_ref.show_all();  // Ensure all elements are visible
        }

        dialog.close();
    }
});




    // Set up the layout
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.pack_start(&button_pid, false, false, 0);
    vbox.pack_start(&button_all, false, false, 0);
    vbox.pack_start(&sort_button, false, false, 0);
    vbox.pack_start(&filter_state_button, false, false, 0);
    vbox.pack_start(&scroll_window, true, true, 0);

    window.add(&vbox);
    window.show_all();
}

fn main() {
    let application = Application::new(
        Some("com.example.process_manager"),
        Default::default(),
    );

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run();
}
