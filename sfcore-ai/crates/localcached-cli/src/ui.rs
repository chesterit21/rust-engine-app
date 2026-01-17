use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Screen};
use crate::theme::Theme;

pub fn render(frame: &mut Frame, app: &App) {
    let theme = Theme::default();

    match &app.screen {
        Screen::MainMenu => render_main_menu(frame, app, &theme),
        Screen::CacheList => render_cache_list(frame, app, &theme),
        Screen::MemoryStatus => render_memory_status(frame, app, &theme),
        Screen::SetMemoryLimit => render_set_memory_limit(frame, app, &theme),
        Screen::ConfirmDelete(key) => {
            render_cache_list(frame, app, &theme);
            render_confirm_delete(frame, key, false, &theme);
        }
        Screen::ConfirmDeleteAll => {
            render_main_menu(frame, app, &theme);
            render_confirm_delete(frame, "ALL CACHE ITEMS", true, &theme);
        }
        Screen::Message(msg, is_error) => {
            render_main_menu(frame, app, &theme);
            render_message(frame, msg, *is_error, &theme);
        }
    }
}

fn render_main_menu(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = frame.area();

    // Title block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .title(" 🔥 LocalCached TUI ")
        .title_style(theme.title);

    frame.render_widget(block, area);

    // Inner area
    let inner = Layout::default()
        .constraints([
            Constraint::Length(2), // Padding
            Constraint::Min(10),   // Menu
            Constraint::Length(2), // Footer
        ])
        .split(inner_rect(area, 2));

    // Menu items
    let items: Vec<ListItem> = app
        .menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.menu_index {
                theme.selected
            } else {
                theme.normal
            };
            ListItem::new(format!("  {}  ", item.label())).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(theme.selected);

    frame.render_widget(list, inner[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑/↓ ", theme.key_hint),
        Span::styled("Navigate", theme.muted),
        Span::raw("  "),
        Span::styled(" Enter ", theme.key_hint),
        Span::styled("Select", theme.muted),
        Span::raw("  "),
        Span::styled(" q ", theme.key_hint),
        Span::styled("Quit", theme.muted),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(footer, inner[2]);
}

fn render_cache_list(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = frame.area();

    let title = format!(" 📦 Cache Items ({}) ", app.cache_keys.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .title(title)
        .title_style(theme.title);

    frame.render_widget(block, area);

    let inner = Layout::default()
        .constraints([
            Constraint::Length(1), // Padding
            Constraint::Min(5),    // List
            Constraint::Length(2), // Footer
        ])
        .split(inner_rect(area, 2));

    if app.cache_keys.is_empty() {
        let msg = if app.is_loading {
            "Loading..."
        } else if app.last_error.is_some() {
            "Error loading keys"
        } else {
            "(No cache items)"
        };
        let p = Paragraph::new(msg)
            .style(theme.muted)
            .alignment(Alignment::Center);
        frame.render_widget(p, inner[1]);
    } else {
        let items: Vec<ListItem> = app
            .cache_keys
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let prefix = if i == app.cache_index { "▸ " } else { "  " };
                let style = if i == app.cache_index {
                    theme.selected
                } else {
                    theme.normal
                };
                ListItem::new(format!("{}{}", prefix, key)).style(style)
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner[1]);
    }

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑/↓ ", theme.key_hint),
        Span::styled("Navigate", theme.muted),
        Span::raw("  "),
        Span::styled(" Enter/Del ", theme.key_hint),
        Span::styled("Delete", theme.muted),
        Span::raw("  "),
        Span::styled(" Esc ", theme.key_hint),
        Span::styled("Back", theme.muted),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(footer, inner[2]);
}

fn render_memory_status(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = frame.area();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .title(" 📊 Memory Status ")
        .title_style(theme.title);

    frame.render_widget(block, area);

    let inner = inner_rect(area, 2);

    if let Some(stats) = &app.stats {
        let usage_pct = stats.cache_usage_percent();
        let limit_pct = stats.memory_limit_percent() as f64;

        let layout = Layout::default()
            .constraints([
                Constraint::Length(3), // Cache usage gauge
                Constraint::Length(2), // Spacing
                Constraint::Length(1), // Details line 1
                Constraint::Length(1), // Details line 2
                Constraint::Length(1), // Details line 3
                Constraint::Length(1), // Details line 4
                Constraint::Min(1),    // Spacer
                Constraint::Length(2), // Footer
            ])
            .split(inner);

        // Cache usage gauge
        let ratio = (usage_pct / 100.0).min(1.0);
        let gauge_label = format!("{:.1}% of available RAM", usage_pct);
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title("Cache Usage")
                    .title_style(theme.normal),
            )
            .gauge_style(if usage_pct > limit_pct {
                theme.danger
            } else if usage_pct > limit_pct * 0.8 {
                theme.warning
            } else {
                theme.success
            })
            .ratio(ratio)
            .label(gauge_label);

        frame.render_widget(gauge, layout[0]);

        // Details
        let details = vec![
            format!(
                "Cache Memory:      {} ({:.1}% of available)",
                stats.cache_mem_human(),
                usage_pct
            ),
            format!("Available RAM:     {}", stats.available_mem_human()),
            format!("Memory Limit:      {}%", stats.memory_limit_percent()),
            format!("Total Evictions:   {}", stats.evictions_total),
        ];

        for (i, detail) in details.iter().enumerate() {
            let p = Paragraph::new(detail.as_str()).style(theme.normal);
            frame.render_widget(p, layout[2 + i]);
        }

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" r ", theme.key_hint),
            Span::styled("Refresh", theme.muted),
            Span::raw("  "),
            Span::styled(" Esc ", theme.key_hint),
            Span::styled("Back", theme.muted),
        ]))
        .alignment(Alignment::Center);

        frame.render_widget(footer, layout[7]);
    } else {
        let msg = if app.is_loading {
            "Loading stats..."
        } else if let Some(err) = &app.last_error {
            err.as_str()
        } else {
            "No stats available"
        };
        let p = Paragraph::new(msg)
            .style(theme.muted)
            .alignment(Alignment::Center);
        frame.render_widget(p, inner);
    }
}

