use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use howler_core::Database;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

struct App {
    sightings: Vec<SightingData>,
    selected_sighting: usize,
    map_offset: (i32, i32),
}

struct SightingData {
    species: String,
    latitude: f64,
    longitude: f64,
    date: String,
    source: String,
    details: Option<String>,
}

impl App {
    fn new() -> Result<Self> {
        let db = Database::new("howler.db")?;
        let sightings = db.get_all_sightings()?;

        let sighting_data = sightings
            .into_iter()
            .map(|s| SightingData {
                species: s.species,
                latitude: s.latitude,
                longitude: s.longitude,
                date: s.observed_on.format("%Y-%m-%d").to_string(),
                source: s.source.to_string(),
                details: s.details,
            })
            .collect();

        Ok(Self {
            sightings: sighting_data,
            selected_sighting: 0,
            map_offset: (0, 0),
        })
    }

    fn next(&mut self) {
        if !self.sightings.is_empty() {
            self.selected_sighting = (self.selected_sighting + 1) % self.sightings.len();
        }
    }

    fn previous(&mut self) {
        if !self.sightings.is_empty() {
            self.selected_sighting = if self.selected_sighting == 0 {
                self.sightings.len() - 1
            } else {
                self.selected_sighting - 1
            };
        }
    }
}

fn render_map(f: &mut Frame, area: Rect, sightings: &[SightingData], _offset: (i32, i32)) {
    let width = area.width as usize;
    let height = area.height as usize;

    if width < 2 || height < 2 {
        return;
    }

    let mut grid = vec![vec![' '; width]; height];

    if sightings.is_empty() {
        let text = Text::from("No sightings data available");
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Map View"))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let min_lat = sightings
        .iter()
        .map(|s| s.latitude)
        .fold(f64::INFINITY, f64::min);
    let max_lat = sightings
        .iter()
        .map(|s| s.latitude)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_lon = sightings
        .iter()
        .map(|s| s.longitude)
        .fold(f64::INFINITY, f64::min);
    let max_lon = sightings
        .iter()
        .map(|s| s.longitude)
        .fold(f64::NEG_INFINITY, f64::max);

    let lat_range = max_lat - min_lat;
    let lon_range = max_lon - min_lon;

    for sighting in sightings {
        if lat_range > 0.0 && lon_range > 0.0 {
            let x =
                ((sighting.longitude - min_lon) / lon_range * (width as f64 - 2.0)) as usize + 1;
            let y =
                ((sighting.latitude - min_lat) / lat_range * (height as f64 - 2.0)) as usize + 1;

            if y < height && x < width {
                grid[y][x] = match sighting.source.as_str() {
                    "GBIF" => 'G',
                    "Movebank" => 'M',
                    "iNaturalist" => 'I',
                    _ => '*',
                };
            }
        }
    }

    let lines: Vec<ratatui::prelude::Line> = grid
        .into_iter()
        .map(|row| {
            let line: String = row.into_iter().collect();
            ratatui::prelude::Line::from(line)
        })
        .collect();

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Map View (G=GBIF, M=Movebank, I=iNaturalist)"),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(f.area());

    let items: Vec<ListItem> = app
        .sightings
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.selected_sighting {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("{} - {}", s.species, s.date)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Sightings"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    render_map(f, right_chunks[0], &app.sightings, app.map_offset);

    let details = if let Some(sighting) = app.sightings.get(app.selected_sighting) {
        format!(
            "Species: {}\nDate: {}\nSource: {}\nLat: {:.4}\nLon: {:.4}\nDetails: {}",
            sighting.species,
            sighting.date,
            sighting.source,
            sighting.latitude,
            sighting.longitude,
            sighting.details.as_deref().unwrap_or("N/A")
        )
    } else {
        "No sighting selected".to_string()
    };

    let details_paragraph = Paragraph::new(details.as_str())
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });
    f.render_widget(details_paragraph, right_chunks[1]);
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    let mut last_tick = std::time::Instant::now();
    let tick_rate = Duration::from_millis(1000);

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }
}

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new()?;

    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}
