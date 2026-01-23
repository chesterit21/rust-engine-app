use super::{DbPool, DocumentChunk, UserDocument};
use anyhow::Result;
use pgvector::Vector;
use sqlx::{Row, FromRow};
use tracing::debug;

pub struct Repository {
    pool: DbPool,
}

impl Repository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    /// Check if user has access to document
    pub async fn check_user_document_access(
        &self,
        user_id: i32,
        document_id: i32,
    ) -> Result<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT check_user_document_access($1, $2)"
        )
        .bind(user_id)
        .bind(document_id)
        .fetch_one(self.pool.get_pool())
        .await?;
        
        Ok(result)
    }
    
    /// Get all document IDs accessible by user
    pub async fn get_user_document_ids(&self, user_id: i32) -> Result<Vec<i32>> {
        let rows = sqlx::query_scalar::<_, i32>(
            "SELECT document_id FROM get_user_document_ids($1)"
        )
        .bind(user_id)
        .fetch_all(self.pool.get_pool())
        .await?;
        
        Ok(rows)
    }
    
    /// Get user's accessible documents with metadata
    pub async fn get_user_documents(&self, user_id: i32) -> Result<Vec<UserDocument>> {
        let docs = sqlx::query_as::<_, UserDocument>(
            r#"SELECT 
                document_id,
                owner_user_id,
                document_title,
                created_at,
                user_id,
                permission_level
               FROM vw_user_documents
               WHERE user_id = $1
               ORDER BY created_at DESC"#
        )
        .bind(user_id)
        .fetch_all(self.pool.get_pool())
        .await?;
        
        Ok(docs)
    }
    
    /// Vector search dengan user authorization
    pub async fn search_user_documents(
        &self,
        user_id: i32,
        query_embedding: Vector,
        limit: i32,
        document_id: Option<i32>,
    ) -> Result<Vec<DocumentChunk>> {
        let chunks = sqlx::query_as::<_, DocumentChunk>(
            r#"SELECT 
                chunk_id,
                document_id,
                document_title,
                content,
                similarity,
                chunk_index,
                page_number
               FROM search_user_documents($1, $2, $3, $4)"#
        )
        .bind(user_id)
        .bind(query_embedding)
        .bind(limit)
        .bind(document_id)
        .fetch_all(self.pool.get_pool())
        .await?;
        
        debug!("Found {} relevant chunks for user {}", chunks.len(), user_id);
        
        Ok(chunks)
    }
    
    /// Hybrid search (vector + full-text)
    pub async fn hybrid_search_user_documents(
        &self,
        user_id: i32,
        query_embedding: Vector,
        query_text: String,
        limit: i32,
        document_id: Option<i32>,
    ) -> Result<Vec<DocumentChunk>> {
        #[derive(FromRow)]
        struct HybridResult {
            chunk_id: i64,
            document_id: i32,
            document_title: String,
            content: String,
            hybrid_score: f32,
            chunk_index: i32,
        }
        
        let results = sqlx::query_as::<_, HybridResult>(
            r#"SELECT 
                chunk_id,
                document_id,
                document_title,
                content,
                hybrid_score,
                chunk_index
               FROM hybrid_search_user_documents($1, $2, $3, $4, $5)"#
        )
        .bind(user_id)
        .bind(query_embedding)
        .bind(&query_text)
        .bind(limit)
        .bind(document_id)
        .fetch_all(self.pool.get_pool())
        .await?;
        
        // Convert to DocumentChunk
        let chunks = results
            .into_iter()
            .map(|r| DocumentChunk {
                chunk_id: r.chunk_id,
                document_id: r.document_id,
                document_title: r.document_title,
                content: r.content,
                similarity: r.hybrid_score,
                chunk_index: r.chunk_index,
                page_number: None,
            })
            .collect();
        
        Ok(chunks)
    }
    
    /// Insert uploaded document chunks (after processing)
    pub async fn insert_document_chunks(
        &self,
        document_id: i32,
        chunks: Vec<(String, Vector)>, // (content, embedding)
    ) -> Result<()> {
        let mut transaction = self.pool.get_pool().begin().await?;
        
        for (index, (content, embedding)) in chunks.into_iter().enumerate() {
            sqlx::query(
                r#"INSERT INTO rag_document_chunks 
                   (document_id, chunk_index, content, char_count, embedding)
                   VALUES ($1, $2, $3, $4, $5)"#
            )
            .bind(document_id)
            .bind(index as i32)
            .bind(&content)
            .bind(content.len() as i32)
            .bind(embedding)
            .execute(&mut *transaction)
            .await?;
        }
        
        transaction.commit().await?;
        debug!("Inserted chunks for document {}", document_id);
        
        Ok(())
    }
}
