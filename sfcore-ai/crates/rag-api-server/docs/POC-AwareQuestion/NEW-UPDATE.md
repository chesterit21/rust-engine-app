# 🚀 Integration Guide: Meta-Question Handling (SAFE PRODUCTION VERSION)

## ⚠️ CRITICAL UPDATE: Separate Metadata Table Approach

**Problem Identified:** Cannot ALTER `TblDocuments` while application is running in production.

**Solution:** Use separate `rag_document_metadata` table with LEFT JOIN.

---

## 📁 Updated Files

### Migration (CHANGED - SAFE FOR PRODUCTION)

**OLD (UNSAFE):**

```sql
ALTER TABLE "TblDocuments" ADD COLUMN auto_summary TEXT;  -- ❌ DANGEROUS
```

**NEW (SAFE):**

```sql
CREATE TABLE rag_document_metadata (  -- ✅ SAFE
    document_id INT PRIMARY KEY,
    auto_summary TEXT,
    -- ... other fields
);
```

---

## 🔧 Step-by-Step Integration (UPDATED)

### Step 1: Database Migration (5 min) - SAFE FOR PRODUCTION ✅

```bash
# Create migration file
cat > migrations/$(date +%Y%m%d%H%M%S)_create_metadata_table.sql << 'EOF'
-- Safe migration - creates new table, no ALTER
CREATE TABLE IF NOT EXISTS rag_document_metadata (
    document_id INT PRIMARY KEY,
    auto_summary TEXT,
    summary_generated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    summary_token_count INT,
    topics TEXT[],
    language VARCHAR(10),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT fk_document_metadata_document 
        FOREIGN KEY (document_id) 
        REFERENCES "TblDocuments"("Id") 
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_metadata_document_id 
    ON rag_document_metadata(document_id);

CREATE INDEX IF NOT EXISTS idx_metadata_has_summary 
    ON rag_document_metadata(auto_summary) 
    WHERE auto_summary IS NOT NULL;

COMMENT ON TABLE rag_document_metadata IS 
'AI-generated metadata for documents. Separate table to avoid altering production TblDocuments.';
EOF

# Run migration (SAFE - no downtime)
psql -U postgres -d your_database -f migrations/*_create_metadata_table.sql
```

**Why This Is Safe:**

- ✅ Creates NEW table (no locks on existing tables)
- ✅ Can run while app is live
- ✅ No ALTER TABLE needed
- ✅ Foreign key CASCADE handles cleanup
- ✅ Zero downtime

---

### Step 2-9: Same as Before

(Steps 2-9 remain unchanged from original INTEGRATION_GUIDE.md)

---

### Step 10: Update Repository Methods (CHANGED)

**IMPORTANT:** Repository queries now use **LEFT JOIN** with metadata table.

```rust
// In src/database/repository.rs

/// Get document metadata - UPDATED to use LEFT JOIN
pub async fn get_document_metadata(
    &self,
    document_id: i32,
) -> Result<DocumentMetadata> {
    let row = sqlx::query_as::<_, MetadataRow>(
        r#"
        SELECT 
            d."Id" as document_id,
            d."DocumentTitle" as title,
            d."DocumentDesc" as description,
            m.auto_summary,  -- ← LEFT JOIN with rag_document_metadata
            d."FileSize" as file_size,
            COUNT(c.id) as total_chunks,
            d."InsertedAt" as created_at
        FROM "TblDocuments" d
        LEFT JOIN rag_document_metadata m ON m.document_id = d."Id"
        LEFT JOIN rag_document_chunks c ON c.document_id = d."Id"
        WHERE d."Id" = $1 AND d."IsDeleted" = false
        GROUP BY d."Id", m.auto_summary
        "#
    )
    .bind(document_id)
    .fetch_one(self.pool.get_pool())
    .await?;
    
    Ok(DocumentMetadata {
        document_id: row.document_id,
        title: row.title,
        description: row.description,
        auto_summary: row.auto_summary,  // Will be NULL if no metadata
        // ... rest of fields
    })
}

/// Update summary - UPDATED to UPSERT into metadata table
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
```

---

## 📊 Comparison: OLD vs NEW Approach

