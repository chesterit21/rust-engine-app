use serde::Deserialize;

#[derive(Deserialize)]
pub struct GeneratePromptRequest {
    pub stack_id: String,
    pub version: String,
}
