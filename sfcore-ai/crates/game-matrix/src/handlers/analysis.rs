use axum::{
    extract::{Path, State, Query},
    response::IntoResponse,
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct AnalysisParams {
    pub window_size: Option<usize>,
}

#[derive(Deserialize)]
pub struct HistoryAnalysisParams {
    pub window_size: Option<usize>,
    pub depth: Option<usize>,
}

pub async fn get_log_analysis(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
) -> impl IntoResponse {
    match state.game_service.get_game_analysis(&game_code).await {
        Ok(results) => (StatusCode::OK, Json(results)).into_response(),
        Err(e) => {
            eprintln!("Error fetching analysis: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json::<serde_json::Value>(serde_json::json!({
                "error": e.to_string()
            }))).into_response()
        }
    }
}

pub async fn get_frequency_analysis(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    Query(params): Query<AnalysisParams>,
) -> impl IntoResponse {
    let window_size = params.window_size.unwrap_or(300);

    if window_size < 100 || window_size > 1000 {
         return (StatusCode::BAD_REQUEST, Json::<serde_json::Value>(serde_json::json!({
            "error": "Window size must be between 100 and 1000"
        }))).into_response();
    }

    match state.repositories.log_game.find_by_game_code(&game_code, window_size as u32, 0).await {
        Ok(logs) => {
            let analysis_result = state.frequency_service.analyze(&logs, window_size);
            (StatusCode::OK, Json(analysis_result)).into_response()
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json::<serde_json::Value>(serde_json::json!({
                "error": e.to_string()
            }))).into_response()
        }
    }
}

pub async fn get_history_analysis(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    Query(params): Query<HistoryAnalysisParams>,
) -> impl IntoResponse {
    // 1. Params Validation
    let window_size = params.window_size.unwrap_or(100);
    // User requested limit: 7 periods max.
    let depth = params.depth.unwrap_or(7).min(30); 

    if window_size < 50 || window_size > 500 {
         return (StatusCode::BAD_REQUEST, Json::<serde_json::Value>(serde_json::json!({
            "error": "Window size must be between 50 and 500"
        }))).into_response();
    }

    // 2. Fetch target periods (e.g., last 7 periods)
    let target_logs_result = state.repositories.log_game.find_by_game_code(&game_code, depth as u32, 0).await;
    
    let target_logs = match target_logs_result {
        Ok(logs) => logs,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json::<serde_json::Value>(serde_json::json!({"error": e.to_string()}))).into_response(),
    };

    if target_logs.is_empty() {
        return (StatusCode::OK, Json::<serde_json::Value>(serde_json::json!({
            "game_code": game_code,
            "window_size": window_size,
            "history": []
        }))).into_response();
    }

    let mut history_items = Vec::new();

    // 3. Loop through each target period for retrospective analysis
    for target in target_logs {
        let period = target.periode;
        
        // Manual query using non-macro query_as to avoid offline-mode issues and ensure column mapping
        // Select exactly the columns that LogGame expects from FromRow
        let previous_logs = state.repositories.log_game.find_previous_logs(
            &game_code,
            period,
            window_size as u32
        ).await;

        match previous_logs {
            Ok(prev_logs) => {
                 if prev_logs.len() >= 50 { // Only analyze if we have meaningful data
                     let analysis = state.frequency_service.analyze(&prev_logs, window_size);
                     let summary = state.frequency_service.generate_human_summary(&analysis);
                     
                     // Convert analysis to DTOs (PositionAnalysis, etc.)
                     use game_models::history_analysis::*;
                     
                     // Helper to extract singleton status
                     let get_status = |pos: &str| -> SinglePositionStatus {
                         let freq = analysis.results.iter().find(|r| r.position == pos && r.digit == target.get_digit(pos).unwrap_or(10)).map(|r| r.label.clone()).unwrap_or("NORMAL".to_string());
                         let cons = analysis.consistency.iter().find(|r| r.position == pos && r.digit == target.get_digit(pos).unwrap_or(10)).map(|r| r.label.clone()).unwrap_or("WEAK".to_string());
                         // Entropy is aggregate per position (not per digit), so it's same for all digits in that pos
                         let entr = analysis.entropy.iter().find(|r| r.position == pos).map(|r| r.label.clone()).unwrap_or("CUKUP MERATA".to_string());
                         
                         let consistency_text = match cons.as_str() {
                             "STRONG" => "POLA KUAT".to_string(),
                             "MEDIUM" => "POLA CUKUP KUAT".to_string(),
                             _ => "POLA LEMAH".to_string(),
                         };
                         
                         let freq_text = match freq.as_str() {
                             "OVERREPRESENTED" => "OVER".to_string(),
                             "UNDERREPRESENTED" => "UNDER".to_string(),
                             _ => "NORMAL".to_string()
                         };

                         SinglePositionStatus {
                             frequency: freq_text,
                             consistency: consistency_text,
                             entropy: entr,
                         }
                     };

                     let positions = PositionAnalysis {
                         as_pos: get_status("As"),
                         kop: get_status("Kop"),
                         kepala: get_status("Kepala"),
                         ekor: get_status("Ekor"),
                     };

                     // Pairs
                     let mut pairs_dto = Vec::new();
                     // Flatten the map
                     for (key, list) in &analysis.pairs {
                         for p in list.iter().take(2) { // Take top 2 curated
                             if p.label != "NORMAL" {
                                 pairs_dto.push(PairPatternItem {
                                     positions: key.clone(),
                                     digits: format!("{}-{}", p.digit_a, p.digit_b),
                                     status: p.label.clone() // "SANGAT SERING", etc.
                                 });
                             }
                         }
                     }
                     // Limit total pairs to 2 max
                     pairs_dto.truncate(2);

                     history_items.push(HistoryItem {
                         periode: target.periode,
                         result: target.log_result.clone(), // Use correct field name: log_result
                         analysis_timestamp: chrono::Utc::now().to_rfc3339(),
                         summary,
                         positions,
                         pairs: pairs_dto,
                     });
                 }
            }
            Err(e) => eprintln!("Failed to fetch history window for period {}: {:?}", period, e),
        }
    }

    use game_models::history_analysis::HistoryAnalysisResponse;
    (StatusCode::OK, Json(HistoryAnalysisResponse {
        game_code,
        window_size,
        history: history_items,
    })).into_response()
}

// Extension to LogGame to get digit helper
trait LogGameExt {
    fn get_digit(&self, pos: &str) -> Option<u8>;
}
impl LogGameExt for game_models::LogGame {
    fn get_digit(&self, pos: &str) -> Option<u8> {
        match pos {
            "As" => self.as_digit.map(|d| d as u8),
            "Kop" => self.kop.map(|d| d as u8),
            "Kepala" => self.kepala.map(|d| d as u8),
            "Ekor" => self.ekor.map(|d| d as u8),
            _ => None
        }
    }
}
