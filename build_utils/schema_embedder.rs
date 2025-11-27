use super::BuildError;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Generate embedded schemas code
pub fn generate_embedded_code(
    out_dir: &Path,
    schema_files: &HashMap<String, String>,
) -> Result<(), BuildError> {
    // Convert schema files to bytes (store as raw JSON string)
    let schema_files_json = serde_json::to_string(schema_files)?;
    let schema_files_bytes = schema_files_json.as_bytes();

    // Read the template
    let template =
        fs::read_to_string("build_templates/embedded_schemas.rs.template").map_err(|e| {
            BuildError::Template(format!(
                "Failed to read embedded_schemas.rs.template: {}",
                e
            ))
        })?;

    // Replace placeholders
    let embedded_code =
        template.replace("{SCHEMA_FILES_BYTES}", &format!("{:?}", schema_files_bytes));

    let embedded_file = Path::new(&out_dir).join("embedded_schemas.rs");
    fs::write(embedded_file, embedded_code)?;

    Ok(())
}