fn render_set_memory_limit(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = center_rect(frame.area(), 60, 10);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.title)
        .title(" ⚙️  Set Memory Limit ")
        .title_style(theme.title);

    frame.render_widget(block, area);

    let inner = inner_rect(area, 2);

    let layout = Layout::default()
        .constraints([
            Constraint::Length(1), // Info
            Constraint::Length(1), // Warning
            Constraint::Length(1), // Spacing
            Constraint::Length(1), // Input label
            Constraint::Length(1), // Input
            Constraint::Min(1),    // Spacer
            Constraint::Length(1), // Footer
        ])
        .split(inner);

    // Info
    let info = Paragraph::new("Set maximum cache memory as % of available RAM")
        .style(theme.normal)
        .alignment(Alignment::Center);
    frame.render_widget(info, layout[0]);

    // Warning
    let warning = Paragraph::new("⚠️  Maximum allowed: 85%")
        .style(theme.danger)
        .alignment(Alignment::Center);
    frame.render_widget(warning, layout[1]);

    // Input label
    let label = Paragraph::new("Enter percentage (1-85):")
        .style(theme.muted)
        .alignment(Alignment::Center);
    frame.render_widget(label, layout[3]);

    // Input value
    let input_display = if app.limit_input.is_empty() {
        "_".to_string()
    } else {
        format!("{}%", app.limit_input)
    };
    let input = Paragraph::new(input_display)
        .style(theme.title)
        .alignment(Alignment::Center);
    frame.render_widget(input, layout[4]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" 0-9 ", theme.key_hint),
        Span::styled("Input", theme.muted),
        Span::raw("  "),
        Span::styled(" Backspace ", theme.key_hint),
        Span::styled("Clear", theme.muted),
        Span::raw("  "),
        Span::styled(" Enter ", theme.key_hint),
        Span::styled("Confirm", theme.muted),
        Span::raw("  "),
        Span::styled(" Esc ", theme.key_hint),
        Span::styled("Cancel", theme.muted),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(footer, layout[6]);
}

fn render_confirm_delete(frame: &mut Frame, key: &str, is_all: bool, theme: &Theme) {
    let area = center_rect(frame.area(), 50, 8);

    // Clear background
    frame.render_widget(Clear, area);

    let title = if is_all {
        " ⚠️  Confirm Delete ALL "
    } else {
        " ⚠️  Confirm Delete "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.danger)
        .title(title)
        .title_style(theme.danger);

    frame.render_widget(block, area);

    let inner = inner_rect(area, 2);

    let display_key = if key.len() > 40 {
        format!("{}...", &key[..37])
    } else {
        key.to_string()
    };

    let text = Paragraph::new(vec![
        Line::raw(""),
        Line::from(vec![
            Span::raw("Key: "),
            Span::styled(&display_key, theme.title),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(" [Y] ", theme.key_hint),
            Span::styled("Yes, Delete", theme.danger),
            Span::raw("    "),
            Span::styled(" [N] ", theme.key_hint),
            Span::raw("Cancel"),
        ]),
    ])
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });

    frame.render_widget(text, inner);
}

fn render_message(frame: &mut Frame, msg: &str, is_error: bool, theme: &Theme) {
    let area = center_rect(frame.area(), 50, 6);

    frame.render_widget(Clear, area);

    let style = if is_error {
        theme.danger
    } else {
        theme.success
    };
    let title = if is_error {
        " ❌ Error "
    } else {
        " ✅ Success "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .title(title)
        .title_style(style);

    frame.render_widget(block, area);

    let inner = inner_rect(area, 2);

    let text = Paragraph::new(vec![
        Line::raw(""),
        Line::raw(msg),
        Line::raw(""),
        Line::from(Span::styled("Press any key to continue", theme.muted)),
    ])
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });

    frame.render_widget(text, inner);
}

// Helper: shrink rect by margin
fn inner_rect(area: Rect, margin: u16) -> Rect {
    Rect {
        x: area.x + margin,
        y: area.y + margin,
        width: area.width.saturating_sub(margin * 2),
        height: area.height.saturating_sub(margin * 2),
    }
}

// Helper: center a popup
fn center_rect(area: Rect, percent_x: u16, height: u16) -> Rect {
    let width = area.width * percent_x / 100;
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;
    Rect {
        x: area.x + x,
        y: area.y + y,
        width,
        height,
    }
}
