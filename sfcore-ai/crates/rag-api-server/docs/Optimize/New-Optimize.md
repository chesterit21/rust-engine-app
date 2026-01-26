## Yang sudah OK

- `chat_stream_handler` sudah nge-map `ChatStreamChunk::{Stage,Message,Done}` jadi SSE event `stage/message/done` dengan payload JSON yang rapi.
- `LlmProvider` sekarang ada `generate_with(max_tokens, temperature)` dan `LlmService` implement `generate_chat_with()` untuk planner call yang lebih murah & stabil.
- Planner call udah dipasang sebelum embedding, dan embedding di-skip saat `intent == metadata`.

## Masalah compile yang kemungkinan besar muncul

1) `ContextBuilder::decide_retrieval()` di code lama signature-nya masih pakai `current_document_id: Option<i64>`, tapi di manager kamu sekarang dipanggil dengan `effective_doc_ids: Option<Vec<i64>>`.
2) `ConversationState` di code lama punya field `document_id: Option<i64>`, tapi manager kamu sekarang pakai `state.document_ids` dan `ConversationState::new(..., document_ids)` (berarti `types.rs` juga harus sudah kamu ubah ke `document_ids`).
3) `handlers/chat.rs` sekarang expect `ChatRequest` punya `document_ids`, jadi `models/chat.rs` harus sudah kamu tambah field itu dan pastikan client payload-nya juga sesuai.

## Patch minimal biar “planner simple” stabil

Di bawah ini patch yang aku saranin supaya konsep “metadata vs vector vs clarify” jalan mulus tanpa nambah kompleksitas.

### A) Update `ConversationState` jadi `document_ids`

Di `src/services/conversation/types.rs`, ganti `document_id: Option<i64>` jadi `document_ids: Option<Vec<i64>>`, dan update `new()` serta perbandingan dokumen. (Manager kamu sudah assume ini.)

Konsep minimal:

- `document_ids: Option<Vec<i64>>` disimpan apa adanya.
- Untuk “document scope changed?”, bandingin normalized vector (sort + dedup) biar deterministik.

### B) Update `ContextBuilder::decide_retrieval()` signature

Di `context_builder.rs`, ubah signature supaya menerima `current_document_ids: Option<Vec<i64>>` dan di `ConversationState` juga simpan `document_ids`.

Logic minimal (tanpa rewrite query):

- Kalau session baru → Retrieve.
- Kalau `state.document_ids != current_document_ids` (setelah normalize) → Retrieve (context_aware=true).
- Kalau embedding similarity tinggi → Skip.

Ini penting supaya call di manager kamu compile dan decision-nya konsisten.

### C) Perbaiki update doc scope ke cache

Di `get_or_create_session()`, kamu update `state.document_ids` kalau berubah, tapi state itu cuma dikembalikan—tidak disimpan balik ke cache, jadi perubahan scope bisa “hilang” di request berikutnya.
Fix minimal: setelah mutate `state`, panggil `self.cache.set(session_id, state.clone())` sebelum `return Ok(state)`.

## Cleanup kecil (recommended)

- Di `handlers/chat.rs` sekarang ada import yang kemungkinan unused (`debug`, `SessionEvent`, `SystemEvent`) dan bisa bikin warning/CI fail jika kamu pakai `#![deny(warnings)]`.
Update yang kamu kirim sudah **konsisten** untuk “multi-doc + planner intent simple + SSE stage/message/done”.
`ChatRequest` sekarang sudah support `document_ids` (backward compatible), dan `ConversationState` sudah pindah dari `document_id` ke `document_ids`.

## 1) Validasi perubahan kamu

- `ChatRequest` punya `document_id` + `document_ids` dan keduanya `#[serde(default)]`, jadi request lama masih valid.
- `ConversationState` sudah menyimpan `document_ids: Option<Vec<i64>>` dan `RetrievalReason` sudah diganti jadi `DocumentContextChanged` (bukan `DocumentIdChanged`).
- `ContextBuilder::decide_retrieval()` sudah menerima `current_document_ids: Option<Vec<i64>>` dan kalau beda dengan `state.document_ids`, dia return `Retrieve { reason: DocumentContextChanged }`.
- `handlers/chat.rs` sudah mem-forward event SSE `stage/message/done` sesuai payload JSON yang kamu set.

## 2) Fix penting: normalisasi `document_ids`

Saat ini perbandingan di `ContextBuilder` pakai `state.document_ids != current_document_ids` (vector equality), jadi kalau user pilih dokumen yang sama tapi urutan beda, sistem bakal mengira “context changed” dan retrieval akan kepanggil terus.

Patch paling aman: **sort + dedup** `document_ids` sekali, sebelum disimpan ke state dan sebelum dipakai untuk `decide_retrieval()`.

```rust
fn normalize_doc_ids(mut ids: Vec<i64>) -> Vec<i64> {
    ids.sort_unstable();
    ids.dedup();
    ids
}

// di handle_message, setelah merge legacy document_id + document_ids
let mut final_doc_ids = document_ids.unwrap_or_default();
if let Some(id) = document_id {
    if !final_doc_ids.contains(&id) {
        final_doc_ids.push(id);
    }
}

let effective_doc_ids = if final_doc_ids.is_empty() {
    None
} else {
    Some(normalize_doc_ids(final_doc_ids))
};
```

Dengan ini, `ContextBuilder::decide_retrieval()` jadi deterministik dan tidak “false positive context changed”.

## 3) Fix penting: persist update doc-scope ke cache

Di `get_or_create_session()`, kamu memang meng-update `state.document_ids` kalau berubah (good), tapi pastikan state hasil update itu **di-set lagi** ke cache, supaya request berikutnya tidak balik ke doc scope lama dan memicu `DocumentContextChanged` terus-menerus.

Patch minimal (konsep):

```rust
if let Some(mut state) = self.cache.get(session_id) {
    if state.document_ids != document_ids {
        state.document_ids = document_ids.clone();
        self.cache.set(session_id, state.clone()); // penting
    }
    return Ok(state);
}
```

## 4) (Optional tapi recommended) Harden planner JSON parsing

Planner kamu sekarang `serde_json::from_str::<PlannerOut>(&planner_raw)`; di real world LLM kadang ngasih prefix/suffix (newline, teks tambahan).  
Saran production-grade: extract substring JSON `{...}` pertama, baru parse; kalau gagal fallback ke `"vector"` (yang kamu sudah lakukan).
