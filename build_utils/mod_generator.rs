use super::BuildError;
use std::fs;
use std::path::Path;

/// Generate dynamic mod.rs for actions
pub fn generate_actions_mod_rs() -> Result<(), BuildError> {
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
    let template = fs::read_to_string("build_templates/actions_mod.rs.template").map_err(|e| {
        BuildError::Template(format!("Failed to read actions_mod.rs.template: {}", e))
    })?;

    // Generate action modules content
    let mut action_modules_content = String::new();
    for action_name in &action_modules {
        action_modules_content.push_str(&format!(
            "pub mod {} {{\n    include!(\"../actions/{}/action.rs\");\n}}\n\n",
            action_name, action_name
        ));
    }

    // Replace placeholder
    let mod_content = template.replace("{ACTION_MODULES}", &action_modules_content);

    // Write the mod.rs file
    let mod_rs_path = actions_dir.join("mod.rs");
    fs::write(mod_rs_path, mod_content)?;

    Ok(())
}

/// Generate dynamic mod.rs for triggers
pub fn generate_triggers_mod_rs() -> Result<(), BuildError> {
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
    let template = fs::read_to_string("build_templates/triggers_mod.rs.template").map_err(|e| {
        BuildError::Template(format!("Failed to read triggers_mod.rs.template: {}", e))
    })?;

    // Generate trigger modules content
    let mut trigger_modules_content = String::new();
    for trigger_name in &trigger_modules {
        trigger_modules_content.push_str(&format!(
            "pub mod {} {{\n    include!(\"../triggers/{}/fetch_events.rs\");\n}}\n\n",
            trigger_name, trigger_name
        ));
    }

    // Replace placeholder
    let mod_content = template.replace("{TRIGGER_MODULES}", &trigger_modules_content);

    // Write the mod.rs file
    let mod_rs_path = triggers_dir.join("mod.rs");
    fs::write(mod_rs_path, mod_content)?;

    Ok(())
}