| Aspect | OLD (ALTER TABLE) | NEW (Separate Table) |
|--------|-------------------|----------------------|
| **Production Safety** | ❌ Requires downtime | ✅ Zero downtime |
| **Risk** | 🔴 High (schema change) | 🟢 Low (new table) |
| **Rollback** | ❌ Complex | ✅ Easy (DROP TABLE) |
| **Query Performance** | ⚡ Direct column | ⚡ LEFT JOIN (minimal overhead) |
| **Existing Data** | ❌ Requires migration | ✅ Gradual population |
| **Flexibility** | Limited | ✅ Can add fields easily |

---

## 🔄 Data Population Strategy

### For New Documents (Automatic)

```rust
// In document_service.rs process_upload()
// After chunks are saved:
self.generate_document_summary(document_id, &chunks).await?;
// ↓ Saves to rag_document_metadata table
```

### For Existing Documents (Optional Background Job)

**Option 1: Lazy Loading**

- Summary generated on first "overview" query
- Cached in metadata table for future queries

**Option 2: Batch Processing**

```rust
// Background job script
async fn regenerate_all_summaries() {
    let documents = repository.get_documents_without_metadata().await?;
    
    for doc in documents {
        let chunks = repository.get_document_overview_chunks(doc.id, 10).await?;
        let summary = llm_service.generate_summary(&chunks).await?;
        repository.update_document_summary(doc.id, summary).await?;
    }
}
```

---

## ✅ Testing Checklist (UPDATED)

### Test 1: Fresh Document Upload

```
User uploads new document
Expected:
✅ Document processed
✅ Summary generated automatically
✅ Row inserted into rag_document_metadata
✅ Meta-question returns summary
```

### Test 2: Existing Document (No Metadata Yet)

```
User asks: "ini dokumen tentang apa?" on old document
Expected:
✅ Query returns auto_summary = NULL
✅ System uses first chunks instead
✅ Still provides good overview
```

### Test 3: Migration Verification

```sql
-- Verify table exists
SELECT COUNT(*) FROM rag_document_metadata;

-- Verify foreign key
SELECT 
    d."Id", 
    d."DocumentTitle", 
    m.auto_summary 
FROM "TblDocuments" d
LEFT JOIN rag_document_metadata m ON m.document_id = d."Id"
LIMIT 10;
```

---

## 🐛 Common Issues (UPDATED)

### Issue 1: "table rag_document_metadata does not exist"

**Solution:**

```bash
psql -d your_db -f create_metadata_table_migration.sql
```

### Issue 2: Foreign key violation

**Cause:** Trying to insert metadata for non-existent document_id
**Solution:** Ensure document exists in TblDocuments first

### Issue 3: NULL auto_summary for old documents

**This is EXPECTED behavior**

- Old documents won't have metadata yet
- System will gracefully handle NULL and use first chunks
- Optionally run batch job to populate

---

## 🔄 Rollback Plan (UPDATED - EASIER)

If issues occur:

### 1. Stop Using Metadata Table

```rust
// In repository.rs, comment out LEFT JOIN:
// LEFT JOIN rag_document_metadata m ON m.document_id = d."Id"
// Just return NULL for auto_summary
```

### 2. Drop Table (if needed)

```sql
DROP TABLE IF EXISTS rag_document_metadata CASCADE;
```

**No risk to main TblDocuments table!** ✅

---

## 🎯 Key Benefits of Separate Table

1. ✅ **Zero Production Risk** - No ALTER on active tables
2. ✅ **Easy Rollback** - Just DROP new table
3. ✅ **Gradual Migration** - Populate over time
4. ✅ **Extensible** - Easy to add new metadata fields
5. ✅ **Clean Separation** - RAG data separate from core business data

---

## 📝 Summary

**Old Approach:**

```
ALTER TABLE "TblDocuments" ADD COLUMN auto_summary TEXT;  -- ❌ Risky
```

**New Approach:**

```
CREATE TABLE rag_document_metadata (...);  -- ✅ Safe
SELECT ... FROM TblDocuments d 
LEFT JOIN rag_document_metadata m ON ...;  -- ✅ Works perfectly
```

**Migration Impact:**

- ✅ Zero downtime
- ✅ No data loss risk
- ✅ Easy rollback
- ✅ Same end-user experience

**Ready to integrate?** Start with **Step 1: Migration**! 🚀
