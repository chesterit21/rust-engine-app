use crate::models::chat::*;
use crate::security::DocumentAuthorization;
use crate::services::{DocumentService, RagService};
use crate::utils::error::ApiError;
use axum::{
    extract::Extension,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

pub async fn chat_stream_handler(
    Extension(rag_service): Extension<Arc<RagService>>,
    Extension(document_service): Extension<Arc<DocumentService>>,
    Extension(doc_auth): Extension<Arc<DocumentAuthorization>>,
    Json(request): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    
    let start_time = Instant::now();
    
    // Parse user_id
    let user_id: i32 = request.user_id.parse()
        .map_err(|_| ApiError::BadRequest("Invalid user_id format".to_string()))?;
    
    let session_id = request.session_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    info!(
        "Chat request: user={}, session={}, message_len={}, has_upload={}, has_selected={}",
        user_id,
        session_id,
        request.message.len(),
        request.document_upload.is_some(),
        request.document_selected.is_some()
    );
    
    // Clone untuk move into async stream
    let session_id_clone = session_id.clone();
    let message = request.message.clone();
    let document_upload = request.document_upload.clone();
    let document_selected = request.document_selected.clone();
    
    // Create SSE stream
    let stream = async_stream::stream! {
        // ===== EVENT 1: Session Info =====
        yield Ok(create_sse_event("session", &SessionInfo {
            session_id: session_id_clone.clone(),
            user_id: user_id.to_string(),
            timestamp: chrono::Utc::now(),
        }));
        
        let mut uploaded_doc_ids: Vec<i32> = Vec::new();
        
        // ===== STEP 1: Handle Document Uploads (if any) =====
        if let Some(uploads) = document_upload {
            info!("Processing {} uploaded files", uploads.len());
            
            yield Ok(create_sse_event("status", &StatusInfo {
                stage: "uploading".to_string(),
                message: format!("Processing {} file(s)...", uploads.len()),
                progress: 10,
            }));
            
            let mut uploaded_results = Vec::new();
            
            for (idx, upload) in uploads.iter().enumerate() {
                debug!("Processing file {}/{}: {}", idx + 1, uploads.len(), upload.file_name);
                
                // Decode base64
                let file_data = match base64::decode(&upload.file_base64) {
                    Ok(data) => data,
                    Err(e) => {
                        warn!("Failed to decode base64 for {}: {}", upload.file_name, e);
                        uploaded_results.push(UploadedDocInfo {
                            document_id: 0,
                            file_name: upload.file_name.clone(),
                            status: "failed".to_string(),
                            chunks_created: 0,
                            error_message: Some(format!("Invalid base64: {}", e)),
                        });
                        continue;
                    }
                };
                
                yield Ok(create_sse_event("status", &StatusInfo {
                    stage: "parsing".to_string(),
                    message: format!("Parsing {}...", upload.file_name),
                    progress: 20 + ((idx as u8 * 20) / uploads.len() as u8),
                }));
                
                // Process document
                match document_service
                    .process_upload(user_id, upload.file_name.clone(), file_data)
                    .await
                {
                    Ok((document_id, chunks_count)) => {
                        info!("Document {} processed: id={}, chunks={}", 
                            upload.file_name, document_id, chunks_count);
                        
                        uploaded_doc_ids.push(document_id);
                        
                        uploaded_results.push(UploadedDocInfo {
                            document_id,
                            file_name: upload.file_name.clone(),
                            status: "success".to_string(),
                            chunks_created: chunks_count,
                            error_message: None,
                        });
                    }
                    Err(e) => {
                        warn!("Failed to process {}: {}", upload.file_name, e);
                        
                        uploaded_results.push(UploadedDocInfo {
                            document_id: 0,
                            file_name: upload.file_name.clone(),
                            status: "failed".to_string(),
                            chunks_created: 0,
                            error_message: Some(e.to_string()),
                        });
                    }
                }
            }
            
            // Send upload results
            yield Ok(create_sse_event("documents_uploaded", &uploaded_results));
        }
        
        // ===== STEP 2: Determine Document Scope =====
        let target_document_ids: Option<Vec<i32>> = if !uploaded_doc_ids.is_empty() {
            // Use uploaded documents
            Some(uploaded_doc_ids)
        } else if let Some(selected) = document_selected {
            // Use selected documents
            let parsed: Vec<i32> = selected
                .iter()
                .filter_map(|id| id.parse().ok())
                .collect();
            
            if parsed.is_empty() {
                yield Ok(create_sse_event("error", &ErrorInfo {
                    code: "INVALID_DOCUMENT_IDS".to_string(),
                    message: "Invalid document IDs provided".to_string(),
                }));
                return;
            }
            
            // Verify access to all selected documents
            for doc_id in &parsed {
                match doc_auth.check_access(user_id, *doc_id).await {
                    Ok(true) => {},
                    Ok(false) => {
                        yield Ok(create_sse_event("error", &ErrorInfo {
                            code: "DOCUMENT_ACCESS_DENIED".to_string(),
                            message: format!("Access denied to document ID: {}", doc_id),
                        }));
                        return;
                    }
                    Err(e) => {
                        yield Ok(create_sse_event("error", &ErrorInfo {
                            code: "DATABASE_ERROR".to_string(),
                            message: format!("Failed to check access: {}", e),
                        }));
                        return;
                    }
                }
            }
            
            Some(parsed)
        } else {
            // General chat - use all user's documents
            None
        };
        
        debug!("Document scope: {:?}", target_document_ids);
        
        // ===== STEP 3: Retrieve Relevant Context =====
        yield Ok(create_sse_event("status", &StatusInfo {
            stage: "retrieving".to_string(),
            message: "Searching relevant documents...".to_string(),
            progress: 50,
        }));
        
        let document_id_for_search = target_document_ids.as_ref().and_then(|ids| ids.first().copied());
        
        let chunks = match rag_service
            .retrieve(user_id, &message, document_id_for_search)
            .await
        {
            Ok(chunks) => chunks,
            Err(e) => {
                yield Ok(create_sse_event("error", &ErrorInfo {
                    code: "RETRIEVAL_ERROR".to_string(),
                    message: format!("Failed to retrieve context: {}", e),
                }));
                return;
            }
        };
        
        if chunks.is_empty() {
            yield Ok(create_sse_event("error", &ErrorInfo {
                code: "NO_RELEVANT_CONTEXT".to_string(),
                message: "Tidak ditemukan informasi yang relevan dalam dokumen Anda.".to_string(),
            }));
            return;
        }
        
        info!("Retrieved {} relevant chunks", chunks.len());
        
        // ===== EVENT 4: Send Sources =====
        let sources: Vec<SourceInfo> = chunks
            .iter()
            .map(|chunk| SourceInfo {
                document_id: chunk.document_id,
                document_name: chunk.document_title.clone(),
                chunk_id: chunk.chunk_id,
                similarity: chunk.similarity,
                page_number: chunk.page_number,
                preview: chunk.content.chars().take(150).collect::<String>(),
                download_url: format!("/api/documents/{}/download", chunk.document_id),
                view_url: format!("/api/documents/{}/view?page={}", 
                    chunk.document_id,
                    chunk.page_number.unwrap_or(1)
                ),
            })
            .collect();
        
        yield Ok(create_sse_event("sources", &sources));
        
        // ===== STEP 4: Generate AI Response =====
        yield Ok(create_sse_event("status", &StatusInfo {
            stage: "generating".to_string(),
            message: "Generating response...".to_string(),
            progress: 70,
        }));
        
        let context = rag_service.build_context(chunks);
        let llm_messages = rag_service.build_prompt(&message, &context);
        
        let mut llm_stream = match rag_service.llm_service.chat_stream(llm_messages).await {
            Ok(stream) => stream,
            Err(e) => {
                yield Ok(create_sse_event("error", &ErrorInfo {
                    code: "LLM_ERROR".to_string(),
                    message: format!("Failed to generate response: {}", e),
                }));
                return;
            }
        };
        
        // ===== EVENT 5: Stream AI Response =====
        use futures::StreamExt;
        
        while let Some(result) = llm_stream.next().await {
            match result {
                Ok(content) => {
                    if !content.is_empty() {
                        yield Ok(create_sse_event("message", &MessageChunk {
                            delta: content,
                        }));
                    }
                }
                Err(e) => {
                    yield Ok(create_sse_event("error", &ErrorInfo {
                        code: "LLM_STREAM_ERROR".to_string(),
                        message: format!("Streaming error: {}", e),
                    }));
                    break;
                }
            }
        }
        
        // ===== EVENT 6: Completion =====
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        yield Ok(create_sse_event("done", &CompletionInfo {
            session_id: session_id_clone,
            message_id: uuid::Uuid::new_v4().to_string(),
            sources_count: sources.len(),
            processing_time_ms: processing_time,
        }));
        
        info!("Chat completed in {}ms", processing_time);
    };
    
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

// Helper: Create SSE event
fn create_sse_event<T: serde::Serialize>(event_type: &str, data: &T) -> Event {
    Event::default()
        .event(event_type)
        .data(serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string()))
}
