use ratatui::style::{Color, Modifier, Style};

/// Theme tokens for consistent styling across the TUI
pub struct Theme {
    pub title: Style,
    pub border: Style,
    pub selected: Style,
    pub normal: Style,
    pub muted: Style,
    pub danger: Style,
    pub warning: Style,
    pub success: Style,
    pub key_hint: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            title: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::DarkGray),
            selected: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            normal: Style::default().fg(Color::White),
            muted: Style::default().fg(Color::DarkGray),
            danger: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            warning: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            success: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            key_hint: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }
}
