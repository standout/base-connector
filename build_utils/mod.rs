pub mod action_router;
pub mod error;
pub mod mod_generator;
pub mod schema_collector;
pub mod schema_embedder;
pub mod trigger_router;

pub use action_router::generate_action_routing;
pub use error::BuildError;
pub use mod_generator::{generate_actions_mod_rs, generate_triggers_mod_rs};
pub use schema_collector::collect_schema_files;
pub use schema_embedder::generate_embedded_code;
pub use trigger_router::generate_trigger_routing;
