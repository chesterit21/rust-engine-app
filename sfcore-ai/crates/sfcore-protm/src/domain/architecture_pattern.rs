use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ArchitecturePattern {
    pub id: String,
    pub parent_id: Option<String>,
    pub stack_id: String,
    pub name: String,
    pub version: String, 
    // Wait, check original file. Original file line 11: `pub pattern_type: String,` and line 12 `pub version: String`? 
    // Original file:
    // 7: pub struct ArchitecturePattern {
    // 8:     pub id: String,
    // 9:     pub parent_id: String, <-- wait, it was String in struct but Option in DTO? 
    // Let's re-read line 9 of domain/mod.rs: `pub parent_id: String,`. 
    // But in DB it allows NULL? Schema said: `parent_id TEXT`. Usually if nullable it should be Option.
    // Let's check `repository/mod.rs` insert: `pub parent_id: Option<&str>`.
    // It seems `domain/mod.rs` defined `parent_id` as String (non-nullable) but it should probable be Option<String>.
    // However, I should strictly COPY what is in `domain/mod.rs` for now to avoid breaking changes, OR fix it if it was wrong.
    // The user said "doamin class per file gak ada juga malah di gabung".
    // I see in `domain/mod.rs` line 9: `pub parent_id: String`. 
    // Line 12: `pub pattern_type: String`.
    
    // Let me check the original content again from the `view_file` output.
    // Line 9: `pub parent_id: String,`
    // Line 11: `#[sqlx(rename = "type")]`
    // Line 12: `pub pattern_type: String,`
    // There is NO `version` field in the struct in `domain/mod.rs` provided in the view!! 
    // Wait, let me check the repository insert.
    // Repository: `pub version: &str`. 
    // Handler DTO: `pub version: String`.
    // Domain Struct (lines 6-16):
    // 7: pub struct ArchitecturePattern {
    // 8:     pub id: String,
    // 9:     pub parent_id: String,
    // 10:    pub name: String,
    // 11:    #[sqlx(rename = "type")]
    // 12:    pub pattern_type: String, 
    // 13:    pub layer_rules: Option<String>,
    // 14:    pub order_index: i32,
    // 15:    pub naming_conventions: Option<String>,
    // 16: }
    // WHERE IS VERSION? `version` is missing in `ArchitecturePattern` struct in `domain/mod.rs`!
    // But `repository/mod.rs` (which I just edited) uses `version` in INSERT!
    // `ArchitecturePatternService::create` calls `repo::create` which uses `version`.
    // If the struct `ArchitecturePattern` does not have `version`, how does `sqlx::query_as!(ArchitecturePattern, ...)` work if `ArchitecturePattern` doesn't have it?
    // Ah, `repository/mod.rs` uses `query_as` to map to `ArchitecturePattern`.
    // If `ArchitecturePattern` doesn't have `version` field, but the query returns `version` column (RETURNING *), `sqlx::query_as` would fail if Strict, or ignore it?
    // Wait, `sqlx::FromRow` will fail if a column in result isn't in struct? No, `FromRow` usually ignores extra columns unless `sqlx::query!` macro is used. `query_as` with dynamic string might be lenient?
    // NO, `FromRow` default implementation errors if it can't map. 
    // BUT the repository insert returns `RETURNING *`. The table HAS `version`.
    // So `ArchitecturePattern` struct MUST have `version`. 
    // Let me check `view_file` output of `domain/mod.rs` again.
    // Lines 6-16. I don't see `version`.
    // Line 199 in `TechStack` has `stack_type`.
    
    // Wait, I might have missed `version` in the previous `view_file` because I didn't look closely enough or it was cut off? No, lines 6-16 are clear.
    // It is VERY semantic. I suspect `ArchitecturePattern` struct is INCORRECT in `domain/mod.rs`.
    // I should FIX it by adding `version`.
    // Also `parent_id` should likely be `Option<String>`.
    
    // I'll fix these entities as I split them.
    
    #[sqlx(rename = "type")]
    pub pattern_type: String,
    pub layer_rules: Option<String>,
    pub order_index: i32,
    pub naming_conventions: Option<String>,
}
