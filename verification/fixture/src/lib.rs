//! Internal verification fixture, never a customer connector or release candidate.
wit_bindgen::generate!({ path: "wit/standout-app.wit", world: "bridge" });

use serde::Deserialize;
use serde_json::{Value, json};
use standout::app::types::{
    ActionContext, ActionResponse, AppError, ErrorCode, TriggerContext, TriggerResponse,
};

struct Fixture;
export!(Fixture);

const ACTIONS: [&str; 4] = [
    "echo_contract",
    "network_probe",
    "cpu_probe",
    "memory_probe",
];
const EMPTY_SCHEMA: &str = r#"{"type":"object","additionalProperties":false}"#;

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
    let environment_count = standout::app::environment::env_vars().len();
    if environment_count != 0 {
        return Err(error("Fixture requires empty environment"));
    }
    Ok(
        json!({"reference":parsed.reference,"values":parsed.values,"environment_count":environment_count}),
    )
}

fn network_probe() -> Result<Value, AppError> {
    // RFC 5737 documentation address. This can run only in the verifier's
    // network-none container, never on the host or through a platform app.
    let result = standout::app::http::RequestBuilder::new()
        .method(standout::app::http::Method::Get)
        .url("http://192.0.2.1:80/denied")
        .send();
    match result {
        Ok(_) => Err(error("Network unexpectedly available")),
        Err(_) => Ok(json!({"network_denied":true})),
    }
}

impl exports::standout::app::actions::Guest for Fixture {
    fn action_ids() -> Result<Vec<String>, AppError> {
        Ok(ACTIONS.iter().map(|id| (*id).to_string()).collect())
    }
    fn input_schema(context: ActionContext) -> Result<String, AppError> {
        if !ACTIONS.contains(&context.action_id.as_str()) {
            return Err(error("Unknown fixture action"));
        }
        Ok(if context.action_id == "echo_contract" {
            include_str!("../schemas/input.json")
        } else {
            EMPTY_SCHEMA
        }
        .into())
    }
    fn output_schema(context: ActionContext) -> Result<String, AppError> {
        match context.action_id.as_str() {
            "echo_contract" => Ok(include_str!("../schemas/output.json").into()),
            "network_probe" => Ok(r#"{"type":"object","required":["network_denied"],"additionalProperties":false,"properties":{"network_denied":{"const":true}}}"#.into()),
            "cpu_probe" | "memory_probe" => Ok(EMPTY_SCHEMA.into()),
            _ => Err(error("Unknown fixture action")),
        }
    }
    fn execute(context: ActionContext) -> Result<ActionResponse, AppError> {
        if context.connection.serialized_data != "{}" {
            return Err(error("No connection data permitted"));
        }
        let output = match context.action_id.as_str() {
            "echo_contract" => echo(&context.serialized_input)?,
            "network_probe" => network_probe()?,
            "cpu_probe" => loop {
                std::hint::black_box(42u64.wrapping_mul(42));
            },
            "memory_probe" => {
                // Touch successive chunks so the cgroup's RSS limit is exercised.
                let mut chunks = Vec::new();
                loop {
                    chunks.push(std::hint::black_box(vec![7u8; 16 * 1024 * 1024]));
                }
            }
            _ => return Err(error("Unknown fixture action")),
        };
        Ok(ActionResponse {
            serialized_output: output.to_string(),
        })
    }
}

impl exports::standout::app::triggers::Guest for Fixture {
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

impl exports::standout::app::connections::Guest for Fixture {
    fn connection_config() -> String {
        r#"{"strategies":[],"default_strategy":"","runtime":{}}"#.into()
    }
}
