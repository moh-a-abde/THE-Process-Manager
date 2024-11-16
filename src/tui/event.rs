use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub enum EventAction {
    Quit,
    ScrollUp,
    ScrollDown,
    ExecuteCommand(String),
    ToggleStatusBar,
    None,
}

pub fn handle_events(input: &mut String) -> Result<EventAction, Box<dyn std::error::Error>> {
    if event::poll(Duration::from_millis(200))? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(EventAction::Quit),
                KeyCode::Char('h') => return Ok(EventAction::ToggleStatusBar),
                KeyCode::Down | KeyCode::Char('j') => return Ok(EventAction::ScrollDown),
                KeyCode::Up | KeyCode::Char('k') => return Ok(EventAction::ScrollUp),
                KeyCode::Char(c) => input.push(c),
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => {
                    return Ok(EventAction::ExecuteCommand(input.clone()));
                }
                _ => (),
            }
        }
    }
    Ok(EventAction::None)
}

