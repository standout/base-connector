//! Harmless internal candidate: one bounded deterministic action, no diagnostic code.
wit_bindgen::generate!({ path: "wit/standout-app.wit", world: "bridge" });

use serde::Deserialize;
use serde_json::{Value, json};
use standout::app::types::{
    ActionContext, ActionResponse, AppError, ErrorCode, TriggerContext, TriggerResponse,
};

struct Candidate;
export!(Candidate);

const ACTIONS: [&str; 1] = ["echo_contract"];

fn error(message: &str) -> AppError {
    AppError {
        code: ErrorCode::Misconfigured,
        message: message.into(),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    reference: String,
    values: Vec<Value>,
}

fn echo(input: &str) -> Result<Value, AppError> {
    let parsed: Input = serde_json::from_str(input).map_err(|_| error("Invalid fixture input"))?;
    let valid = !parsed.reference.is_empty()
        && parsed.reference.chars().count() <= 80
        && parsed.values.len() <= 8
        && parsed.values.iter().all(|value| {
            value.is_null()
                || value.is_boolean()
                || value
                    .as_f64()
                    .is_some_and(|n| n.fract() == 0.0 && n.abs() <= 9007199254740991.0)
        });
    if !valid {
        return Err(error("Invalid fixture input"));
    }
    let environment_count = 0;
    Ok(
        json!({"reference":parsed.reference,"values":parsed.values,"environment_count":environment_count}),
    )
}

impl exports::standout::app::actions::Guest for Candidate {
    fn action_ids() -> Result<Vec<String>, AppError> {
        Ok(ACTIONS.iter().map(|id| (*id).to_string()).collect())
    }
    fn input_schema(context: ActionContext) -> Result<String, AppError> {
        if !ACTIONS.contains(&context.action_id.as_str()) {
            return Err(error("Unknown fixture action"));
        }
        Ok(include_str!("../schemas/input.json").into())
    }
    fn output_schema(context: ActionContext) -> Result<String, AppError> {
        if !ACTIONS.contains(&context.action_id.as_str()) {
            return Err(error("Unknown fixture action"));
        }
        Ok(include_str!("../schemas/output.json").into())
    }
    fn execute(context: ActionContext) -> Result<ActionResponse, AppError> {
        if context.connection.serialized_data != "{}" {
            return Err(error("No connection data permitted"));
        }
        let output = match context.action_id.as_str() {
            "echo_contract" => echo(&context.serialized_input)?,
            _ => return Err(error("Unknown fixture action")),
        };
        Ok(ActionResponse {
            serialized_output: output.to_string(),
        })
    }
}

impl exports::standout::app::triggers::Guest for Candidate {
    fn trigger_ids() -> Result<Vec<String>, AppError> {
        Ok(vec![])
    }
    fn input_schema(_: TriggerContext) -> Result<String, AppError> {
        Err(error("No triggers"))
    }
    fn output_schema(_: TriggerContext) -> Result<String, AppError> {
        Err(error("No triggers"))
    }
    fn fetch_events(_: TriggerContext) -> Result<TriggerResponse, AppError> {
        Err(error("No triggers"))
    }
}

impl exports::standout::app::connections::Guest for Candidate {
    fn connection_config() -> String {
        r#"{"strategies":[],"default_strategy":"","runtime":{}}"#.into()
    }
}
