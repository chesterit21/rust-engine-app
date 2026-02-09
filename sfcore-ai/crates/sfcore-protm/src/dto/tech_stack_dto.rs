use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct CreateTechStackDto {
    pub name: String,
    #[serde(rename = "type")]
    pub stack_type: String,
    pub language: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UpdateTechStackDto {
    pub name: String,
    #[serde(rename = "type")]
    pub stack_type: String,
    pub language: String,
    pub description: Option<String>,
}
