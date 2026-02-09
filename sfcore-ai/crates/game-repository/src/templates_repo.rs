use sqlx::{SqlitePool, Error as SqlxError};
use game_models::{
    TemplateNumberTwoDigit,
    TemplateNumberTreeDigit,
    TemplateNumberFourDigit,
    CreateTemplateNumberTwoDigit,
    CreateTemplateNumberTreeDigit,
    CreateTemplateNumberFourDigit,
};

pub struct TemplatesRepository {
    pool: SqlitePool,
}

impl TemplatesRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ==================== TemplateNumberTwoDigit ====================

    pub async fn get_all_two_digit(&self) -> Result<Vec<TemplateNumberTwoDigit>, SqlxError> {
        sqlx::query_as::<_, TemplateNumberTwoDigit>(
            r#"SELECT "Id", "Numb" FROM "TemplateNumberTwoDigit" ORDER BY "Id""#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_two_digit(&self, data: &CreateTemplateNumberTwoDigit) -> Result<TemplateNumberTwoDigit, SqlxError> {
        sqlx::query_as::<_, TemplateNumberTwoDigit>(
            r#"INSERT INTO "TemplateNumberTwoDigit" ("Numb") VALUES (?1) RETURNING "Id", "Numb""#
        )
        .bind(&data.numb)
        .fetch_one(&self.pool)
        .await
    }

    // ==================== TemplateNumberTreeDigit ====================

    pub async fn get_all_tree_digit(&self) -> Result<Vec<TemplateNumberTreeDigit>, SqlxError> {
        sqlx::query_as::<_, TemplateNumberTreeDigit>(
            r#"
            SELECT "Id", "FormatNumber", "DigitTengah", "DigitBelakang", "DigitAsEkor" 
            FROM "TemplateNumberTreeDigit" 
            ORDER BY "Id"
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_tree_digit(&self, data: &CreateTemplateNumberTreeDigit) -> Result<TemplateNumberTreeDigit, SqlxError> {
        sqlx::query_as::<_, TemplateNumberTreeDigit>(
            r#"
            INSERT INTO "TemplateNumberTreeDigit" 
                ("FormatNumber", "DigitTengah", "DigitBelakang", "DigitAsEkor")
            VALUES (?1, ?2, ?3, ?4)
            RETURNING "Id", "FormatNumber", "DigitTengah", "DigitBelakang", "DigitAsEkor"
            "#
        )
        .bind(&data.format_number)
        .bind(&data.digit_tengah)
        .bind(&data.digit_belakang)
        .bind(&data.digit_as_ekor)
        .fetch_one(&self.pool)
        .await
    }

    // ==================== TemplateNumberFourDigit ====================

    pub async fn get_all_four_digit(&self) -> Result<Vec<TemplateNumberFourDigit>, SqlxError> {
        sqlx::query_as::<_, TemplateNumberFourDigit>(
            r#"
            SELECT 
                "Id", "TheNumber", "AsKop", "KopKepala", "KepalaEkor", 
                "AsKepala", "AsEkor", "KopEkor"
            FROM "TemplateNumberFourDigit" 
            ORDER BY "Id"
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_four_digit_by_number(&self, number: &str) -> Result<Option<TemplateNumberFourDigit>, SqlxError> {
        sqlx::query_as::<_, TemplateNumberFourDigit>(
            r#"
            SELECT 
                "Id", "TheNumber", "AsKop", "KopKepala", "KepalaEkor", 
                "AsKepala", "AsEkor", "KopEkor"
            FROM "TemplateNumberFourDigit" 
            WHERE "TheNumber" = ?1
            "#
        )
        .bind(number)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_four_digit_by_pattern(&self, pattern: &str) -> Result<Vec<TemplateNumberFourDigit>, SqlxError> {
        sqlx::query_as::<_, TemplateNumberFourDigit>(
            r#"
            SELECT 
                "Id", "TheNumber", "AsKop", "KopKepala", "KepalaEkor", 
                "AsKepala", "AsEkor", "KopEkor"
            FROM "TemplateNumberFourDigit" 
            WHERE "TheNumber" LIKE ?1
            ORDER BY "Id"
            "#
        )
        .bind(pattern)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_four_digit(&self, data: &CreateTemplateNumberFourDigit) -> Result<TemplateNumberFourDigit, SqlxError> {
        sqlx::query_as::<_, TemplateNumberFourDigit>(
            r#"
            INSERT INTO "TemplateNumberFourDigit" 
                ("TheNumber", "AsKop", "KopKepala", "KepalaEkor", 
                 "AsKepala", "AsEkor", "KopEkor")
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            RETURNING 
                "Id", "TheNumber", "AsKop", "KopKepala", "KepalaEkor", 
                "AsKepala", "AsEkor", "KopEkor"
            "#
        )
        .bind(&data.the_number)
        .bind(&data.as_kop)
        .bind(&data.kop_kepala)
        .bind(&data.kepala_ekor)
        .bind(&data.as_kepala)
        .bind(&data.as_ekor)
        .bind(&data.kop_ekor)
        .fetch_one(&self.pool)
        .await
    }
}