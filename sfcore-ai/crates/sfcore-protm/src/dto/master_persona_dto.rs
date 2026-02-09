use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateMasterPersonaDto {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct UpdateMasterPersonaDto {
    pub name: String,
    pub description: String,
}
