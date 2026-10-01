use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal,
    Frame,
    layout::{Constraint, Layout},
    text::Line,
    widgets::Paragraph,
};

use crate::player::{TrackInfo, play_audio};

struct PlaybackState {
    track_info: Option<TrackInfo>,
    is_playing: bool,
}

struct App {
    state: Arc<Mutex<PlaybackState>>,
}

impl App {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(PlaybackState {
                track_info: None,
                is_playing: false,
            })),
        }
    }

    fn handle_input(&self, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Char('p') | KeyCode::Char('P') => {
                let mut state = self.state.lock().unwrap();
                if state.is_playing {
                    return true;
                }
                state.is_playing = true;
                drop(state);

                let state = Arc::clone(&self.state);
                std::thread::spawn(move || {
                    let result = play_audio(|info| {
                        state.lock().unwrap().track_info = Some(info.clone());
                    });

                    if let Err(e) = result {
                        eprintln!("Error playing audio: {}", e);
                    }
                    state.lock().unwrap().is_playing = false;
                });
                true
            }
            KeyCode::Esc => false,
            _ => true,
        }
    }
}

pub fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let app = App::new();

    loop {
        terminal.draw(|frame| render(frame, &app.state.lock().unwrap()))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key_event) = event::read()? {
                if !app.handle_input(key_event.code) {
                    return Ok(());
                }
            }
        }
    }
}

fn render(frame: &mut Frame, state: &PlaybackState) {
    let unknown = "Unknown";
    let header = if state.is_playing {
        "Now playing — Q/Esc to quit"
    } else {
        "Press P to play audio and Q to quit"
    };
    let [header_area, body_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(frame.area());

    frame.render_widget(header, header_area);

    let mut lines = Vec::new();

    if let Some(info) = &state.track_info {
        lines.push(Line::from(format!(
            "Title: {}",
            info.title.as_deref().unwrap_or(unknown)
        )));

        lines.push(Line::from(format!(
            "Artist: {}",
            info.artist.as_deref().unwrap_or(unknown)
        )));

        lines.push(Line::from(format!(
            "Album: {}",
            info.album.as_deref().unwrap_or(unknown)
        )));
    }

    frame.render_widget(Paragraph::new(lines), body_area);
}
