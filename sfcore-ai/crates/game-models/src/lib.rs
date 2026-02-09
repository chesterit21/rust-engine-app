//! Game Models
//! 
//! Pure entity definitions - no business logic, no database operations.

pub mod master_game;
pub mod log_game;
pub mod history_playing_game;
pub mod pattern_hit;
pub mod setup_link_game;
pub mod site_master;
pub mod member_game;
pub mod history_analysis;
pub mod templates;

// Public re-exports - SEMUA type harus di-re-export biar bisa di-import dari root
pub use history_analysis::{
    HistoryAnalysisResponse,
    HistoryItem,
    PositionAnalysis,
    SinglePositionStatus,
    PairPatternItem,
};
pub use master_game::{
    MasterGame,
    CreateMasterGame,
    UpdateMasterGame,
};
pub use log_game::{
    LogGame,
    CreateLogGame,
    LogGameFilter,
    LogAnalysisResult,
};
pub use history_playing_game::{
    HistoryPlayingGame,
    CreateHistoryPlayingGame,
    HistoryPlayingGameFilter,
    HistorySummary,
};
pub use pattern_hit::{
    PatternHitGameQueue,
    PatternHitGameHistory,
    TemplatePatternHitGame,
    CreatePatternHitGameQueue,
    CreatePatternHitGameHistory,
    CreateTemplatePatternHitGame,
};
pub use setup_link_game::{
    SetupLinkGame,
    CreateSetupLinkGame,
    UpdateSetupLinkGame,  // <-- tambahin ini
};
pub use site_master::{
    SiteMaster,
    CreateSiteMaster,
    UpdateSiteMaster,     // <-- tambahin ini
};
pub use member_game::{
    MemberGame,
    CreateMemberGame,
    UpdateMemberGame,     // <-- tambahin ini
};
pub use templates::{
    TemplateNumberTwoDigit,
    TemplateNumberTreeDigit,
    TemplateNumberFourDigit,
    CreateTemplateNumberTwoDigit,
    CreateTemplateNumberTreeDigit,
    CreateTemplateNumberFourDigit,
};

pub mod playing_game_queue;
pub mod data_playing;

pub use playing_game_queue::{
    PlayingGameQueue,
    CreatePlayingGameQueue,
};
pub use data_playing::{
    DataPlaying,
    CreateDataPlaying,
};

// Common imports
pub use serde;
pub use chrono;