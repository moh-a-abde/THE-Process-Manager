use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    run_gui();
    Ok(())
}

fn run_gui() {
    let app = Application::builder()
        .application_id("com.malak.the_process_manager")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Task Manager")
            .default_width(800)
            .default_height(600)
            .build();

        window.show();
    });

    app.run();
}
