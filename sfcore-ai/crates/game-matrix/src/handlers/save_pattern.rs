use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use crate::state::AppState;
use crate::dtos::ApiResponse;
use game_models::{CreateHistoryPlayingGame, CreatePlayingGameQueue, CreateDataPlaying, LogGame, TemplateNumberFourDigit};
use chrono::Local;

#[derive(Debug, Deserialize)]
pub struct SavePatternRequest {
    pub game_code: String,
    pub digit: String, // e.g., "1-2-3"
    pub tipe: String,  // e.g., "F Match"
}

pub async fn save_pattern_handler(
    State(data): State<AppState>,
    Json(payload): Json<SavePatternRequest>,
) -> impl IntoResponse {
    let repositories = &data.repositories;
    
    // 1. Validation
    if payload.game_code.is_empty() || payload.digit.is_empty() || payload.tipe.is_empty() {
         return (StatusCode::BAD_REQUEST, Json(ApiResponse {
            status: "error".to_string(),
            message: "All fields are required".to_string(),
            data: None::<String>,
        })).into_response();
    }

    // 2. Normalize Digit (remove dashes)
    let normalized_digit = payload.digit.replace("-", "");
    if normalized_digit.is_empty() {
         return (StatusCode::BAD_REQUEST, Json(ApiResponse {
            status: "error".to_string(),
            message: "Invalid digit format".to_string(),
            data: None::<String>,
        })).into_response();
    }
    
    // Map Tipe to TypePick (F Match -> 3DF, etc)
    let type_pick = match payload.tipe.as_str() {
        "F Match" => "3DF",
        "B Match" => "3DB",
        "AKE Match" => "3DAKE",
        "AKpE Match" => "3DAKpE",
        "2DF" => "2DF",
        "2DM" => "2DM",
        "2DB" => "2DB",
        "2DAKp" => "2DAKp",
        "2DKE" => "2DKE",
        "2DAE" => "2DAE",
        _ => {
            return (StatusCode::BAD_REQUEST, Json(ApiResponse {
                status: "error".to_string(),
                message: format!("Invalid or Unknown Tipe: {}", payload.tipe),
                data: None::<String>,
            })).into_response();
        }
    }.to_string();

    let created_date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let created_by = "Owner-App".to_string(); // Default

    // 3. Mapping Pattern (Tipe -> Pattern Matches)
    let d: Vec<char> = normalized_digit.chars().collect();
    let pattern_query = match payload.tipe.as_str() {
        "F Match" | "B Match" | "AKE Match" | "AKpE Match" => {
            if normalized_digit.len() != 3 {
                 return (StatusCode::BAD_REQUEST, Json(ApiResponse {
                    status: "error".to_string(),
                    message: "Digit must be 3 numbers for Pattern Match".to_string(),
                    data: None::<String>,
                })).into_response();
            }
             match payload.tipe.as_str() {
                "F Match" => format!("{}{}{}%", d[0], d[1], d[2]), // "357%"
                "B Match" => format!("%{}{}{}", d[0], d[1], d[2]), // "%357"
                "AKE Match" => format!("{}{}%{}", d[0], d[1], d[2]), // "35%7"
                "AKpE Match" => format!("{}%{}{}", d[0], d[1], d[2]), // "3%57"
                _ => "".to_string(),
            }
        },
        "2DF" | "2DM" | "2DB" | "2DAKp" | "2DKE" | "2DAE" => {
             if normalized_digit.len() != 2 {
                 return (StatusCode::BAD_REQUEST, Json(ApiResponse {
                    status: "error".to_string(),
                    message: "Digit must be 2 numbers for Composition Match".to_string(),
                    data: None::<String>,
                })).into_response();
            }
            match payload.tipe.as_str() {
                "2DF" => format!("{}{}%", d[0], d[1]), // AsKop "06%"
                "2DM" => format!("_{}{}_", d[0], d[1]), // KopKepala "_06_"
                "2DB" => format!("%{}{}", d[0], d[1]), // KepalaEkor "%06"
                "2DAKp" => format!("{}_{}_", d[0], d[1]), // AsKepala "0_6_"
                "2DKE" => format!("_{}_{}", d[0], d[1]), // KopEkor "_0_6"
                "2DAE" => format!("{}__{}", d[0], d[1]), // AsEkor "0__6"
                _ => "".to_string(),
            }
        },
        _ => "".to_string(),
    };
    
    // 4. Fetch Matching Templates
    let templates: Vec<TemplateNumberFourDigit> = if !pattern_query.is_empty() {
        match repositories.templates.find_four_digit_by_pattern(&pattern_query).await {
            Ok(tmpls) => tmpls,
            Err(e) => {
                 return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
                    status: "error".to_string(),
                    message: format!("Database error fetching templates: {}", e),
                    data: None::<String>,
                })).into_response();
            }
        }
    } else {
         return (StatusCode::BAD_REQUEST, Json(ApiResponse {
            status: "error".to_string(),
            message: "Invalid Pattern Type".to_string(),
            data: None::<String>,
        })).into_response();
    };

    if templates.is_empty() {
         return (StatusCode::NOT_FOUND, Json(ApiResponse {
            status: "error".to_string(),
            message: "No matching templates found for this pattern".to_string(),
            data: None::<String>,
        })).into_response();
    }

    // 5. GameId Lookup
    let game_id = match repositories.master_game.find_by_game_code(&payload.game_code).await {
        Ok(Some(game)) => game.id,
        Ok(None) => {
             return (StatusCode::NOT_FOUND, Json(ApiResponse {
                status: "error".to_string(),
                message: "GameCode not found".to_string(),
                data: None::<String>,
            })).into_response();
        },
        Err(e) => {
             return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
                status: "error".to_string(),
                message: format!("Database error: {}", e),
                data: None::<String>,
            })).into_response();
        }
    };
    
    // Fetch latest log for context
    let latest_log = match repositories.log_game.find_latest_by_game_code(&payload.game_code).await {
        Ok(Some(log)) => log,
        _ => {
            LogGame {
                 id: 0, 
                 periode: 0, 
                 log_result: "0000".to_string(), 
                 as_digit: None, kop: None, kepala: None, ekor: None,
                 created_date: None, date_result_in_game: None, game_code: payload.game_code.clone()
            }
        }
    };
    
    // Transcode base
    let trans_code_base = format!("{}{}{}", payload.game_code, latest_log.periode, latest_log.log_result);

    // 6. Insert into HistoryPlayingGame (Looping)
    for tmpl in templates {
        let history_data = CreateHistoryPlayingGame {
            trans_code: trans_code_base.clone(),
            game_id,
            game_code: payload.game_code.clone(),
            created_by: created_by.clone(),
            created_date: created_date.clone(),
            template_number_id: tmpl.id,
            type_pick: type_pick.clone(),
            number: normalized_digit.clone(), 
        };
        
        if let Err(e) = repositories.history_playing_game.create(&history_data).await {
             return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
                status: "error".to_string(),
                message: format!("Failed to create history for id {}: {}", tmpl.id, e),
                data: None::<String>,
            })).into_response();
        }
    }

    // 7. Insert into PlayingGameQueue (ONCE per request, CHECK EXISTENCE)
    let queue_exists = match repositories.playing_game_queue.find_by_trans_code(&trans_code_base).await {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
               status: "error".to_string(),
               message: format!("Database error checking queue: {}", e),
               data: None::<String>,
           })).into_response();
        }
    };

    if !queue_exists {
        let queue_data = CreatePlayingGameQueue {
            game_id,
            game_code: payload.game_code.clone(),
            trans_code: trans_code_base.clone(),
            created_by: created_by.clone(),
            created_date: Some(created_date.clone()),
        };

        if let Err(e) = repositories.playing_game_queue.create(&queue_data).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
               status: "error".to_string(),
               message: format!("Failed to create queue: {}", e),
               data: None::<String>,
           })).into_response();
       }
    }

    // 8. Insert into DataPlaying (ONCE per request)
    let playing_data = CreateDataPlaying {
        game_code: payload.game_code.clone(),
        digit: normalized_digit, 
        tipe: type_pick,
    };
    
    if let Err(e) = repositories.data_playing.create(&playing_data).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
           status: "error".to_string(),
           message: format!("Failed to create data playing: {}", e),
           data: None::<String>,
       })).into_response();
    }

    (StatusCode::OK, Json(ApiResponse {
        status: "success".to_string(),
        message: "Pattern match saved successfully".to_string(),
        data: None::<String>,
    })).into_response()
}
