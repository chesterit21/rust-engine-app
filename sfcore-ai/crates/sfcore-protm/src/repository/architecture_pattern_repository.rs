use crate::domain::ArchitecturePattern;
use sqlx::SqlitePool;

pub struct ArchitecturePatternRepository;

impl ArchitecturePatternRepository {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<ArchitecturePattern>, sqlx::Error> {
        sqlx::query_as::<_, ArchitecturePattern>(
            "SELECT id, parent_id, stack_id, name, version, type, layer_rules, order_index, naming_conventions FROM architecture_patterns"
        )
        .fetch_all(pool)
        .await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM architecture_patterns WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<ArchitecturePattern>, sqlx::Error> {
        sqlx::query_as::<_, ArchitecturePattern>(
            "SELECT id, parent_id, stack_id, name, version, type, layer_rules, order_index, naming_conventions FROM architecture_patterns WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &SqlitePool, 
        id: Option<&str>,
        parent_id: Option<&str>,
        stack_id: &str,
        name: &str, 
        version: &str,
        pattern_type: &str,
        layer_rules: Option<&str>,
        order_index: i32,
        naming_conventions: Option<&str>
    ) -> Result<ArchitecturePattern, sqlx::Error> {
        let final_id = if let Some(i) = id { i.to_string() } else { uuid::Uuid::new_v4().to_string() };
        
        sqlx::query_as::<sqlx::Sqlite, ArchitecturePattern>(
            "INSERT INTO architecture_patterns (id, parent_id, stack_id, name, version, type, layer_rules, order_index, naming_conventions) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) 
             ON CONFLICT(id) DO UPDATE SET
                parent_id = excluded.parent_id,
                stack_id = excluded.stack_id,
                name = excluded.name,
                version = excluded.version,
                type = excluded.type,
                layer_rules = excluded.layer_rules,
                order_index = excluded.order_index,
                naming_conventions = excluded.naming_conventions
             RETURNING id, parent_id, stack_id, name, version, type, layer_rules, order_index, naming_conventions"
        )
        .bind(final_id)
        .bind(parent_id)
        .bind(stack_id)
        .bind(name)
        .bind(version)
        .bind(pattern_type)
        .bind(layer_rules)
        .bind(order_index)
        .bind(naming_conventions)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &SqlitePool, 
        id: &str, 
        parent_id: Option<&str>,
        stack_id: &str,
        name: &str, 
        version: &str,
        pattern_type: &str,
        layer_rules: Option<&str>,
        order_index: i32,
        naming_conventions: Option<&str>
    ) -> Result<ArchitecturePattern, sqlx::Error> {
        sqlx::query_as::<sqlx::Sqlite, ArchitecturePattern>(
            "UPDATE architecture_patterns SET parent_id = ?, stack_id = ?, name = ?, version = ?, type = ?, layer_rules = ?, order_index = ?, naming_conventions = ? WHERE id = ? 
             RETURNING id, parent_id, stack_id, name, version, type, layer_rules, order_index, naming_conventions"
        )
        .bind(parent_id)
        .bind(stack_id)
        .bind(name)
        .bind(version)
        .bind(pattern_type)
        .bind(layer_rules)
        .bind(order_index)
        .bind(naming_conventions)
        .bind(id)
        .fetch_one(pool)
        .await
    }
}
