use localcached_client::{CacheStats, Client, SetLimitResult};

/// Screen states for the TUI state machine
#[derive(Clone, Debug, PartialEq)]
pub enum Screen {
    MainMenu,
    CacheList,
    MemoryStatus,                // View memory usage
    SetMemoryLimit,              // Input screen for memory limit
    ConfirmDelete(String),       // Key to delete
    ConfirmDeleteAll,
    Message(String, bool),       // Message, is_error
}

/// Menu items for navigation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItem {
    ViewCache,
    MemoryStatus,
    SetMemoryLimit,
    DeleteAll,
    Exit,
}

impl MenuItem {
    pub fn label(&self) -> &'static str {
        match self {
            MenuItem::ViewCache => "📦 View All Cache Items",
            MenuItem::MemoryStatus => "📊 View Memory Status",
            MenuItem::SetMemoryLimit => "⚙️  Set Memory Limit",
            MenuItem::DeleteAll => "🗑️  Delete All Cache",
            MenuItem::Exit => "🚪 Exit",
        }
    }

    pub fn all() -> Vec<MenuItem> {
        vec![
            MenuItem::ViewCache,
            MenuItem::MemoryStatus,
            MenuItem::SetMemoryLimit,
            MenuItem::DeleteAll,
            MenuItem::Exit,
        ]
    }
}

/// Application state
pub struct App {
    pub screen: Screen,
    pub should_quit: bool,
    pub socket_path: String,

    // Menu state
    pub menu_index: usize,
    pub menu_items: Vec<MenuItem>,

    // Cache list state
    pub cache_keys: Vec<String>,
    pub cache_index: usize,
    pub is_loading: bool,
    pub last_error: Option<String>,

    // Memory status
    pub stats: Option<CacheStats>,

    // Memory limit input
    pub limit_input: String,
}

impl App {
    pub fn new(socket_path: String) -> Self {
        Self {
            screen: Screen::MainMenu,
            should_quit: false,
            socket_path,
            menu_index: 0,
            menu_items: MenuItem::all(),
            cache_keys: Vec::new(),
            cache_index: 0,
            is_loading: false,
            last_error: None,
            stats: None,
            limit_input: String::new(),
        }
    }

    pub fn selected_menu_item(&self) -> MenuItem {
        self.menu_items[self.menu_index]
    }

    pub fn selected_cache_key(&self) -> Option<&String> {
        self.cache_keys.get(self.cache_index)
    }

    // Navigation helpers
    pub fn menu_up(&mut self) {
        if self.menu_index > 0 {
            self.menu_index -= 1;
        }
    }

    pub fn menu_down(&mut self) {
        if self.menu_index < self.menu_items.len() - 1 {
            self.menu_index += 1;
        }
    }

    pub fn cache_up(&mut self) {
        if self.cache_index > 0 {
            self.cache_index -= 1;
        }
    }

    pub fn cache_down(&mut self) {
        if self.cache_index < self.cache_keys.len().saturating_sub(1) {
            self.cache_index += 1;
        }
    }

    pub fn go_back(&mut self) {
        match &self.screen {
            Screen::CacheList | Screen::ConfirmDelete(_) | Screen::ConfirmDeleteAll
            | Screen::MemoryStatus | Screen::SetMemoryLimit => {
                self.screen = Screen::MainMenu;
                self.limit_input.clear();
            }
            Screen::Message(_, _) => {
                self.screen = Screen::MainMenu;
            }
            _ => {}
        }
    }

    // Async operations (called from main loop)
    pub async fn load_keys(&mut self) -> anyhow::Result<()> {
        self.is_loading = true;
        self.last_error = None;

        match Client::connect(&self.socket_path).await {
            Ok(mut client) => {
                match client.keys("").await {
                    Ok(keys) => {
                        self.cache_keys = keys;
                        self.cache_index = 0;
                        self.is_loading = false;
                    }
                    Err(e) => {
                        self.last_error = Some(format!("Failed to list keys: {}", e));
                        self.is_loading = false;
                    }
                }
            }
            Err(e) => {
                self.last_error = Some(format!("Connection failed: {}", e));
                self.is_loading = false;
            }
        }
        Ok(())
    }

    pub async fn load_stats(&mut self) -> anyhow::Result<()> {
        self.is_loading = true;
        self.last_error = None;

        match Client::connect(&self.socket_path).await {
            Ok(mut client) => {
                match client.stats().await {
                    Ok(stats) => {
                        self.stats = Some(stats);
                        self.is_loading = false;
                    }
                    Err(e) => {
                        self.last_error = Some(format!("Failed to get stats: {}", e));
                        self.is_loading = false;
                    }
                }
            }
            Err(e) => {
                self.last_error = Some(format!("Connection failed: {}", e));
                self.is_loading = false;
            }
        }
        Ok(())
    }

    pub async fn set_memory_limit(&mut self, limit_percent: u8) -> anyhow::Result<()> {
        match Client::connect(&self.socket_path).await {
            Ok(mut client) => {
                match client.set_memory_limit(limit_percent).await {
                    Ok(result) => {
                        match result {
                            SetLimitResult::Success { old_percent, new_percent } => {
                                self.screen = Screen::Message(
                                    format!("Memory limit changed: {}% → {}%", old_percent, new_percent),
                                    false,
                                );
                            }
                            SetLimitResult::TooHigh { max_percent } => {
                                self.screen = Screen::Message(
                                    format!("⚠️ Limit too high! Maximum allowed is {}%", max_percent),
                                    true,
                                );
                            }
                        }
                    }
                    Err(e) => {
                        self.screen = Screen::Message(format!("Failed: {}", e), true);
                    }
                }
            }
            Err(e) => {
                self.screen = Screen::Message(format!("Connection failed: {}", e), true);
            }
        }
        self.limit_input.clear();
        Ok(())
    }

    pub async fn delete_key(&mut self, key: &str) -> anyhow::Result<bool> {
        match Client::connect(&self.socket_path).await {
            Ok(mut client) => {
                match client.del(key).await {
                    Ok(_) => {
                        // Refresh list
                        self.load_keys().await?;
                        Ok(true)
                    }
                    Err(e) => {
                        self.last_error = Some(format!("Delete failed: {}", e));
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                self.last_error = Some(format!("Connection failed: {}", e));
                Ok(false)
            }
        }
    }

    pub async fn delete_all(&mut self) -> anyhow::Result<usize> {
        match Client::connect(&self.socket_path).await {
            Ok(mut client) => {
                match client.clear_all().await {
                    Ok(count) => {
                        self.cache_keys.clear();
                        self.cache_index = 0;
                        Ok(count)
                    }
                    Err(e) => {
                        self.last_error = Some(format!("Clear failed: {}", e));
                        Ok(0)
                    }
                }
            }
            Err(e) => {
                self.last_error = Some(format!("Connection failed: {}", e));
                Ok(0)
            }
        }
    }
}
