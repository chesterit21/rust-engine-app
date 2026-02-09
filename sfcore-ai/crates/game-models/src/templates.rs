use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Template Number Two Digit - "TemplateNumberTwoDigit" table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplateNumberTwoDigit {
    #[sqlx(rename = "Id")]
    pub id: i64,

    #[sqlx(rename = "Numb")]
    pub numb: String,
}

/// Template Number Three Digit - "TemplateNumberTreeDigit" table
/// Note: Table name has typo "TreeDigit" instead of "ThreeDigit"
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplateNumberTreeDigit {
    #[sqlx(rename = "Id")]
    pub id: i64,

    #[sqlx(rename = "FormatNumber")]
    pub format_number: String,

    #[sqlx(rename = "DigitTengah")]
    pub digit_tengah: String,

    #[sqlx(rename = "DigitBelakang")]
    pub digit_belakang: String,

    #[sqlx(rename = "DigitAsEkor")]
    pub digit_as_ekor: String,
}

/// Template Number Four Digit - "TemplateNumberFourDigit" table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplateNumberFourDigit {
    #[sqlx(rename = "Id")]
    pub id: i64,

    #[sqlx(rename = "TheNumber")]
    pub the_number: String,

    #[sqlx(rename = "AsKop")]
    pub as_kop: String,

    #[sqlx(rename = "KopKepala")]
    pub kop_kepala: String,

    #[sqlx(rename = "KepalaEkor")]
    pub kepala_ekor: String,

    #[sqlx(rename = "AsKepala")]
    pub as_kepala: String,

    #[sqlx(rename = "AsEkor")]
    pub as_ekor: String,

    #[sqlx(rename = "KopEkor")]
    pub kop_ekor: String,
}

/// Create TemplateNumberTwoDigit input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateNumberTwoDigit {
    pub numb: String,
}

/// Create TemplateNumberTreeDigit input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateNumberTreeDigit {
    pub format_number: String,
    pub digit_tengah: String,
    pub digit_belakang: String,
    pub digit_as_ekor: String,
}

/// Create TemplateNumberFourDigit input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateNumberFourDigit {
    pub the_number: String,
    pub as_kop: String,
    pub kop_kepala: String,
    pub kepala_ekor: String,
    pub as_kepala: String,
    pub as_ekor: String,
    pub kop_ekor: String,
}

/// Dashboard Game Result DTO (Custom Query Result)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DashboardGameResult {
    #[sqlx(rename = "GameCode")]
    pub game_code: String,

    #[sqlx(rename = "Periode")]
    pub periode: Option<i64>,

    #[sqlx(rename = "GameHour")]
    pub game_hour: Option<i64>,

    #[sqlx(rename = "GameMinute")]
    pub game_minute: Option<i64>,

    #[sqlx(rename = "DateResult")]
    pub date_result: Option<String>,

    #[sqlx(rename = "LastResult")]
    pub last_result: Option<String>,

    #[sqlx(rename = "Holiday")]
    pub holiday: Option<String>,

    #[sqlx(rename = "InputResultDate")]
    pub input_result_date: Option<String>,
    
    // Virtual field for Trend ("UP" or "DOWN"), not from DB
    #[sqlx(default)] 
    pub trend: Option<String>,
}