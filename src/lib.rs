#![recursion_limit = "512"]

mod actions;
mod client;
mod connections;
mod triggers;

// Include the dynamically generated action routing
include!("schemas/generated/action_routing.rs");

// Include the dynamically generated trigger routing
include!("schemas/generated/trigger_routing.rs");

// Include the embedded schemas
include!("schemas/generated/embedded_schemas.rs");

use wit_bindgen::generate;

generate!({
    path: "wit/standout-app.wit",
    world: "bridge",
});

use crate::exports::standout::app::actions::{AppError as ActionsAppError, Guest as ActionsGuest};
use crate::exports::standout::app::connections::Guest as ConnectionsGuest;
use crate::exports::standout::app::triggers::{
    AppError as TriggersAppError, Guest as TriggersGuest,
};
use crate::standout::app::types::{
    ActionContext, ActionResponse, ActionSchemaContext, SchemaResponse, TriggerContext,
    TriggerResponse, TriggerSchemaContext,
};

struct App;

impl TriggersGuest for App {
    fn trigger_ids() -> Result<Vec<String>, TriggersAppError> {
        // Get trigger IDs from the dynamically generated trigger routing
        Ok(get_available_triggers())
    }

    fn input_schema(context: TriggerSchemaContext) -> Result<SchemaResponse, TriggersAppError> {
        execute_trigger_input_schema_dynamically(&context.trigger_id, &context)
    }

    fn output_schema(context: TriggerSchemaContext) -> Result<SchemaResponse, TriggersAppError> {
        execute_trigger_output_schema_dynamically(&context.trigger_id, &context)
    }

    fn fetch_events(context: TriggerContext) -> Result<TriggerResponse, TriggersAppError> {
        // Execute the appropriate trigger dynamically
        let trigger_id = context.trigger_id.clone();
        execute_trigger_fetch_events_dynamically(&trigger_id, context).map_err(|e| {
            TriggersAppError {
                code: e.code,
                message: e.message,
            }
        })
    }
}

impl ActionsGuest for App {
    fn action_ids() -> Result<Vec<String>, ActionsAppError> {
        // Get action IDs from the dynamically generated action routing
        Ok(get_available_actions())
    }

    fn input_schema(context: ActionSchemaContext) -> Result<SchemaResponse, ActionsAppError> {
        execute_action_input_schema_dynamically(&context.action_id, &context)
    }

    fn output_schema(context: ActionSchemaContext) -> Result<SchemaResponse, ActionsAppError> {
        execute_action_output_schema_dynamically(&context.action_id, &context)
    }

    fn execute(context: ActionContext) -> Result<ActionResponse, ActionsAppError> {
        // Execute the appropriate action dynamically
        let action_id = context.action_id.clone();
        let result =
            execute_action_dynamically(&action_id, context).map_err(|e| ActionsAppError {
                code: e.code,
                message: e.message,
            })?;

        // Serialize the response
        let serialized_output = serde_json::to_string(&result).map_err(|_e| ActionsAppError {
            code: crate::standout::app::types::ErrorCode::Other,
            message: "Failed to serialize response".to_string(),
        })?;

        Ok(ActionResponse { serialized_output })
    }
}

impl ConnectionsGuest for App {
    fn connection_config() -> String {
        connections::connection_config()
    }
}

export!(App);
