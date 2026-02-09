use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateProjectDto {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateProjectDto {
    pub name: String,
    pub description: Option<String>,
}
