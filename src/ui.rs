use crate::app::{App, Phase};
use crate::digits;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

const PHOSPHOR_GREEN: Color = Color::Rgb(0, 255, 200);
const DARK_GREEN: Color = Color::Rgb(0, 50, 40);
const WORK_COLOR: Color = Color::Rgb(0, 255, 200);
const SHORT_BREAK_COLOR: Color = Color::Cyan;
const LONG_BREAK_COLOR: Color = Color::Yellow;
const DIM_GREEN: Color = Color::Rgb(0, 128, 100);

fn phase_color(phase: Phase) -> Color {
    match phase {
        Phase::Work => WORK_COLOR,
        Phase::ShortBreak => SHORT_BREAK_COLOR,
        Phase::LongBreak => LONG_BREAK_COLOR,
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // Outer border
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GREEN))
        .title(" pomo-clock ")
        .title_style(Style::default().fg(PHOSPHOR_GREEN).add_modifier(Modifier::BOLD));
    f.render_widget(outer_block, size);

    let inner = inner_rect(size, 1);

    if app.show_stats {
        draw_stats_view(f, app, inner);
    } else {
        draw_timer_view(f, app, inner);
    }
}

fn inner_rect(area: Rect, margin: u16) -> Rect {
    Rect {
        x: area.x + margin,
        y: area.y + margin,
        width: area.width.saturating_sub(margin * 2),
        height: area.height.saturating_sub(margin * 2),
    }
}

