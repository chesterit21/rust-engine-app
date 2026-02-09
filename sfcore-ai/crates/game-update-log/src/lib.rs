use anyhow::Result;
use chrono::{DateTime, Local, Timelike};
use sqlx::sqlite::SqlitePoolOptions;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error};

mod models;
mod repository;
mod scraper;

use repository::GameRepository;
use scraper::ScraperService;

pub struct App {
    repository: GameRepository,
    scraper: ScraperService,
}

impl App {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Create connection pool with 45 second timeout
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(45))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(3600))
            .connect(database_url)
            .await?;

        // SQLite optimizations
        // WAL mode for better concurrency
        sqlx::query("PRAGMA journal_mode=WAL;")
            .execute(&pool)
            .await?;
        
        // Synchronous NORMAL - balance between safety and performance
        sqlx::query("PRAGMA synchronous=NORMAL;")
            .execute(&pool)
            .await?;
        
        // Increase cache size (negative = KB, e.g., -64000 = 64MB)
        sqlx::query("PRAGMA cache_size=-32000;")
            .execute(&pool)
            .await?;
        
        // Set busy timeout to 45 seconds (matches connection timeout)
        sqlx::query("PRAGMA busy_timeout=45000;")
            .execute(&pool)
            .await?;
        
        // Enable foreign keys
        sqlx::query("PRAGMA foreign_keys=ON;")
            .execute(&pool)
            .await?;
        
        info!("SQLite configured: WAL mode, 45s timeout, 32MB cache");

        let repository = GameRepository::new(pool);
        let scraper = ScraperService::new(repository.clone()).await?;

        Ok(Self {
            repository,
            scraper,
        })
    }

    /// Main update loop
    pub async fn start_update_log(&self) -> Result<()> {
        info!("-----------------------------------------------------");
        info!("START UPDATE RESULT GAME - {}", Local::now().format("%d %b %Y %H:%M:%S"));
        
        // Update MQ games first
        if let Err(e) = self.scraper.update_result_per_game_mq(13).await {
            error!("Error updating MQ games: {}", e);
        }

        // Update regular games
        // if let Err(e) = self.scraper.update_result_per_game(13).await {
        //     error!("Error updating regular games: {}", e);
        // }

        sleep(Duration::from_secs(1)).await;

        info!("DONE UPDATE RESULT GAME - {}", Local::now().format("%d %b %Y %H:%M:%S"));
        info!("-----------------------------------------------------");

        Ok(())
    }

    /// Validate and display last logs
    pub async fn validate_last_log_games(&self) -> Result<()> {
        let all_games = self.repository.get_all_master_games().await?;

        for game in all_games {
            let logs = self.repository.get_top_logs(&game.game_code, 5).await?;
            
            if logs.is_empty() {
                continue;
            }

            info!("GameCode: {}, Last Log: {}", game.game_code, logs[0].periode);
            
            for log in &logs {
                info!("Log: {} - Result: {}.000", log.periode, log.log_result);
            }
            
            info!("");
            info!("-----------------------------------------------------");
            info!("");
        }

        Ok(())
    }

    /// Run the main application loop
    pub async fn run(&self) -> Result<()> {
        loop {
            let now_local: DateTime<Local> = Local::now();
            let hour = now_local.hour();

            info!("Waktu lokal di WIB: {}", now_local.format("%Y-%m-%d %H:%M:%S"));

            // Run update
            if let Err(e) = self.start_update_log().await {
                error!("Error in update log: {}", e);
            }

            // Special handling for specific time windows
            if hour > 11 && hour < 14 {
                info!("Midday update window (12-14)");
                if let Err(e) = self.start_update_log().await {
                    error!("Error in midday update: {}", e);
                }
                sleep(Duration::from_secs(3 * 60 * 60)).await; // 3 hours
            } else if hour > 2 && hour < 5 {
                info!("Early morning update window (3-5)");
                if let Err(e) = self.start_update_log().await {
                    error!("Error in early morning update: {}", e);
                }
                sleep(Duration::from_secs(3 * 60 * 60)).await; // 3 hours
            }

            info!("-----------------------------------------------------");
            info!("APLIKASI SEDANG REHAT SELAMA 2 JAM KEDEPAN...");
            sleep(Duration::from_secs(2 * 60 * 60)).await; // 2 hours
        }
    }

    /// Run log correction
    pub async fn correct_logs(&self) -> Result<()> {
        info!("Starting log correction process");
        self.scraper.correct_logs().await?;
        info!("Log correction completed");
        Ok(())
    }
}