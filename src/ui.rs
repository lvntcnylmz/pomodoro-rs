use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Gauge, Paragraph},
};

use crate::app::App;

const DIGITS: [[&str; 5]; 11] = [
    ["██████", "██  ██", "██  ██", "██  ██", "██████"], // 0
    ["████  ", "  ██  ", "  ██  ", "  ██  ", "██████"], // 1
    ["██████", "    ██", "██████", "██    ", "██████"], // 2
    ["██████", "    ██", " █████", "    ██", "██████"], // 3
    ["██  ██", "██  ██", "██████", "    ██", "    ██"], // 4
    ["██████", "██    ", "██████", "    ██", "██████"], // 5
    ["██████", "██    ", "██████", "██  ██", "██████"], // 6
    ["██████", "    ██", "    ██", "    ██", "    ██"], // 7
    ["██████", "██  ██", "██████", "██  ██", "██████"], // 8
    ["██████", "██  ██", "██████", "    ██", "██████"], // 9
    ["      ", "  ██  ", "      ", "  ██  ", "      "], // :
];

const DIGIT_W: usize = 6;
const DIGIT_GAP: usize = 2;

fn char_to_idx(c: char) -> Option<usize> {
    match c {
        '0'..='9' => Some((c as u8 - b'0') as usize),
        ':' => Some(10),
        _ => None,
    }
}

// ── Public entry point ─────────────────────────────────────────────────────────

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let [timer_area, keys_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(area);

    draw_timer(frame, app, timer_area);
    draw_keys(frame, app, keys_area);
}

// ── Timer block ────────────────────────────────────────────────────────────────

fn draw_timer(frame: &mut Frame, app: &App, area: Rect) {
    let color = app.phase.color();

    let title = format!(
        "[pomodoro]——[session {}/{}]",
        app.session, app.settings.cycles
    );
    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_style(Style::new().fg(Color::DarkGray));
    frame.render_widget(&block, area);

    let inner = block.inner(area);

    let [_g0, dots, mode, _g1, timer, _g2, gauge, _g3, stats, _g4] = Layout::vertical([
        Constraint::Fill(1),   // gap
        Constraint::Length(1), // dots
        Constraint::Length(1), // mode
        Constraint::Fill(1),   // gap
        Constraint::Min(5),    // timer — grows with terminal
        Constraint::Fill(1),   // gap
        Constraint::Length(1), // gauge
        Constraint::Fill(1),   // gap
        Constraint::Length(3), // stats
        Constraint::Fill(1),   // gap
    ])
    .areas(inner);

    frame.render_widget(
        Paragraph::new(app.dots_str())
            .style(Style::new().fg(color))
            .alignment(Alignment::Center),
        dots,
    );
    frame.render_widget(
        Paragraph::new(app.phase.label())
            .style(Style::new().fg(color))
            .alignment(Alignment::Center),
        mode,
    );

    let time_str = app.timer_str();
    let min_w = (time_str.chars().filter_map(char_to_idx).count() * (DIGIT_W + DIGIT_GAP))
        .saturating_sub(DIGIT_GAP) as u16;

    if timer.height >= 5 && inner.width >= min_w {
        render_big_timer(frame, &time_str, color, timer);
    } else {
        frame.render_widget(
            Paragraph::new(time_str)
                .style(Style::new().fg(color).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center),
            timer,
        );
    }

    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::new().fg(color).bg(Color::Black))
            .ratio(app.progress())
            .label(""),
        gauge,
    );
    frame.render_widget(
        Paragraph::new(stats_lines(app)).alignment(Alignment::Center),
        stats,
    );
}

// ── Pixel font renderer ────────────────────────────────────────────────────────

fn render_big_timer(frame: &mut Frame, time_str: &str, color: Color, area: Rect) {
    let indices: Vec<usize> = time_str.chars().filter_map(char_to_idx).collect();
    let total_w = indices.len() * DIGIT_W + indices.len().saturating_sub(1) * DIGIT_GAP;
    let x_pad = (area.width as usize).saturating_sub(total_w) / 2;
    let y_pad = area.height.saturating_sub(5) / 2;

    for row in 0u16..5 {
        let y = area.y + y_pad + row;
        if y >= area.y + area.height {
            break;
        }
        let mut line = " ".repeat(x_pad);
        for (i, &idx) in indices.iter().enumerate() {
            line.push_str(DIGITS[idx][row as usize]);
            if i + 1 < indices.len() {
                line.push_str(&" ".repeat(DIGIT_GAP));
            }
        }
        frame.render_widget(
            Paragraph::new(Span::styled(
                line,
                Style::new().fg(color).add_modifier(Modifier::BOLD),
            )),
            Rect {
                y,
                height: 1,
                ..area
            },
        );
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn stats_lines(app: &App) -> Vec<Line<'static>> {
    let dim = Style::new().fg(Color::DarkGray);
    let val = Style::new().fg(Color::White);
    vec![
        Line::from(vec![
            Span::styled("󰸞 completed  ", dim),
            Span::styled(app.stats.completed.to_string(), val),
        ])
        .alignment(Alignment::Center),
        Line::from(vec![
            Span::styled("󱫒 focus      ", dim),
            Span::styled(format!("{}m", app.stats.focus_secs / 60), val),
        ])
        .alignment(Alignment::Center),
        Line::from(vec![
            Span::styled("󰅶 breaks     ", dim),
            Span::styled(app.stats.breaks.to_string(), val),
        ])
        .alignment(Alignment::Center),
    ]
}

fn draw_keys(frame: &mut Frame, app: &App, area: Rect) {
    let label = if app.running { "pause" } else { "start" };
    let hint = format!("[space] {label}   [s] skip   [r] reset   [q] quit");
    frame.render_widget(
        Paragraph::new(hint)
            .style(Style::new().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::bordered().border_style(Style::new().fg(Color::DarkGray))),
        area,
    );
}
