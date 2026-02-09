
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub db_path: String,
    pub port: u16,
}

impl Config {
    pub fn init() -> Self {
        // Basic hardcoded defaults as fallback, but prefer env
        let db_path = env::var("DATABASE_URL")
            .or_else(|_| env::var("DB_PATH"))
            .unwrap_or_else(|_| "/home/sfcore/server-db/SFCoreProTM.db".to_string());
        
        Self {
            db_path,
            port: 3000,
        }
    }
}
