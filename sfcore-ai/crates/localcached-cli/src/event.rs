use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

use crate::app::{App, MenuItem, Screen};

/// Poll for keyboard events with timeout
pub fn poll_event(timeout_ms: u64) -> anyhow::Result<Option<Event>> {
    if event::poll(Duration::from_millis(timeout_ms))? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

/// Handle keyboard input, returns true if an async action is needed
pub async fn handle_event(app: &mut App, event: Event) -> anyhow::Result<bool> {
    if let Event::Key(key) = event {
        // Only handle key press events (not release)
        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }

        match &app.screen {
            Screen::MainMenu => handle_main_menu(app, key.code).await,
            Screen::CacheList => handle_cache_list(app, key.code).await,
            Screen::MemoryStatus => handle_memory_status(app, key.code).await,
            Screen::SetMemoryLimit => handle_set_memory_limit(app, key.code).await,
            Screen::ConfirmDelete(key_to_delete) => {
                handle_confirm_delete(app, key.code, key_to_delete.clone()).await
            }
            Screen::ConfirmDeleteAll => handle_confirm_delete_all(app, key.code).await,
            Screen::Message(_, _) => {
                // Any key dismisses the message
                app.screen = Screen::MainMenu;
                Ok(false)
            }
        }
    } else if let Event::Resize(_, _) = event {
        // Resize handled by ratatui automatically
        Ok(false)
    } else {
        Ok(false)
    }
}

async fn handle_main_menu(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.menu_up();
            Ok(false)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.menu_down();
            Ok(false)
        }
        KeyCode::Enter => {
            match app.selected_menu_item() {
                MenuItem::ViewCache => {
                    app.screen = Screen::CacheList;
                    app.load_keys().await?;
                    Ok(true)
                }
                MenuItem::MemoryStatus => {
                    app.screen = Screen::MemoryStatus;
                    app.load_stats().await?;
                    Ok(true)
                }
                MenuItem::SetMemoryLimit => {
                    app.screen = Screen::SetMemoryLimit;
                    app.limit_input.clear();
                    Ok(false)
                }
                MenuItem::DeleteAll => {
                    app.screen = Screen::ConfirmDeleteAll;
                    Ok(false)
                }
                MenuItem::Exit => {
                    app.should_quit = true;
                    Ok(false)
                }
            }
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            Ok(false)
        }
        _ => Ok(false),
    }
}

async fn handle_cache_list(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.cache_up();
            Ok(false)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.cache_down();
            Ok(false)
        }
        KeyCode::Enter | KeyCode::Delete | KeyCode::Char('d') => {
            if let Some(key) = app.selected_cache_key() {
                app.screen = Screen::ConfirmDelete(key.clone());
            }
            Ok(false)
        }
        KeyCode::Char('r') => {
            // Refresh
            app.load_keys().await?;
            Ok(true)
        }
        KeyCode::Esc | KeyCode::Backspace => {
            app.go_back();
            Ok(false)
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
            Ok(false)
        }
        _ => Ok(false),
    }
}

async fn handle_memory_status(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    match code {
        KeyCode::Char('r') => {
            // Refresh stats
            app.load_stats().await?;
            Ok(true)
        }
        KeyCode::Esc | KeyCode::Backspace => {
            app.go_back();
            Ok(false)
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
            Ok(false)
        }
        _ => Ok(false),
    }
}

async fn handle_set_memory_limit(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    match code {
        KeyCode::Char(c) if c.is_ascii_digit() => {
            // Only allow up to 2 digits
            if app.limit_input.len() < 2 {
                app.limit_input.push(c);
            }
            Ok(false)
        }
        KeyCode::Backspace => {
            app.limit_input.pop();
            Ok(false)
        }
        KeyCode::Enter => {
            if let Ok(limit) = app.limit_input.parse::<u8>() {
                if limit > 85 {
                    // Show error
                    app.screen = Screen::Message(
                        "⚠️ Limit too high! Maximum allowed is 85%".to_string(),
                        true,
                    );
                } else if limit == 0 {
                    app.screen = Screen::Message(
                        "Limit must be at least 1%".to_string(),
                        true,
                    );
                } else {
                    app.set_memory_limit(limit).await?;
                }
            } else if !app.limit_input.is_empty() {
                app.screen = Screen::Message(
                    "Invalid number. Enter 1-85.".to_string(),
                    true,
                );
            }
            Ok(true)
        }
        KeyCode::Esc => {
            app.go_back();
            Ok(false)
        }
        _ => Ok(false),
    }
}

async fn handle_confirm_delete(app: &mut App, code: KeyCode, key: String) -> anyhow::Result<bool> {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            let success = app.delete_key(&key).await?;
            if success {
                app.screen = Screen::Message(format!("Deleted: {}", key), false);
            } else {
                app.screen = Screen::Message(
                    app.last_error.clone().unwrap_or("Delete failed".into()),
                    true,
                );
            }
            Ok(true)
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.screen = Screen::CacheList;
            Ok(false)
        }
        _ => Ok(false),
    }
}

async fn handle_confirm_delete_all(app: &mut App, code: KeyCode) -> anyhow::Result<bool> {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            let count = app.delete_all().await?;
            app.screen = Screen::Message(format!("Deleted {} cache items", count), false);
            Ok(true)
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.screen = Screen::MainMenu;
            Ok(false)
        }
        _ => Ok(false),
    }
}
