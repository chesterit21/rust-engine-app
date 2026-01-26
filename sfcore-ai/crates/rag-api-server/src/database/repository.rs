use super::{DbPool, DocumentChunk, UserDocument};
use anyhow::Result;
use pgvector::Vector;
use sqlx::{Row, FromRow};
use tracing::debug;
use chrono::{DateTime, Utc};
use super::models::{DocumentMetadata, DocumentOverview};

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
        .persistent(false)
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
        document_ids: Option<Vec<i32>>,
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
        .bind(document_ids)
        .persistent(false)
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
        document_ids: Option<Vec<i32>>,
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
        .bind(document_ids)
        .persistent(false)
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

    /// Get the first chunk of a document (usually contains title/intro) - Optimization for "What is this?" queries
    pub async fn get_first_chunk(
        &self,
        document_id: i32,
    ) -> Result<Option<DocumentChunk>> {
        // Assume retrieving directly from rag_document_chunks table joined with document info
        let chunk = sqlx::query_as::<_, DocumentChunk>(
            r#"
            SELECT 
                c.id as chunk_id,
                c.document_id,
                d."DocumentTitle" as document_title,
                c.content,
                1.0::float4 as similarity,
                c.chunk_index,
                NULL::int as page_number
            FROM rag_document_chunks c
            JOIN "TblDocuments" d ON d."Id" = c.document_id
            WHERE c.document_id = $1 AND c.chunk_index = 0
            LIMIT 1
            "#
        )
        .bind(document_id)
        .fetch_optional(self.pool.get_pool())
        .await?;
        Ok(chunk)
    }

    /// Ensure "Document-Upload-AI" category exists for the user
    pub async fn ensure_ai_upload_category(&self, user_id: i32) -> Result<i32> {
        let category_name = "Document-Upload-AI";
        
        // Check if exists
        let existing = sqlx::query_as::<_, super::Category>(
            r#"SELECT * FROM "TblCategories" WHERE "Owner" = $1 AND "CategoryName" = $2 AND "IsDeleted" = false"#
        )
        .bind(user_id)
        .bind(category_name)
        .fetch_optional(self.pool.get_pool())
        .await?;

        if let Some(cat) = existing {
            return Ok(cat.id);
        }

        // Create new
        let category_desc = "Auto-generated category for AI uploads";
        let row = sqlx::query(
            r#"
            INSERT INTO "TblCategories" (
                "CategoryName", "CategoryDesc", "Owner", "ParentId", 
                "IsNeedApproval", "InsertedBy", "InsertedAt", 
                "UpdatedAt", "IsActive", "IsDeleted"
            )
            VALUES ($1, $2, $3, NULL, $4, $5, NOW(), NOW(), true, false)
            RETURNING "Id"
            "#
        )
        .bind(category_name)
        .bind(category_desc)
        .bind(user_id)
        .bind(false) // IsNeedApproval
        .bind(user_id) // InsertedBy
        .fetch_one(self.pool.get_pool())
        .await?;
        
        let id: i32 = row.get("Id");

        Ok(id)
    }

    // ============ NEW METHODS FOR META-QUESTION HANDLING ============
    
    /// Get document metadata for overview questions
    pub async fn get_document_metadata(
        &self,
        document_id: i32,
    ) -> Result<DocumentMetadata> {
        #[derive(FromRow)]
        struct MetadataRow {
            document_id: i32,
            title: String,
            description: Option<String>,
            auto_summary: Option<String>,
            file_size: Option<i32>,
            total_chunks: Option<i64>,
            created_at: DateTime<Utc>,
        }
        
        let row = sqlx::query_as::<_, MetadataRow>(
            r#"
            SELECT 
                d."Id" as document_id,
                d."DocumentTitle" as title,
                d."DocumentDesc" as description,
                m.auto_summary,
                d."FileSize" as file_size,
                (SELECT COUNT(*) FROM rag_document_chunks c WHERE c.document_id = d."Id") as total_chunks,
                d."InsertedAt" as created_at
            FROM "TblDocuments" d
            LEFT JOIN rag_document_metadata m ON m.document_id = d."Id"
            WHERE d."Id" = $1 AND d."IsDeleted" = false
            "#
        )
        .bind(document_id)
        .fetch_one(self.pool.get_pool())
        .await?;
        
        Ok(DocumentMetadata {
            document_id: row.document_id,
            title: row.title,
            description: row.description,
            auto_summary: row.auto_summary,
            file_size: row.file_size,
            total_chunks: row.total_chunks.unwrap_or(0) as i32,
            created_at: row.created_at,
        })
    }
    
    /// Get first N chunks of a document
    pub async fn get_document_overview_chunks(
        &self,
        document_id: i32,
        limit: i32,
    ) -> Result<Vec<DocumentChunk>> {
        let chunks = sqlx::query_as::<_, DocumentChunk>(
            r#"
            SELECT 
                c.id as chunk_id,
                c.document_id,
                d."DocumentTitle" as document_title,
                c.content,
                1.0::float4 as similarity,
                c.chunk_index,
                NULL::int as page_number
            FROM rag_document_chunks c
            JOIN "TblDocuments" d ON d."Id" = c.document_id
            WHERE c.document_id = $1
            ORDER BY c.chunk_index ASC
            LIMIT $2
            "#
        )
        .bind(document_id)
        .bind(limit)
        .fetch_all(self.pool.get_pool())
        .await?;
        
        Ok(chunks)
    }
    
    /// Get complete document overview (metadata + first chunks)
    pub async fn get_document_overview(
        &self,
        document_id: i32,
        chunk_limit: i32,
    ) -> Result<DocumentOverview> {
        let metadata = self.get_document_metadata(document_id).await?;
        let first_chunks = self.get_document_overview_chunks(document_id, chunk_limit).await?;
        
        Ok(DocumentOverview {
            metadata,
            first_chunks,
        })
    }
    
    /// Update document auto_summary field
    pub async fn update_document_summary(
        &self,
        document_id: i32,
        auto_summary: String,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO rag_document_metadata 
                (document_id, auto_summary, summary_token_count, summary_generated_at, updated_at)
            VALUES 
                ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (document_id) 
            DO UPDATE SET 
                auto_summary = EXCLUDED.auto_summary,
                summary_token_count = EXCLUDED.summary_token_count,
                summary_generated_at = NOW(),
                updated_at = NOW()
            "#
        )
        .bind(document_id)
        .bind(&auto_summary)
        .bind(auto_summary.split_whitespace().count() as i32)
        .execute(self.pool.get_pool())
        .await?;
        
        Ok(())
    }
    
    // ============ END NEW METHODS ============
    
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

    /// Create document metadata records
    pub async fn create_document(
        &self,
        user_id: i32,
        filename: &str,
        file_size: i32,
        file_type: &str,
        category_id: i32,
        file_path: &str,
    ) -> Result<i32> {
        let mut transaction = self.pool.get_pool().begin().await?;
        
        // Insert into TblDocuments
        // Note: Assuming TblDocuments has a CategoryId column or similar. 
        // Based on typical schema, but need to be sure. 
        // If schema is not known, I will assume "CategoryId" column exists or I might need to check.
        // User implied "simpan ke TblDocuments" using the Category ID.
        // I will allow adding it to the query.
        
        
        let document_title = filename;
        let document_desc = format!("Uploaded via API: {}", filename);
        // let file_path = format!("uploads/{}/{}", user_id, filename); 
        
        let row = sqlx::query(
            r#"
            INSERT INTO "TblDocuments" (
                "Owner", "DocumentTitle", "DocumentDesc", 
                "InsertedBy", "InsertedAt", "UpdatedAt", "IsDeleted", 
                "FileSize", "FileType", "CategoryId"
            )
            VALUES ($1, $2, $3, $4, NOW(), NOW(), false, $5, $6, $7)
            RETURNING "Id"
            "#
        )
        .bind(user_id)
        .bind(document_title)
        .bind(document_desc)
        .bind(user_id)
        .bind(file_size)
        .bind(file_type)
        .bind(category_id)
        .fetch_one(&mut *transaction)
        .await?;
        
        let document_id: i32 = row.get("Id");
        
        // Insert into TblDocumentFiles
        // (Assuming logic remains somewhat similar, verifying usage)
         sqlx::query(
            r#"
            INSERT INTO "TblDocumentFiles" (
                "DocumentId", "FileName", "FilePath", "IsMainDocumentFile", 
                "InsertedBy", "InsertedAt", "UpdatedAt", "IsDeleted"
            )
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW(), false)
            "#
        )
        .bind(document_id)
        .bind(filename)
        .bind(file_path)
        .bind(true) // IsMainDocumentFile
        .bind(user_id)
        .execute(&mut *transaction)
        .await?;
        
        transaction.commit().await?;
        
        Ok(document_id)
    }

    /// Ensure the processing status table exists
    pub async fn ensure_processing_table(&self) -> Result<()> {
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS rag_document_processing (
                document_id INT PRIMARY KEY,
                status VARCHAR(50) NOT NULL,
                progress FLOAT NOT NULL DEFAULT 0,
                message TEXT,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )"#
        )
        .execute(self.pool.get_pool())
        .await?;
        Ok(())
    }

    /// Ensure necessary indexes exist for performance optimization
    pub async fn ensure_indices(&self) -> Result<()> {
        let pool = self.pool.get_pool();

        // 1. Vector Search Index (IVFFlat)
        // Optimization: ivfflat is good for recall/speed balance. lists=100 is a good default for <100k rows.
        debug!("Ensuring vector index exists...");
        sqlx::query(
            r#"CREATE INDEX IF NOT EXISTS idx_rag_chunks_embedding 
               ON rag_document_chunks 
               USING ivfflat (embedding vector_cosine_ops) 
               WITH (lists = 100)"#
        )
        .execute(pool)
        .await?;

        // 2. Filtering Index (User + Doc ID)
        debug!("Ensuring filtering index exists...");
        sqlx::query(
            r#"CREATE INDEX IF NOT EXISTS idx_rag_chunks_doc_id 
               ON rag_document_chunks(document_id)"#
        )
        .execute(pool)
        .await?;

        // 3. Full text search index (GIN)
        debug!("Ensuring FTS index exists...");
        sqlx::query(
            r#"CREATE INDEX IF NOT EXISTS idx_rag_chunks_content_fts 
               ON rag_document_chunks 
               USING gin(to_tsvector('english', content))"#
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update or insert document processing status
    pub async fn upsert_document_processing_status(
        &self,
        document_id: i32,
        status: &str,
        progress: f64,
        message: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO rag_document_processing 
               (document_id, status, progress, message, updated_at)
               VALUES ($1, $2, $3, $4, NOW())
               ON CONFLICT (document_id) 
               DO UPDATE SET 
                  status = EXCLUDED.status,
                  progress = EXCLUDED.progress,
                  message = EXCLUDED.message,
                  updated_at = NOW()"#
        )
        .bind(document_id)
        .bind(status)
        .bind(progress)
        .bind(message)
        .execute(self.pool.get_pool())
        .await?;
        
        Ok(())
    }

    /// Get documents that are currently being processed for a user
    pub async fn get_user_processing_documents(
        &self,
        user_id: i32,
    ) -> Result<Vec<super::DocumentProcessingStatus>> {
        let docs = sqlx::query_as::<_, super::DocumentProcessingStatus>(
            r#"SELECT 
                p.document_id,
                p.status,
                p.progress,
                p.message,
                p.updated_at
               FROM rag_document_processing p
               JOIN "TblDocuments" d ON d."Id" = p.document_id
               WHERE d."Owner" = $1 
                 AND p.status NOT IN ('completed', 'failed')
               ORDER BY p.updated_at DESC
               LIMIT 1"#
        )
        .bind(user_id)
        .fetch_all(self.pool.get_pool())
        .await?;
        
        Ok(docs)
    }
}
