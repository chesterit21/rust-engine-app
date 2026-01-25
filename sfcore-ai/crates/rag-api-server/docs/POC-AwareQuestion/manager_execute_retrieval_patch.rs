// This is a PATCH for manager.rs execute_retrieval_decision method
// Add this import at the top of manager.rs:
// use crate::database::models::{DocumentMetadata, DocumentOverview};

async fn execute_retrieval_decision(
    &self,
    state: &mut ConversationState,
    decision: &RetrievalDecision,
    current_message: &str,
    document_id: Option<i64>,
    current_embedding: &[f32],
) -> Result<String> {
    match decision {
        RetrievalDecision::Skip { reason } => {
            debug!("Skipping retrieval: {:?}", reason);
            state.metadata.retrieval_skipped_count += 1;
            Ok(state.system_context.clone())
        }
        RetrievalDecision::Retrieve { reason, context_aware } => {
            match reason {
                // ============ NEW: Handle DocumentMetadataQuery ============
                RetrieveReason::DocumentMetadataQuery => {
                    info!("Processing document metadata query (overview/summary question)");
                    
                    if let Some(doc_id) = document_id {
                        // Log metadata retrieval start
                        self.logger.log(
                            ActivityLog::builder(state.session_id, state.user_id, ActivityType::ProcessingStage)
                                .message("FETCH DOCUMENT METADATA")
                                .build()
                        );
                        
                        // Get document overview (metadata + first 5 chunks)
                        let overview = self.retrieval_provider
                            .get_document_overview(doc_id as i32, 5)
                            .await
                            .context("Failed to fetch document overview")?;
                        
                        // Build context from metadata + first chunks
                        let context_text = self.build_metadata_context(&overview);
                        
                        // Build system context (no need for LLM summarization here)
                        let system_context = self.context_builder.build_system_context(
                            &context_text,
                            Some(&format!("Document: {}", overview.metadata.title)),
                        );
                        
                        state.system_context = system_context.clone();
                        state.last_retrieval_summary = context_text;
                        state.document_id = document_id;
                        state.metadata.total_retrievals += 1;
                        
                        // Log completion
                        self.logger.log(
                            ActivityLog::builder(state.session_id, state.user_id, ActivityType::ProcessingStage)
                                .message("METADATA FETCH COMPLETED")
                                .build()
                        );
                        
                        return Ok(system_context);
                    } else {
                        // No document_id provided
                        let error_msg = "Untuk menjawab pertanyaan tentang dokumen, \
                                        silakan upload atau pilih dokumen terlebih dahulu.";
                        
                        state.system_context = error_msg.to_string();
                        return Ok(error_msg.to_string());
                    }
                }
                // ============ END NEW ============
                
                // Existing cases
                _ => {
                    info!("Performing retrieval: {:?}", reason);
                    state.metadata.total_retrievals += 1;

                    let query_embedding = if *context_aware {
                        let context_text = self.context_builder
                            .prepare_context_aware_text(current_message, &state.messages);
                        
                        let config = self.context_builder.weighted_config();
                        self.embedding_provider
                            .embed_weighted(
                                current_message,
                                &context_text,
                                config.current_weight,
                                config.history_weight,
                            )
                            .await?
                    } else {
                        current_embedding.to_vec()
                    };

                    // Catch retrieval errors
                    let chunks = match self.retrieval_provider
                        .search(state.user_id, &query_embedding, document_id)
                        .await {
                            Ok(res) => res,
                            Err(e) => {
                                self.logger.log(
                                    ActivityLog::builder(state.session_id, state.user_id, ActivityType::RetrievalError)
                                        .status(ActivityStatus::Error)
                                        .error(e.to_string(), "RetrievalProviderError")
                                        .build()
                                );
                                error!("Retrieval provider failed: {:?}", e);
                                return Err(e).context("Retrieval failed");
                            }
                        };

                    let summary = self.llm_provider
                        .summarize_chunks(&chunks, current_message)
                        .await
                        .context("Failed to summarize chunks")?;

                    let system_context = self.context_builder.build_system_context(
                        &summary,
                        document_id.map(|id| format!("Document ID: {}", id)).as_deref(),
                    );
                    
                    // Log Prompt Generation
                    self.logger.log(
                        ActivityLog::builder(state.session_id, state.user_id, ActivityType::ProcessingStage)
                            .message("GENERATE MESSAGE PROMPT + CHUNK")
                            .build()
                    );

                    state.system_context = system_context.clone();
                    state.last_retrieval_summary = summary;
                    state.document_id = document_id;

                    Ok(system_context)
                }
            }
        }
    }
}

// ============ NEW HELPER METHOD ============
// Add this helper method to ConversationManager impl block

impl ConversationManager {
    // ... existing methods ...
    
    /// Build context from document metadata and first chunks
    /// Used for meta-questions like "what is this document about?"
    fn build_metadata_context(&self, overview: &DocumentOverview) -> String {
        let metadata = &overview.metadata;
        
        let mut context = String::new();
        
        // Add document metadata
        context.push_str(&format!("=== INFORMASI DOKUMEN ===\n"));
        context.push_str(&format!("Judul: {}\n", metadata.title));
        
        if let Some(desc) = &metadata.description {
            context.push_str(&format!("Deskripsi: {}\n", desc));
        }
        
        if let Some(summary) = &metadata.auto_summary {
            context.push_str(&format!("\nRingkasan:\n{}\n", summary));
        }
        
        context.push_str(&format!("\nTotal bagian: {}\n", metadata.total_chunks));
        
        if let Some(size) = metadata.file_size {
            let size_kb = size as f64 / 1024.0;
            context.push_str(&format!("Ukuran file: {:.1} KB\n", size_kb));
        }
        
        // Add first chunks as preview
        if !overview.first_chunks.is_empty() {
            context.push_str("\n=== CUPLIKAN AWAL DOKUMEN ===\n\n");
            
            for (i, chunk) in overview.first_chunks.iter().enumerate() {
                // Limit preview to first 300 characters per chunk
                let preview = chunk.content
                    .chars()
                    .take(300)
                    .collect::<String>();
                
                let ellipsis = if chunk.content.len() > 300 { "..." } else { "" };
                
                context.push_str(&format!("[Bagian {}]\n{}{}\n\n", i + 1, preview, ellipsis));
            }
        }
        
        context
    }
}
