mod app;
mod digits;
mod stats;
mod ui;

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "pomo-clock", about = "Pomodoro technique timer TUI")]
struct Cli {
    /// Work duration in minutes
    #[arg(long, default_value_t = 25)]
    work: u64,

    /// Short break duration in minutes
    #[arg(long, default_value_t = 5)]
    short_break: u64,

    /// Long break duration in minutes
    #[arg(long, default_value_t = 15)]
    long_break: u64,

    /// Number of work sessions before long break
    #[arg(long, default_value_t = 4)]
    sessions: u32,

    /// Auto-start next phase
    #[arg(long)]
    auto_start: bool,

    /// Command to run on phase completion
    #[arg(long)]
    notify_cmd: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::new(
        cli.work,
        cli.short_break,
        cli.long_break,
        cli.sessions,
        cli.auto_start,
        cli.notify_cmd,
    );

    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut app::App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char(' ') => app.toggle_pause(),
                        KeyCode::Char('r') => app.reset_current(),
                        KeyCode::Char('s') => app.skip_phase(),
                        KeyCode::Char('+') | KeyCode::Char('=') => app.adjust_time(60),
                        KeyCode::Char('-') => app.adjust_time(-60),
                        KeyCode::Tab => app.toggle_stats_view(),
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }
    }
}
