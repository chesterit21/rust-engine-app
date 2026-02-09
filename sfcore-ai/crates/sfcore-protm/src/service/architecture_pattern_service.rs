use crate::repository::ArchitecturePatternRepository;
use crate::domain::ArchitecturePattern;
use sqlx::SqlitePool;

pub struct ArchitecturePatternService;

impl ArchitecturePatternService {
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<ArchitecturePattern>, sqlx::Error> {
        ArchitecturePatternRepository::find_all(pool).await
    }

    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
        ArchitecturePatternRepository::delete(pool, id).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<ArchitecturePattern>, sqlx::Error> {
        ArchitecturePatternRepository::find_by_id(pool, id).await
    }

    pub async fn create(
        pool: &SqlitePool, 
        id: Option<String>,
        parent_id: Option<String>,
        stack_id: String,
        name: String, 
        version: String,
        pattern_type: String,
        layer_rules: Option<String>,
        order_index: i32,
        naming_conventions: Option<String>
    ) -> Result<ArchitecturePattern, sqlx::Error> {
        ArchitecturePatternRepository::create(
            pool, 
            id.as_deref(),
            parent_id.as_deref(), 
            &stack_id, 
            &name, 
            &version, 
            &pattern_type, 
            layer_rules.as_deref(), 
            order_index, 
            naming_conventions.as_deref()
        ).await
    }

    pub async fn update(
        pool: &SqlitePool, 
        id: &String, 
        parent_id: Option<&str>,
        stack_id: &String,
        name: &String, 
        version: &String,
        pattern_type: &String,
        layer_rules: Option<&str>,
        order_index: i32,
        naming_conventions: Option<&str>
    ) -> Result<ArchitecturePattern, sqlx::Error> {
        ArchitecturePatternRepository::update(
            pool, 
            id, 
            parent_id, 
            stack_id, 
            name, 
            version, 
            pattern_type, 
            layer_rules, 
            order_index, 
            naming_conventions
        ).await
    }
}
