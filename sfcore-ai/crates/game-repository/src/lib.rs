//! Game Repository Layer
//! 
//! Data access layer - handles all database operations.

mod master_game_repo;
mod log_game_repo;
mod history_playing_game_repo;
mod pattern_hit_repo;
mod setup_link_game_repo;
mod site_master_repo;
mod member_game_repo;
mod templates_repo;
mod data_playing_repo;
mod playing_game_queue_repo;

// Public re-exports
pub use master_game_repo::MasterGameRepository;

pub use log_game_repo::LogGameRepository;
pub use history_playing_game_repo::HistoryPlayingGameRepository;
pub use pattern_hit_repo::PatternHitRepository;
pub use setup_link_game_repo::SetupLinkGameRepository;
pub use site_master_repo::SiteMasterRepository;
pub use member_game_repo::MemberGameRepository;
pub use templates_repo::TemplatesRepository;
pub use data_playing_repo::DataPlayingRepository;
pub use playing_game_queue_repo::PlayingGameQueueRepository;

// Re-export SQLx types
pub use sqlx;
pub use sqlx::SqlitePool;

/// Repository initialization helper
pub struct Repositories {
    pub master_game: MasterGameRepository,
    pub log_game: LogGameRepository,
    pub history_playing_game: HistoryPlayingGameRepository,
    pub pattern_hit: PatternHitRepository,
    pub setup_link_game: SetupLinkGameRepository,
    pub site_master: SiteMasterRepository,
    pub member_game: MemberGameRepository,
    pub templates: TemplatesRepository,
    pub data_playing: DataPlayingRepository,
    pub playing_game_queue: PlayingGameQueueRepository,
}

impl Repositories {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            master_game: MasterGameRepository::new(pool.clone()),
            log_game: LogGameRepository::new(pool.clone()),
            history_playing_game: HistoryPlayingGameRepository::new(pool.clone()),
            pattern_hit: PatternHitRepository::new(pool.clone()),
            setup_link_game: SetupLinkGameRepository::new(pool.clone()),
            site_master: SiteMasterRepository::new(pool.clone()),
            member_game: MemberGameRepository::new(pool.clone()),
            templates: TemplatesRepository::new(pool.clone()),
            data_playing: DataPlayingRepository::new(pool.clone()),
            playing_game_queue: PlayingGameQueueRepository::new(pool),
        }
    }

    /// Ensure critical indexes exist for performance
    pub async fn create_indexes(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        // Index for Frequency Analysis & History (GameCode + Periode DESC)
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_game_periode ON LogGame("GameCode", "Periode" DESC);
            "#
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }
}


/// Database connection helper with WAL mode
pub async fn connect_with_wal_mode(
    database_url: &str,
) -> Result<SqlitePool, sqlx::Error> {
    use sqlx::sqlite::SqliteConnectOptions;
    use std::time::Duration;

    SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(database_url)
            .create_if_missing(false)
            .busy_timeout(Duration::from_secs(10))
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal),
    )
    .await
}