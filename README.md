# Base Connector Template

A Rust-based WebAssembly connector template for a Standout connector. This connector works with the Standout platform via Standout App Bridge and can automatically generate boilerplate code for actions and triggers based on OpenAPI specs.

## Overview

This connector uses the **WebAssembly Interface Type (WIT)** specification to define the interface (App Bridge) between the connector and the Standout platform. The `wit/standout-app.wit` file defines:

- **Actions interface**: Methods for executing actions (`execute`, `input_schema`, `output_schema`, `action_ids`)
- **Triggers interface**: Methods for fetching trigger events (`fetch_events`, `input_schema`, `output_schema`, `trigger_ids`)
- **Types**: Shared data structures like `ActionContext`, `TriggerContext`, `Connection`, etc.

The WIT file is used to generate Rust bindings that allow the connector to communicate with the Standout App Bridge runtime. This file should not be modified.

## Quick Start

Build actions and triggers (see below). Change the name of the connector from `base-connector`
to the appropriate name in `Cargo.toml`, for example to `github-connector`.

```bash
# Build the WebAssembly component
cargo build --target wasm32-wasip2 --release
```

The compiled WebAssembly file will be at: `target/wasm32-wasip2/release/github_connector.wasm`. Actions and triggers located in the folders explained below will automatically be included in the built file.

## Prerequisites

- **Rust 1.84.0+** with `wasm32-wasip2` target: `rustup target add wasm32-wasip2`
- **Ruby 3.4.2+** (optional, for RSpec tests)

## Action and Trigger Generation

