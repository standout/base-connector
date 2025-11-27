use std::fs;
use std::path::Path;

mod build_utils;
use build_utils::{
    collect_schema_files, generate_action_routing, generate_actions_mod_rs, generate_embedded_code,
    generate_trigger_routing, generate_triggers_mod_rs,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/actions");
    println!("cargo:rerun-if-changed=src/triggers");
    println!("cargo:rerun-if-changed=build_templates");

    let generated_dir = Path::new("src/schemas/generated");
    fs::create_dir_all(generated_dir)?;

    // Collect all schema files
    let schema_files = collect_schema_files()?;

    // Generate the embedded code
    generate_embedded_code(generated_dir, &schema_files)?;

    // Generate dynamic mod.rs for actions
    generate_actions_mod_rs()?;

    // Generate dynamic action routing
    generate_action_routing(generated_dir)?;

    // Generate dynamic mod.rs for triggers
    generate_triggers_mod_rs()?;

    // Generate dynamic trigger routing
    generate_trigger_routing(generated_dir)?;

    println!("cargo:warning=Build artifacts generated successfully");
    Ok(())
}
