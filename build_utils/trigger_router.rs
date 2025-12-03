use super::BuildError;
use std::fs;
use std::path::Path;

/// Generate dynamic trigger routing
pub fn generate_trigger_routing(out_dir: &Path) -> Result<(), BuildError> {
    let triggers_dir = Path::new("src/triggers");
    let mut trigger_modules = Vec::new();

    // Scan the triggers directory for subdirectories with fetch_events.rs files
    if triggers_dir.exists() {
        for entry in fs::read_dir(triggers_dir)? {
            let path = entry?.path();
            if path.is_dir() {
                let trigger_name = path
                    .file_name()
                    .ok_or_else(|| {
                        BuildError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid trigger directory name",
                        ))
                    })?
                    .to_string_lossy()
                    .to_string();
                let trigger_path = path.join("fetch_events.rs");

                // Only include triggers that have a fetch_events.rs file
                if trigger_path.exists() {
                    trigger_modules.push(trigger_name);
                }
            }
        }
    }

    // Sort the modules for consistent ordering
    trigger_modules.sort();

    // Read the template
    let template =
        fs::read_to_string("build_templates/trigger_routing.rs.template").map_err(|e| {
            BuildError::Template(format!("Failed to read trigger_routing.rs.template: {}", e))
        })?;

    // Generate match arms for fetch_events function
    let mut fetch_events_match_arms = String::new();
    for trigger_name in &trigger_modules {
        fetch_events_match_arms.push_str(&format!(
            "        \"{}\" => triggers::{}::fetch_events(context),\n",
            trigger_name, trigger_name
        ));
    }

    // Generate match arms for input schema function
    let mut input_schema_match_arms = String::new();
    for trigger_name in &trigger_modules {
        input_schema_match_arms.push_str(&format!(
            "        \"{}\" => triggers::{}::input_schema(context),\n",
            trigger_name, trigger_name
        ));
    }

    // Generate match arms for output schema function
    let mut output_schema_match_arms = String::new();
    for trigger_name in &trigger_modules {
        output_schema_match_arms.push_str(&format!(
            "        \"{}\" => triggers::{}::output_schema(context),\n",
            trigger_name, trigger_name
        ));
    }

    // Generate available triggers list
    let mut available_triggers_list = String::new();
    for trigger_name in &trigger_modules {
        available_triggers_list.push_str(&format!("        \"{}\".to_string(),\n", trigger_name));
    }

    // Replace placeholders
    let routing_code = template
        .replace("{FETCH_EVENTS_MATCH_ARMS}", &fetch_events_match_arms)
        .replace("{INPUT_SCHEMA_MATCH_ARMS}", &input_schema_match_arms)
        .replace("{OUTPUT_SCHEMA_MATCH_ARMS}", &output_schema_match_arms)
        .replace("{AVAILABLE_TRIGGERS_LIST}", &available_triggers_list);

    // Write the routing file
    let routing_file = out_dir.join("trigger_routing.rs");
    fs::write(routing_file, routing_code)?;

    Ok(())
}
