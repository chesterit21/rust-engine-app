use super::{DbPool, DocumentChunk, DocumentFile, IngestionLog, IngestionStatus};
use anyhow::Result;

use sqlx::Row;
use tracing::debug;

pub struct Repository {
    pool: DbPool,
}

impl Repository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    // ==================== Document Files ====================
    
    pub async fn get_document_file(&self, document_id: i32) -> Result<Option<DocumentFile>> {
        let result = sqlx::query_as::<_, DocumentFile>(
            r#"SELECT "DocumentID", "DocumentFilePath" 
               FROM "TblDocumentFiles" 
               WHERE "DocumentID" = $1"#
        )
        .bind(document_id)
        .fetch_optional(self.pool.get_pool())
        .await?;
        
        Ok(result)
    }
    
    pub async fn get_all_document_files(&self) -> Result<Vec<DocumentFile>> {
        let results = sqlx::query_as::<_, DocumentFile>(
            r#"SELECT "DocumentID", "DocumentFilePath" 
               FROM "TblDocumentFiles"
               ORDER BY "DocumentID""#
        )
        .fetch_all(self.pool.get_pool())
        .await?;
        
        Ok(results)
    }
    
    // ==================== Chunks ====================
    
    pub async fn insert_chunks(&self, chunks: Vec<DocumentChunk>) -> Result<()> {
        if chunks.is_empty() {
            return Ok(());
        }
        
        let chunk_count = chunks.len();
        let mut transaction = self.pool.get_pool().begin().await?;
        
        for chunk in chunks {
            sqlx::query(
                r#"INSERT INTO rag_document_chunks 
                   (document_id, tenant_id, chunk_index, content, char_count, 
                    token_count, embedding, page_number, section, tags)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                   ON CONFLICT (document_id, chunk_index) 
                   DO UPDATE SET 
                       content = EXCLUDED.content,
                       char_count = EXCLUDED.char_count,
                       token_count = EXCLUDED.token_count,
                       embedding = EXCLUDED.embedding,
                       page_number = EXCLUDED.page_number,
                       section = EXCLUDED.section,
                       tags = EXCLUDED.tags,
                       updated_at = now()"#
            )
            .bind(chunk.document_id)
            .bind(chunk.tenant_id)
            .bind(chunk.chunk_index)
            .bind(&chunk.content)
            .bind(chunk.char_count)
            .bind(chunk.token_count)
            .bind(chunk.embedding)
            .bind(chunk.page_number)
            .bind(chunk.section)
            .bind(chunk.tags)
            .execute(&mut *transaction)
            .await?;
        }
        
        transaction.commit().await?;
        debug!("Inserted {} chunks", chunk_count);
        
        Ok(())
    }
    
    pub async fn delete_chunks_by_document(&self, document_id: i32) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM rag_document_chunks WHERE document_id = $1"
        )
        .bind(document_id)
        .execute(self.pool.get_pool())
        .await?;
        
        Ok(result.rows_affected())
    }
    
    pub async fn count_chunks_by_document(&self, document_id: i32) -> Result<i64> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM rag_document_chunks WHERE document_id = $1"
        )
        .bind(document_id)
        .fetch_one(self.pool.get_pool())
        .await?;
        
        Ok(row.get("count"))
    }
    
    // ==================== Ingestion Log ====================
    
    pub async fn upsert_ingestion_log(&self, log: &IngestionLog) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO rag_ingestion_log 
               (document_id, file_path, file_size, file_type, 
                embedding_model, chunk_size, chunk_overlap, status,
                total_chunks, processed_chunks, last_error, retry_count,
                started_at, processed_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               ON CONFLICT (document_id) 
               DO UPDATE SET 
                       file_path = EXCLUDED.file_path,
                   file_size = EXCLUDED.file_size,
                   file_type = EXCLUDED.file_type,
                   embedding_model = EXCLUDED.embedding_model,
                   chunk_size = EXCLUDED.chunk_size,
                   chunk_overlap = EXCLUDED.chunk_overlap,
                   status = EXCLUDED.status,
                   total_chunks = EXCLUDED.total_chunks,
                   processed_chunks = EXCLUDED.processed_chunks,
                   last_error = EXCLUDED.last_error,
                   retry_count = EXCLUDED.retry_count,
                   started_at = COALESCE(EXCLUDED.started_at, rag_ingestion_log.started_at),
                   processed_at = EXCLUDED.processed_at,
                   updated_at = now()"#
        )
        .bind(log.document_id)
        .bind(&log.file_path)
        .bind(log.file_size)
        .bind(&log.file_type)
        .bind(&log.embedding_model)
        .bind(log.chunk_size)
        .bind(log.chunk_overlap)
        .bind(&log.status)
        .bind(log.total_chunks)
        .bind(log.processed_chunks)
        .bind(&log.last_error)
        .bind(log.retry_count)
        .bind(log.started_at)
        .bind(log.processed_at)
        .execute(self.pool.get_pool())
        .await?;
        
        Ok(())
    }
    
    pub async fn get_ingestion_log(&self, document_id: i32) -> Result<Option<IngestionLog>> {
        let result = sqlx::query_as::<_, IngestionLog>(
            "SELECT * FROM rag_ingestion_log WHERE document_id = $1"
        )
        .bind(document_id)
        .fetch_optional(self.pool.get_pool())
        .await?;
        
        Ok(result)
    }
    
    pub async fn update_ingestion_status(
        &self,
        document_id: i32,
        status: IngestionStatus,
        error: Option<String>
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE rag_ingestion_log 
               SET status = $2, 
                   last_error = $3,
                   processed_at = CASE WHEN $2 IN ('completed', 'failed') THEN now() ELSE processed_at END,
                   updated_at = now()
               WHERE document_id = $1"#
        )
        .bind(document_id)
        .bind(status.to_string())
        .bind(error)
        .execute(self.pool.get_pool())
        .await?;
        
        Ok(())
    }
}
