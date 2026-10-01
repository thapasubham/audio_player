use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal,
    Frame,
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use ratatui_image::{StatefulImage, picker::Picker, protocol::StatefulProtocol};

use crate::player::{TrackInfo, play_audio};

struct PlaybackState {
    track_info: Option<TrackInfo>,
    is_playing: bool,
    cover_version: u64,

}

struct App {
    state: Arc<Mutex<PlaybackState>>,
    picker: Picker,
    cover_protocol: Option<StatefulProtocol>,
    loaded_cover_version: u64,
}

impl App {
    fn new(picker: Picker) -> Self {
        Self {
            state: Arc::new(Mutex::new(PlaybackState {
                track_info: None,
                is_playing: false,
                cover_version: 0,
            })),
            picker,
            cover_protocol: None,
            loaded_cover_version: 0,
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
                        let mut state = state.lock().unwrap();
                        state.track_info = Some(info.clone());
                        state.cover_version += 1;
                    });

                    if let Err(e) = result {
                        eprintln!("Error playing audio: {}", e);
                    }
                    state.lock().unwrap().is_playing = false;
                });
                true
            }
            KeyCode::Esc | KeyCode::Char('q') |KeyCode::Char('Q') => false,
            _ => true,
        }
    }

    fn sync_cover(&mut self) {
        let state = self.state.lock().unwrap();
        if state.cover_version == self.loaded_cover_version {
            return;
        }
        self.loaded_cover_version = state.cover_version;

        let cover_bytes = state
            .track_info
            .as_ref()
            .and_then(|info| info.cover.as_ref());

        self.cover_protocol = cover_bytes.and_then(|bytes| {
            image::load_from_memory(bytes)
                .ok()
                .map(|img| self.picker.new_resize_protocol(img))
        });
    }
}

pub fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
    let mut app = App::new(picker);

    loop {
        app.sync_cover();
        terminal.draw(|frame| render(frame, &app.state.lock().unwrap(), &mut app.cover_protocol))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key_event) = event::read()? {
                if !app.handle_input(key_event.code) {
                    return Ok(());
                }
            }
        }
    }
}

fn render(
    frame: &mut Frame,
    state: &PlaybackState,
    cover_protocol: &mut Option<StatefulProtocol>,
) {
    let unknown = "Unknown";
    let instruction = Line::from(vec!["play".into(), "<p>".blue().bold()," Quit".bold().into(), "<Esc>/<Q>".red().bold()]);
    let header = if state.is_playing {
        "Playing — Q/Esc to quit"
    } else {
        "Press P to play audio and Q to quit"
    };
    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let [browser_area, now_playing_area] =
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
            .areas(body_area);
    let [cover_area, info_area] =
        Layout::vertical([Constraint::Percentage(60), Constraint::Min(4)]).areas(now_playing_area);

    frame.render_widget(header, header_area);

    let browser = Block::default().title(" Songs ").borders(Borders::ALL);
    frame.render_widget(browser, browser_area);

    let cover_block = Block::default().title(" Cover ").borders(Borders::ALL);
    let cover_inner = cover_block.inner(cover_area);
    frame.render_widget(cover_block, cover_area);
    if let Some(protocol) = cover_protocol {
        frame.render_stateful_widget(StatefulImage::default(), cover_inner, protocol);
    }

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
        lines.push(Line::from(format!(
            "Duration: {}",
            info.duration
                .map(|d| format!("{}:{:02}", d / 60, d % 60))
                .unwrap_or_else(|| unknown.to_string())
        )));
        lines.push(Line::from(format!(
            "Year: {}",
            info.year.as_deref().unwrap_or(unknown)
        )));
    }

    let now_playing = Block::default()
        .title(" Now Playing ")
        .borders(Borders::ALL);

    frame.render_widget(Paragraph::new(lines).block(now_playing), info_area);

    frame.render_widget(Paragraph::new(instruction).centered(), footer_area);
}