fn draw_timer_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Phase label
            Constraint::Length(1),  // Spacer
            Constraint::Length(5),  // ASCII digits
            Constraint::Length(1),  // Spacer
            Constraint::Length(3),  // Progress bar
            Constraint::Length(1),  // Spacer
            Constraint::Length(3),  // Session info
            Constraint::Min(0),    // Spacer
            Constraint::Length(3),  // Controls help
        ])
        .split(area);

    let color = phase_color(app.phase);

    // Phase label
    let status = if app.running { "" } else { " [PAUSED]" };
    let phase_text = format!("{}{}", app.phase.label(), status);
    let phase_label = Paragraph::new(Line::from(vec![
        Span::styled(
            phase_text,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(phase_label, chunks[0]);

    // ASCII art timer
    let time_lines = digits::render_time(app.minutes(), app.seconds());
    let ascii_lines: Vec<Line> = time_lines
        .iter()
        .map(|line| Line::from(Span::styled(line.clone(), Style::default().fg(color))))
        .collect();
    let timer_display = Paragraph::new(ascii_lines).alignment(Alignment::Center);
    f.render_widget(timer_display, chunks[2]);

    // Progress bar
    let progress_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GREEN));
    let gauge = Gauge::default()
        .block(progress_block)
        .gauge_style(Style::default().fg(color).bg(Color::Rgb(20, 20, 20)))
        .ratio(app.progress())
        .label(format!("{:.0}%", app.progress() * 100.0));
    f.render_widget(gauge, chunks[4]);

    // Session info
    let tomatoes = "🍅".repeat(app.stats.today_pomodoros() as usize);
    let tomato_display = if tomatoes.is_empty() {
        "No pomodoros today".to_string()
    } else {
        tomatoes
    };

    let total_work = app.stats.today_work_secs();
    let hours = total_work / 3600;
    let mins = (total_work % 3600) / 60;
    let work_str = if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    };

    let streak = app.stats.current_streak();
    let session_info = vec![
        Line::from(vec![
            Span::styled("Today: ", Style::default().fg(DIM_GREEN)),
            Span::styled(tomato_display, Style::default().fg(color)),
            Span::styled(
                format!("  |  Work: {}  |  Streak: {} day{}", work_str, streak, if streak == 1 { "" } else { "s" }),
                Style::default().fg(DIM_GREEN),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("Session {}/{}", app.completed_work_sessions % app.sessions_before_long + 1, app.sessions_before_long),
                Style::default().fg(DIM_GREEN),
            ),
        ]),
    ];
    let session_para = Paragraph::new(session_info).alignment(Alignment::Center);
    f.render_widget(session_para, chunks[6]);

    // Controls
    let controls = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("[Space]", Style::default().fg(PHOSPHOR_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" Start/Pause  ", Style::default().fg(DIM_GREEN)),
            Span::styled("[r]", Style::default().fg(PHOSPHOR_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" Reset  ", Style::default().fg(DIM_GREEN)),
            Span::styled("[s]", Style::default().fg(PHOSPHOR_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" Skip  ", Style::default().fg(DIM_GREEN)),
            Span::styled("[+/-]", Style::default().fg(PHOSPHOR_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" Adjust  ", Style::default().fg(DIM_GREEN)),
            Span::styled("[Tab]", Style::default().fg(PHOSPHOR_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" Stats  ", Style::default().fg(DIM_GREEN)),
            Span::styled("[q]", Style::default().fg(PHOSPHOR_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" Quit", Style::default().fg(DIM_GREEN)),
        ]),
    ])
    .alignment(Alignment::Center);
    f.render_widget(controls, chunks[8]);
}

fn draw_stats_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Title
            Constraint::Length(1),  // Spacer
            Constraint::Min(10),   // Main stats area
            Constraint::Length(1),  // Spacer
            Constraint::Length(2),  // Controls
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(Span::styled(
        "STATS",
        Style::default()
            .fg(PHOSPHOR_GREEN)
            .add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Split main area into left (today sessions) and right (weekly + all-time)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    // Left: Today's sessions
    draw_today_sessions(f, app, main_chunks[0]);

    // Right: Weekly chart + all-time
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(6)])
        .split(main_chunks[1]);

    draw_weekly_chart(f, app, right_chunks[0]);
    draw_all_time(f, app, right_chunks[1]);

    // Controls
    let controls = Paragraph::new(Line::from(vec![
        Span::styled("[Tab]", Style::default().fg(PHOSPHOR_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(" Back to Timer  ", Style::default().fg(DIM_GREEN)),
        Span::styled("[q]", Style::default().fg(PHOSPHOR_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(" Quit", Style::default().fg(DIM_GREEN)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(controls, chunks[4]);
}

fn draw_today_sessions(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GREEN))
        .title(" Today's Sessions ")
        .title_style(Style::default().fg(PHOSPHOR_GREEN));

    let sessions = app.stats.today_sessions();
    let mut lines = Vec::new();

    if sessions.is_empty() {
        lines.push(Line::from(Span::styled(
            "No sessions yet",
            Style::default().fg(DIM_GREEN),
        )));
    } else {
        for (i, session) in sessions.iter().enumerate() {
            let start = session.start.format("%H:%M").to_string();
            let end = session.end.format("%H:%M").to_string();
            let dur_min = session.duration_secs / 60;
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  #{:<2} ", i + 1),
                    Style::default().fg(PHOSPHOR_GREEN),
                ),
                Span::styled(
                    format!("{} - {}  ({}m)", start, end, dur_min),
                    Style::default().fg(DIM_GREEN),
                ),
            ]));
        }
    }

    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn draw_weekly_chart(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GREEN))
        .title(" This Week ")
        .title_style(Style::default().fg(PHOSPHOR_GREEN));

    let weekly = app.stats.week_daily_totals();

    let bars: Vec<Bar> = weekly
        .iter()
        .map(|(label, count)| {
            Bar::default()
                .value(*count as u64)
                .label(Line::from(label.clone()))
                .style(Style::default().fg(PHOSPHOR_GREEN))
        })
        .collect();

    let bar_group = BarGroup::default().bars(&bars);

    let chart = BarChart::default()
        .block(block)
        .data(bar_group)
        .bar_width(5)
        .bar_gap(1)
        .bar_style(Style::default().fg(PHOSPHOR_GREEN))
        .value_style(Style::default().fg(Color::Black).bg(PHOSPHOR_GREEN));

    f.render_widget(chart, area);
}

fn draw_all_time(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GREEN))
        .title(" All-Time Stats ")
        .title_style(Style::default().fg(PHOSPHOR_GREEN));

    let lines = vec![
        Line::from(vec![
            Span::styled("  Total Pomodoros: ", Style::default().fg(DIM_GREEN)),
            Span::styled(
                format!("{}", app.stats.all_time_pomodoros),
                Style::default().fg(PHOSPHOR_GREEN).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Total Hours:    ", Style::default().fg(DIM_GREEN)),
            Span::styled(
                format!("{:.1}", app.stats.all_time_hours()),
                Style::default().fg(PHOSPHOR_GREEN).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Best Streak:    ", Style::default().fg(DIM_GREEN)),
            Span::styled(
                format!("{} days", app.stats.best_streak),
                Style::default().fg(PHOSPHOR_GREEN).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}
