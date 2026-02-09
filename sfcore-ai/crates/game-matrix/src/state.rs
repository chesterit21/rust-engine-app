use std::sync::Arc;
use sqlx::SqlitePool;
use game_repository::{
    MasterGameRepository,
    LogGameRepository,
    HistoryPlayingGameRepository,
    PatternHitRepository,
    MemberGameRepository,
    SiteMasterRepository,
    SetupLinkGameRepository,
    Repositories, // Import Repositories struct
};
use crate::services::{GameService, PlayService, FrequencyService, SetupService};

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub db_pool: SqlitePool,
    pub game_service: Arc<GameService>,
    pub play_service: Arc<PlayService>,
    pub frequency_service: Arc<FrequencyService>,
    pub setup_service: Arc<SetupService>,
    pub repositories: Arc<Repositories>, // Add repositories field
}

impl AppState {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Setup SQLite pool dengan WAL mode + busy timeout
        let pool = SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(database_url)
                .create_if_missing(true)
                .busy_timeout(std::time::Duration::from_secs(10)),
        )
        .await?;

        // Init repositories via Repositories struct
        let repositories = Arc::new(Repositories::new(pool.clone()));

        // Create separate repo instances for services if needed OR reuse from repositories wrapper?
        // Services currently take individual repos.
        // For consistency with existing code, let's keep services taking specific repos, 
        // BUT we also store the `repositories` wrapper for handlers that need direct access.
        
        let master_repo_1 = MasterGameRepository::new(pool.clone());
        let log_repo_1 = LogGameRepository::new(pool.clone());
        
        let master_repo_2 = MasterGameRepository::new(pool.clone());
        let log_repo_2 = LogGameRepository::new(pool.clone());
        let history_repo = HistoryPlayingGameRepository::new(pool.clone());
        let pattern_repo = PatternHitRepository::new(pool.clone());

        // Init services
        // Init services
        let game_service = Arc::new(GameService::new(master_repo_1, log_repo_1));
        let play_service = Arc::new(PlayService::new(
            master_repo_2,
            log_repo_2,
            history_repo,
            pattern_repo,
        ));
        let setup_service = Arc::new(SetupService::new(
            MasterGameRepository::new(pool.clone()),
            MemberGameRepository::new(pool.clone()),
            SiteMasterRepository::new(pool.clone()),
            SetupLinkGameRepository::new(pool.clone()),
        ));
        let frequency_service = Arc::new(FrequencyService::new());

        Ok(Self {
            db_pool: pool,
            game_service,
            play_service,
            frequency_service,
            setup_service,
            repositories,
        })
    }
}