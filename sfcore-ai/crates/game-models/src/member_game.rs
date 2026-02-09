use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Member game settings - "MemberGame" table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MemberGame {
    #[sqlx(rename = "Id")]
    pub id: i64,

    #[sqlx(rename = "GameGroup")]
    pub game_group: String,

    #[sqlx(rename = "GameUxs")]
    pub game_uxs: String,

    #[sqlx(rename = "GamePxw")]
    pub game_pxw: String,

    #[sqlx(rename = "IsActive")]
    pub is_active: Option<i64>,

    #[sqlx(rename = "BankAccountName")]
    pub bank_account_name: Option<String>,

    #[sqlx(rename = "BankAccountNumber")]
    pub bank_account_number: Option<String>,

    #[sqlx(rename = "BankName")]
    pub bank_name: Option<String>,

    #[sqlx(rename = "IsFlag")]
    pub is_flag: Option<i64>,

    #[sqlx(rename = "Bet")]
    pub bet: Option<String>,
}

/// Create MemberGame input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemberGame {
    pub game_group: String,
    pub game_uxs: String,
    pub game_pxw: String,
    pub is_active: Option<i64>,
    pub bank_account_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_name: Option<String>,
    pub is_flag: Option<i64>,
    pub bet: Option<String>,
}

/// Update MemberGame input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemberGame {
    pub id: i64,
    pub game_group: String,
    pub game_uxs: String,
    pub game_pxw: String,
    pub is_active: Option<i64>,
    pub bank_account_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_name: Option<String>,
    pub is_flag: Option<i64>,
    pub bet: Option<String>,
}