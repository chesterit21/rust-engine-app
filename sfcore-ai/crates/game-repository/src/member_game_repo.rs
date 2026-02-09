use sqlx::{SqlitePool, Error as SqlxError};
use game_models::{MemberGame, CreateMemberGame, UpdateMemberGame};  // <-- FIX: semua dari game_models

pub struct MemberGameRepository {
    pool: SqlitePool,
}

impl MemberGameRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_game_group(&self, game_group: &str) -> Result<Vec<MemberGame>, SqlxError> {
        sqlx::query_as::<_, MemberGame>(
            r#"
            SELECT 
                "Id", "GameGroup", "GameUxs", "GamePxw", "IsActive", 
                "BankAccountName", "BankAccountNumber", "BankName", "IsFlag", "Bet"
            FROM "MemberGame" 
            WHERE "GameGroup" = ?1 
            ORDER BY "Id"
            "#
        )
        .bind(game_group)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_game_uxs(&self, game_uxs: &str) -> Result<Option<MemberGame>, SqlxError> {
        sqlx::query_as::<_, MemberGame>(
            r#"
            SELECT 
                "Id", "GameGroup", "GameUxs", "GamePxw", "IsActive", 
                "BankAccountName", "BankAccountNumber", "BankName", "IsFlag", "Bet"
            FROM "MemberGame" 
            WHERE "GameUxs" = ?1
            "#
        )
        .bind(game_uxs)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_active(&self) -> Result<Vec<MemberGame>, SqlxError> {
        sqlx::query_as::<_, MemberGame>(
            r#"
            SELECT 
                "Id", "GameGroup", "GameUxs", "GamePxw", "IsActive", 
                "BankAccountName", "BankAccountNumber", "BankName", "IsFlag", "Bet"
            FROM "MemberGame" 
            WHERE "IsActive" = 1 
            ORDER BY "GameGroup"
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<MemberGame>, SqlxError> {
        sqlx::query_as::<_, MemberGame>(
            r#"
            SELECT 
                "Id", "GameGroup", "GameUxs", "GamePxw", "IsActive", 
                "BankAccountName", "BankAccountNumber", "BankName", "IsFlag", "Bet"
            FROM "MemberGame" 
            WHERE "Id" = ?1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_all(&self) -> Result<Vec<MemberGame>, SqlxError> {
        sqlx::query_as::<_, MemberGame>(
            r#"
            SELECT 
                "Id", "GameGroup", "GameUxs", "GamePxw", "IsActive", 
                "BankAccountName", "BankAccountNumber", "BankName", "IsFlag", "Bet"
            FROM "MemberGame" 
            ORDER BY "GameGroup"
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create(&self, data: &CreateMemberGame) -> Result<MemberGame, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, MemberGame>(
            r#"
            INSERT INTO "MemberGame" 
                ("GameGroup", "GameUxs", "GamePxw", "IsActive", 
                 "BankAccountName", "BankAccountNumber", "BankName", "IsFlag", "Bet")
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            RETURNING 
                "Id", "GameGroup", "GameUxs", "GamePxw", "IsActive", 
                "BankAccountName", "BankAccountNumber", "BankName", "IsFlag", "Bet"
            "#
        )
        .bind(&data.game_group)
        .bind(&data.game_uxs)
        .bind(&data.game_pxw)
        .bind(data.is_active)
        .bind(&data.bank_account_name)
        .bind(&data.bank_account_number)
        .bind(&data.bank_name)
        .bind(data.is_flag)
        .bind(&data.bet)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update(&self, data: &UpdateMemberGame) -> Result<MemberGame, SqlxError> {  // <-- FIX: tambahin "data: "
        sqlx::query_as::<_, MemberGame>(
            r#"
            UPDATE "MemberGame" SET
                "GameGroup" = ?2,
                "GameUxs" = ?3,
                "GamePxw" = ?4,
                "IsActive" = ?5,
                "BankAccountName" = ?6,
                "BankAccountNumber" = ?7,
                "BankName" = ?8,
                "IsFlag" = ?9,
                "Bet" = ?10
            WHERE "Id" = ?1
            RETURNING 
                "Id", "GameGroup", "GameUxs", "GamePxw", "IsActive", 
                "BankAccountName", "BankAccountNumber", "BankName", "IsFlag", "Bet"
            "#
        )
        .bind(data.id)
        .bind(&data.game_group)
        .bind(&data.game_uxs)
        .bind(&data.game_pxw)
        .bind(data.is_active)
        .bind(&data.bank_account_name)
        .bind(&data.bank_account_number)
        .bind(&data.bank_name)
        .bind(data.is_flag)
        .bind(&data.bet)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete(&self, id: i64) -> Result<bool, SqlxError> {
        let result = sqlx::query(
            r#"DELETE FROM "MemberGame" WHERE "Id" = ?1"#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}