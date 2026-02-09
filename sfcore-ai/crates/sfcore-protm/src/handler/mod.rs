pub mod architecture_pattern_handler;
pub mod attribute_handler;
pub mod entity_handler;
pub mod entity_relationship_handler;
pub mod file_template_handler;
pub mod flow_step_handler;
pub mod master_persona_handler;
pub mod module_handler;
pub mod pattern_layer_handler;
pub mod persona_handler;
pub mod project_handler;
pub mod prompt_handler;
pub mod stack_library_handler;
pub mod task_dependency_handler;
pub mod task_entity_usage_handler;
pub mod task_file_mapping_handler;
pub mod task_handler;
pub mod tech_stack_handler;
pub mod use_case_handler;
pub mod user_story_handler;

use axum::Router;
use std::sync::Arc;
use crate::AppState;

pub use architecture_pattern_handler as architecture_patterns;
pub use attribute_handler as attributes;
pub use entity_handler as entities;
pub use entity_relationship_handler as entity_relationships;
pub use file_template_handler as file_templates;
pub use flow_step_handler as flow_steps;
pub use master_persona_handler as master_personas;
pub use module_handler as modules;
pub use pattern_layer_handler as pattern_layers;
pub use persona_handler as personas;
pub use project_handler as projects;
pub use prompt_handler as prompts;
pub use stack_library_handler as stack_libraries;
pub use task_dependency_handler as task_dependencies;
pub use task_entity_usage_handler as task_entity_usage;
pub use task_file_mapping_handler as task_file_mappings;
pub use task_handler as tasks;
pub use tech_stack_handler as tech_stacks;
pub use use_case_handler as use_cases;
pub use user_story_handler as user_stories;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/architecture-patterns", architecture_patterns::router())
        .nest("/api/attributes", attributes::router())
        .nest("/api/entities", entities::router())
        .nest("/api/entity-relationships", entity_relationships::router())
        .nest("/api/file-templates", file_templates::router())
        .nest("/api/flow-steps", flow_steps::router())
        .nest("/api/modules", modules::router())
        .nest("/api/pattern-layers", pattern_layers::router())
        .nest("/api/personas", personas::router())
        .nest("/api/projects", projects::router())
        .nest("/api/stack-libraries", stack_libraries::router())
        .nest("/api/task-dependencies", task_dependencies::router())
        .nest("/api/task-entity-usage", task_entity_usage::router())
        .nest("/api/task-file-mappings", task_file_mappings::router())
        .nest("/api/tasks", tasks::router())
        .nest("/api/tech-stacks", tech_stacks::router())
        .nest("/api/use-cases", use_cases::router())
        .nest("/api/user-stories", user_stories::router())
        .nest("/api/master-personas", master_personas::router())
        .nest("/api/prompts", prompts::router())
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive()) 
}
