use std::time::Instant;

use ratatui::style::Color;

// ── Phase ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Phase::Work => "WORK",
            Phase::ShortBreak => "SHORT BREAK",
            Phase::LongBreak => "LONG BREAK",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Phase::Work => Color::Cyan,
            Phase::ShortBreak => Color::Green,
            Phase::LongBreak => Color::Yellow,
        }
    }
}

// ── Settings ───────────────────────────────────────────────────────────────────

pub struct Settings {
    pub work_secs: u64,
    pub short_break_secs: u64,
    pub long_break_secs: u64,
    pub cycles: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_secs: 25 * 60,
            short_break_secs: 5 * 60,
            long_break_secs: 15 * 60,
            cycles: 4,
        }
    }
}

// ── Stats ──────────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct Stats {
    pub completed: u32,
    pub breaks: u32,
    pub focus_secs: u64,
}

// ── App ────────────────────────────────────────────────────────────────────────

pub struct App {
    pub phase: Phase,
    pub running: bool,
    pub remaining: u64,
    pub total: u64,
    pub session: u32,
    pub settings: Settings,
    pub stats: Stats,
    last_tick: Option<Instant>,
}

impl App {
    pub fn new() -> Self {
        let settings = Settings::default();
        let total = settings.work_secs;
        Self {
            phase: Phase::Work,
            running: false,
            remaining: total,
            total,
            session: 1,
            settings,
            stats: Stats::default(),
            last_tick: None,
        }
    }

    pub fn toggle(&mut self) {
        self.running = !self.running;
        if self.running {
            self.last_tick = Some(Instant::now());
        }
    }

    pub fn tick(&mut self) {
        if !self.running {
            return;
        }
        let now = Instant::now();
        let Some(last) = self.last_tick else { return };
        let elapsed = now.duration_since(last).as_secs();
        if elapsed == 0 {
            return;
        }
        self.last_tick = Some(now);
        if self.phase == Phase::Work {
            self.stats.focus_secs += elapsed.min(self.remaining);
        }
        if elapsed >= self.remaining {
            self.complete_phase();
        } else {
            self.remaining -= elapsed;
        }
    }

    pub fn skip(&mut self) {
        self.running = false;
        match self.phase {
            Phase::Work => {
                self.stats.breaks += 1;
                self.advance_session();
            }
            _ => self.set_phase(Phase::Work),
        }
    }

    pub fn reset(&mut self) {
        self.running = false;
        self.session = 1;
        self.stats = Stats::default();
        self.set_phase(Phase::Work);
    }

    pub fn progress(&self) -> f64 {
        if self.total == 0 {
            return 1.0;
        }
        self.remaining as f64 / self.total as f64
    }

    pub fn timer_str(&self) -> String {
        format!("{:02}:{:02}", self.remaining / 60, self.remaining % 60)
    }

    pub fn dots_str(&self) -> String {
        (1..=self.settings.cycles)
            .map(|i| {
                if self.phase == Phase::Work && i == self.session {
                    "○ "
                } else if i < self.session || (self.phase != Phase::Work && i <= self.session) {
                    "● "
                } else {
                    "· "
                }
            })
            .collect()
    }

    fn complete_phase(&mut self) {
        self.running = false;
        match self.phase {
            Phase::Work => {
                self.stats.completed += 1;
                self.stats.breaks += 1;
                notify("Pomodoro complete!", "Time for a break.");
                self.advance_session();
            }
            Phase::ShortBreak => {
                notify("Short break over!", "Time to focus.");
                self.set_phase(Phase::Work);
            }
            Phase::LongBreak => {
                notify("Long break over!", "Time to focus.");
                self.set_phase(Phase::Work);
            }
        }
    }

    fn advance_session(&mut self) {
        if self.session >= self.settings.cycles {
            self.session = 1;
            self.set_phase(Phase::LongBreak);
        } else {
            self.session += 1;
            self.set_phase(Phase::ShortBreak);
        }
    }

    fn set_phase(&mut self, phase: Phase) {
        self.phase = phase;
        self.total = match phase {
            Phase::Work => self.settings.work_secs,
            Phase::ShortBreak => self.settings.short_break_secs,
            Phase::LongBreak => self.settings.long_break_secs,
        };
        self.remaining = self.total;
    }
}

// ── Notification ───────────────────────────────────────────────────────────────

fn notify(title: &str, body: &str) {
    #[cfg(target_os = "macos")]
    {
        let script = format!("display notification \"{}\" with title \"{}\"", body, title);
        let _ = std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("notify-send")
            .args([title, body])
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType=WindowsRuntime] | Out-Null; \
             $t = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent(0); \
             $t.GetElementsByTagName('text')[0].AppendChild($t.CreateTextNode('{}')); \
             [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('pomodoro').Show([Windows.UI.Notifications.ToastNotification]::new($t))",
            title
        );
        let _ = std::process::Command::new("powershell")
            .args(["-Command", &script])
            .spawn();
    }
}
