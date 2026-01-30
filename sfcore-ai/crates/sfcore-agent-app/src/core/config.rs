use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub agent: AgentConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub socket_path: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AgentConfig {
    pub system_prompt: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                socket_path: "/tmp/sfcore-ai.sock".to_string(),
                timeout_seconds: 60,
            },
            agent: AgentConfig {
                system_prompt: "Kamu adalah Expert Agent Planner. Baca context yang diberikan dengan teliti. Analisis request user. Respond dengan JSON yang berisi teknologi stack breakdown. Gunakan <think> untuk menunjukkan reasoning process.".to_string(),
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = "settings.toml";
        if Path::new(config_path).exists() {
            match fs::read_to_string(config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("Failed to parse config: {}", e),
                },
                Err(e) => eprintln!("Failed to read config file: {}", e),
            }
        }

        let default_config = Self::default();
        // Optionally save default if missing? For now just use default
        default_config
    }
}