Actions and triggers can be generated from OpenAPI specifications using [base-connector-tools](https://github.com/standout/base-connector-tools), or created manually. The generation tools are optional - you can build a connector without an OpenAPI schema.

### Install Tools

First, install the tools from GitHub (one-time setup):

```bash
cargo install --git https://github.com/standout/base-connector-tools.git --bin endpoints
cargo install --git https://github.com/standout/base-connector-tools.git --bin generate_action
cargo install --git https://github.com/standout/base-connector-tools.git --bin generate_trigger
```

### Discover Available Endpoints

List all operations from an OpenAPI spec:

```bash
endpoints <openapi_url_or_file>
```

- `openapi_url_or_file` - URL or path to OpenAPI specification file (local file paths can be relative or absolute)

**Examples:**
```bash
# From URL
endpoints https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json

# From local file (relative to current directory)
endpoints ./openapi.yaml
endpoints openapi.json

# From local file (absolute path)
endpoints /path/to/openapi.yaml
```

The output shows each operation with a round emoji (🔵) followed by the `operation_id`. Use the `operation_id` shown in this output when generating actions or triggers.

### Generate Action

Generate schemas and executor code for a specific action operation:

```bash
generate_action <openapi_url_or_file> <operation_id> [name]
```

- `openapi_url_or_file` - URL or path to OpenAPI specification file (local file paths can be relative or absolute)
- `operation_id` - The operation ID from the OpenAPI spec (shown after the round emoji in the `endpoints` output)
- `name` (optional) - Custom name for the action. If not provided, the operation ID will be used

**Examples:**
```bash
# From URL with default name (uses operation_id)
generate_action https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json repos/update

# From URL with custom name
generate_action https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json repos/update update_repository

# From local file (relative to current directory)
generate_action ./openapi.yaml repos/update update_repository
generate_action openapi.json repos/update

# From local file (absolute path)
generate_action /path/to/openapi.yaml repos/update
```

### Generate Trigger

Generate schemas and executor code for a specific trigger operation:

```bash
generate_trigger <openapi_url_or_file> <operation_id> [name]
```

- `openapi_url_or_file` - URL or path to OpenAPI specification file (local file paths can be relative or absolute)
- `operation_id` - The operation ID from the OpenAPI spec (shown after the round emoji in the `endpoints` output)
- `name` (optional) - Custom name for the trigger. If not provided, the operation ID will be used

**Examples:**
```bash
# From URL with default name (uses operation_id)
generate_trigger https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json issues/list-for-repo

# From URL with custom name
generate_trigger https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json issues/list-for-repo list_issues

# From local file (relative to current directory) with custom name
generate_trigger ./openapi.yaml issues/list-for-repo list_issues
generate_trigger openapi.json issues/list-for-repo

# From local file (absolute path)
generate_trigger /path/to/openapi.yaml issues/list-for-repo
```

Generated actions will be placed in `src/actions/{action_name}/` with:
- `action.rs` - Action executor code
- `base_input_schema.json` - Input validation schema
- `base_output_schema.json` - Output schema

Generated triggers will be placed in `src/triggers/{trigger_name}/` with:
- `fetch_events.rs` - Trigger executor code
- `input_schema.json` - Input validation schema
- `output_schema.json` - Output schema

**Important:** The generated folder name is used as the action/trigger name in Standout. You can specify a custom name when generating (using the optional `name` argument), or rename the folder after generation to a clear, descriptive name that represents what the action or trigger does. For example:
- `repos_create` → `create_repository`
- `issues_list-for-repo` → `list_issues`
- `repos_get` → `get_repository`
- `repos_update` → `update_repository`

After generation, rebuild to include the new action or trigger:
```bash
cargo build --target wasm32-wasip2 --release
```

### Customizing Actions

Generated actions may need customization for your specific use case:

#### Input Schema

The generated input schema (`base_input_schema.json`) includes all parameters from the OpenAPI spec, but you may not need all of them. You can remove fields that aren't needed for your use case

The generated schema is based on the OpenAPI specification, but it may not always match the actual API behavior. Review and test the schema against the real API to ensure accuracy. Add titles and descriptions to the fields if needed.

#### Output Schema

The generated output schema (`base_output_schema.json`) is based on the API response structure. You may need to adjust it if the API response structure differs from the OpenAPI spec

### Customizing Triggers

Generated triggers may need customization for your specific use case:

#### Store Data

The `store` is a JSON string that persists between trigger runs. Use it to track state like timestamps, pagination cursors, or last processed IDs. In `fetch_events.rs`:

```rust
// Parse existing store data
let store_data: Value = if context.store.is_empty() {
    serde_json::json!({})
} else {
    serde_json::from_str(&context.store)?
};

// Read state from store
let since = store_data.get("since")
    .and_then(|v| v.as_str())
    .map(|s| s.to_string());

// Update store with new state
let updated_store = serde_json::json!({
    "since": chrono::Utc::now().to_rfc3339(),
    "last_id": last_processed_id,
});
let store_string = serde_json::to_string(&updated_store)?;
```

#### Input Schema

The input schema (`input_schema.json`) is typically empty `{}` by default, but you can add fields in valid JSON Schema format:

```json
{
  "type": "object",
  "properties": {
    "filter": {
      "type": "string",
      "description": "Filter criteria"
    }
  }
}
```

#### Output Schema

The output schema (`output_schema.json`) should represent **one** of the objects (an event) returned by the API endpoint. The generated schema is based on the API response, but you may need to adjust it to match your specific event structure.

### Manual Creation

If you don't have an OpenAPI specification or prefer to create actions and triggers manually, you can create them directly following the required structure:

#### Action Structure

Create a directory `src/actions/{action_name}/` with:

- **`action.rs`** - Must export three functions:
  ```rust
  use crate::client::ApiClient;
  use crate::standout::app::types::{AppError, ActionContext};
  use serde_json::Value;

  /// Get the ApiClient from context
  fn client(context: &ActionContext) -> Result<ApiClient, AppError> {
      let connection_data: Value = serde_json::from_str(&context.connection.serialized_data)?;
      ApiClient::new(&connection_data)
  }

  /// Get the input data from context
  fn input_data(context: &ActionContext) -> Result<Value, AppError> {
      serde_json::from_str(&context.serialized_input)
  }

  /// Execute the action
  pub fn execute(context: ActionContext) -> Result<Value, AppError> {
      let client = client(&context)?;
      let input_data = input_data(&context)?;
      
      // Your action implementation
      // Make API calls using client.get(), client.post(), etc.
      // Return the response as Value
  }

  /// Get input schema
  pub fn input_schema(_context: &ActionContext) -> Result<Value, AppError> {
      // Load schema from file or return inline
      // If the schema should be fetched dynamically, use the ApiClient
      let schema = include_str!("base_input_schema.json");
      Ok(serde_json::from_str(schema)?)
  }

  /// Get output schema
  pub fn output_schema(_context: &ActionContext) -> Result<Value, AppError> {
      // Load schema from file or return inline
      // If the schema should be fetched dynamically, use the ApiClient
      let schema = include_str!("base_output_schema.json");
      Ok(serde_json::from_str(schema)?)
  }
  ```

- **`base_input_schema.json`** - JSON Schema for action input (JSON Schema Draft 2020-12 format)
- **`base_output_schema.json`** - JSON Schema for action output (JSON Schema Draft 2020-12 format)

#### Trigger Structure

Create a directory `src/triggers/{trigger_name}/` with:

- **`fetch_events.rs`** - Must export three functions:
  ```rust
  use crate::client::ApiClient;
  use crate::standout::app::types::{AppError, TriggerContext, TriggerResponse, TriggerEvent};
  use serde_json::Value;

  /// Get the ApiClient from context
  fn client(context: &TriggerContext) -> Result<ApiClient, AppError> {
      let connection_data: Value = serde_json::from_str(&context.connection.serialized_data)?;
      ApiClient::new(&connection_data)
  }

  /// Get the input data from context
  fn input_data(context: &TriggerContext) -> Result<Value, AppError> {
      if context.serialized_input.is_empty() {
          Ok(serde_json::json!({}))
      } else {
          serde_json::from_str(&context.serialized_input)
      }
  }

  /// Get the store data from context
  fn store_data(context: &TriggerContext) -> Result<Value, AppError> {
      if context.store.is_empty() {
          Ok(serde_json::json!({}))
      } else {
          serde_json::from_str(&context.store)
      }
  }

  /// Fetch events for the trigger
  pub fn fetch_events(context: TriggerContext) -> Result<TriggerResponse, AppError> {
      let api_client = client(&context)?;
      let input_data = input_data(&context)?;
      let store_data = store_data(&context)?;

      // Fetch data from API, process into events
      let events: Vec<TriggerEvent> = vec![]; // Your events here

      // Update store with new state
      let updated_store = serde_json::json!({});
      let store_string = serde_json::to_string(&updated_store)?;

      Ok(TriggerResponse {
          events,
          store: store_string,
      })
  }

  /// Get input schema
  pub fn input_schema(_context: &TriggerContext) -> Result<Value, AppError> {
      let schema = include_str!("input_schema.json");
      Ok(serde_json::from_str(schema)?)
  }

  /// Get output schema
  pub fn output_schema(_context: &TriggerContext) -> Result<Value, AppError> {
      let schema = include_str!("output_schema.json");
      Ok(serde_json::from_str(schema)?)
  }
  ```

- **`input_schema.json`** - JSON Schema for trigger input (typically empty)
- **`output_schema.json`** - JSON Schema for trigger output (represents one event object)

After creating the files, rebuild to include them:
```bash
cargo build --target wasm32-wasip2 --release
```

## Connection Configuration

The connector expects connection data at runtime. By default, it expects the following structure:

```json
{
  "base_url": "https://api.example.com",
  "headers": {
    "Authorization": "Bearer your-token",
    "Content-Type": "application/json"
  }
}
```

### Customizing Connection Data Structure

If your API's connection data uses a different structure (e.g., different field names, nested objects, or missing `base_url`/`headers`), you'll need to customize the `ApiClient::new()` method in `src/client.rs`.

**Example:** If your connection data looks like this:
```json
{
  "api_endpoint": "https://api.example.com",
  "auth": {
    "token": "your-token"
  }
}
```

You would modify `src/client.rs` to extract these fields:

```rust
pub fn new(connection_data: &Value) -> Result<Self, AppError> {
    let base_url = connection_data
        .get("api_endpoint")  // Changed from "base_url"
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError {
            code: ErrorCode::Misconfigured,
            message: "api_endpoint not found in connection data".to_string(),
        })?
        .to_string();

    let auth_obj = connection_data
        .get("auth")  // Changed from "headers"
        .and_then(|v| v.as_object())
        .ok_or_else(|| AppError {
            code: ErrorCode::Misconfigured,
            message: "auth not found in connection data".to_string(),
        })?;

    let mut headers = HashMap::new();
    if let Some(token) = auth_obj.get("token").and_then(|v| v.as_str()) {
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
    }
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    Ok(ApiClient { base_url, headers })
}
```

See `src/client.rs` for the current implementation.

## Testing

The connector uses RSpec for integration testing with WireMock to mock API responses.

### Setup

1. **Install Ruby dependencies:**
   ```bash
   bundle install
   ```

2. **Build the WASM module:**
   ```bash
   cargo build --release --target wasm32-wasip2
   ```

3. **Ensure Docker is running** (required for WireMock):
   ```bash
   docker --version
   ```

### Running Tests

The test suite automatically starts and stops WireMock using Docker Compose:

```bash
# Run all tests
bundle exec rspec

# Run tests for a specific file
bundle exec rspec spec/triggers/example_spec.rb

# Run a specific test
bundle exec rspec spec/triggers/example_spec.rb:16
```

### WireMock

Tests use [WireMock](https://wiremock.org/) running in Docker to mock external API responses. The mock server is automatically managed by the test suite:

- **Automatic startup**: WireMock starts before tests run
- **Automatic cleanup**: WireMock stops after tests complete
- **Manual control** (optional):
  ```bash
  ./scripts/test-setup.sh start   # Start WireMock
  ./scripts/test-setup.sh stop    # Stop WireMock
  ./scripts/test-setup.sh status  # Check status
  ```

WireMock runs on `http://localhost:8080` by default. The `TestHelper` module in `spec/test_helper.rb` provides utilities for configuring mock endpoints and creating test contexts.

## Development

```bash
# Format code
cargo fmt

# Lint code
cargo clippy --target wasm32-wasip2
```
