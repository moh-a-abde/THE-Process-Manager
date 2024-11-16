// import modules
use tui::Frame;
use tui::backend::Backend;
use tui::widgets::{Block, Borders, Table, Row, Cell, Paragraph, Wrap};
use tui::layout::{Rect, Constraint, Alignment};
use tui::style::{Style, Modifier, Color};
use crate::process::data::ProcessUsage;
use sysinfo::{System, SystemExt, ProcessorExt};

// Helper function for dynamic colors based on usage
fn usage_color(usage: f64) -> Color {
    if usage < 50.0 {
        Color::Green
    } else if usage < 75.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

// Render the collapsible help section
pub fn render_status_bar<B: Backend>(f: &mut Frame<B>, area: Rect, is_collapsed: bool) {
    let status_text = if is_collapsed {
        "Press [h] to expand help"
    } else {
        "Commands: [q] Quit | [cpu/memory/ppid/state/start_time/priority] Sort | /<states> Filter | [k] Scroll Up | [j] Scroll Down"
    };
    
    let status_bar = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
    
    f.render_widget(status_bar, area);
}

// Function to render system information header
pub fn render_system_info<B: Backend>(f: &mut Frame<B>, area: Rect, system: &System) {
    let cpu_usage = system.processors().iter().map(|p| p.cpu_usage()).sum::<f32>() / system.processors().len() as f32;
    let memory_used = system.used_memory();
    let total_memory = system.total_memory();
    let memory_percentage = (memory_used as f64 / total_memory as f64) * 100.0;
    let uptime = system.uptime();
    let cpu_frequency = system.global_processor_info().frequency();
    let num_cores = system.processors().len();
    let num_processes = system.processes().len();

    // Format information with highlighted section title
    let info = format!(
        "CPU Frequency: {} MHz | Cores: {} | CPU Usage: {:.2}% | Number of Processes: {}\n\
        Memory: {}/{} KB ({:.2}%) | Uptime: {}s",
        cpu_frequency, num_cores, cpu_usage, num_processes,
        memory_used, total_memory, memory_percentage, uptime
    );

    let paragraph = Paragraph::new(info)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("System Information"))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

// Function to render layout with alternating row colors and dynamic CPU/memory indicators
pub fn render_layout<B: Backend>(
    f: &mut Frame<B>,
    layout: &[Rect],
    scroll_offset: usize,
    input: &str,
    command_output: &str,
    processes: &[ProcessUsage],
    is_collapsed: bool,
) {
    let height = layout[0].height as usize - 2;

    let visible_processes = &processes[scroll_offset..(scroll_offset + height).min(processes.len())];

    let header_cells = ["PID", "PPID", "Name", "State", "CPU %", "Memory %", "Start Time", "Priority"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));

    let rows: Vec<Row> = visible_processes
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let cells = vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.ppid.to_string()),
                Cell::from(p.name.clone()),
                Cell::from(p.state.clone()).style(Style::default().fg(match p.state.as_str() {
                    "Running" => Color::Green,
                    "Sleeping" => Color::Yellow,
                    _ => Color::Red,
                })),
                Cell::from(format!("{:.2}%", p.cpu_usage))
                    .style(Style::default().fg(usage_color(p.cpu_usage as f64))),
                Cell::from(format!("{:.2}%", p.memory_usage))
                    .style(Style::default().fg(usage_color(p.memory_usage as f64))),
                Cell::from(p.start_time.clone()),
                Cell::from(p.priority.clone()),
            ];
            Row::new(cells).style(Style::default().bg(if i % 2 == 0 { Color::Black } else { Color::Black}))
        })
        .collect();

    let table = Table::new(rows)
        .header(Row::new(header_cells))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Processes")
                .title_alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray)),
        )
        .widths(&[
            Constraint::Percentage(6),
            Constraint::Percentage(8),
            Constraint::Percentage(25),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
        ]);

    f.render_widget(table, layout[0]);

    let input_text = Paragraph::new(format!("Input: {}", input))
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command Input")
                .title_alignment(Alignment::Center),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(input_text, layout[1]);

    let output_text = Paragraph::new(command_output)
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("THE Process Manager")
                .title_alignment(Alignment::Center),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(output_text, layout[2]);
    
    // Render status bar with collapsible help
    render_status_bar(f, layout[3], is_collapsed);
}

