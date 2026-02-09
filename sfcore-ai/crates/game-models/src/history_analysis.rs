use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryAnalysisResponse {
    pub game_code: String,
    pub window_size: usize,
    pub history: Vec<HistoryItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryItem {
    pub periode: i64,
    pub result: String,
    pub analysis_timestamp: String,
    pub summary: String,
    pub positions: PositionAnalysis,
    pub pairs: Vec<PairPatternItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PositionAnalysis {
    pub as_pos: SinglePositionStatus, // "as" is a reserved keyword in Rust, so we use as_pos
    pub kop: SinglePositionStatus,
    pub kepala: SinglePositionStatus,
    pub ekor: SinglePositionStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SinglePositionStatus {
    pub frequency: String,   // "UNDER", "NORMAL", "OVER"
    pub consistency: String, // "POLA LEMAH", "POLA CUKUP KUAT", "POLA KUAT"
    pub entropy: String,     // "SANGAT MERATA", "CUKUP MERATA", "KURANG MERATA"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairPatternItem {
    pub positions: String,
    pub digits: String,
    pub status: String,
}
