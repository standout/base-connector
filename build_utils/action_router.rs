use super::BuildError;
use std::fs;
use std::path::Path;

/// Generate dynamic action routing
pub fn generate_action_routing(out_dir: &Path) -> Result<(), BuildError> {
    let actions_dir = Path::new("src/actions");
    let mut action_modules = Vec::new();

    // Scan the actions directory for subdirectories with action.rs files
    if actions_dir.exists() {
        for entry in fs::read_dir(actions_dir)? {
            let path = entry?.path();
            if path.is_dir() {
                let action_name = path
                    .file_name()
                    .ok_or_else(|| {
                        BuildError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid action directory name",
                        ))
                    })?
                    .to_string_lossy()
                    .to_string();
                let action_path = path.join("action.rs");

                // Only include actions that have an action.rs file
                if action_path.exists() {
                    action_modules.push(action_name);
                }
            }
        }
    }

    // Sort the modules for consistent ordering
    action_modules.sort();

    // Read the template
    let template =
        fs::read_to_string("build_templates/action_routing.rs.template").map_err(|e| {
            BuildError::Template(format!("Failed to read action_routing.rs.template: {}", e))
        })?;

    // Generate match arms for execute function
    let mut execute_match_arms = String::new();
    for action_name in &action_modules {
        execute_match_arms.push_str(&format!(
            "        \"{}\" => actions::{}::execute(context),\n",
            action_name, action_name
        ));
    }

    // Generate match arms for input schema function
    let mut input_schema_match_arms = String::new();
    for action_name in &action_modules {
        input_schema_match_arms.push_str(&format!(
            "        \"{}\" => actions::{}::input_schema(context),\n",
            action_name, action_name
        ));
    }

    // Generate match arms for output schema function
    let mut output_schema_match_arms = String::new();
    for action_name in &action_modules {
        output_schema_match_arms.push_str(&format!(
            "        \"{}\" => actions::{}::output_schema(context),\n",
            action_name, action_name
        ));
    }

    // Generate available actions list
    let mut available_actions_list = String::new();
    for action_name in &action_modules {
        available_actions_list.push_str(&format!("        \"{}\".to_string(),\n", action_name));
    }

    // Replace placeholders
    let routing_code = template
        .replace("{EXECUTE_MATCH_ARMS}", &execute_match_arms)
        .replace("{INPUT_SCHEMA_MATCH_ARMS}", &input_schema_match_arms)
        .replace("{OUTPUT_SCHEMA_MATCH_ARMS}", &output_schema_match_arms)
        .replace("{AVAILABLE_ACTIONS_LIST}", &available_actions_list);

    // Write the routing file
    let routing_file = out_dir.join("action_routing.rs");
    fs::write(routing_file, routing_code)?;

    Ok(())
}
