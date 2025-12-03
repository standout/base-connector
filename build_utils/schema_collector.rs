use super::BuildError;
use std::collections::HashMap;
use std::fs;

/// Scan and collect schema files from directories
pub fn collect_schema_files() -> Result<HashMap<String, String>, BuildError> {
    let mut schema_files = HashMap::new();

    // Collect action schemas
    for schema_dir in ["src/actions"] {
        for entry in fs::read_dir(schema_dir)? {
            let path = entry?.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .ok_or_else(|| {
                        BuildError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid directory name",
                        ))
                    })?
                    .to_string_lossy()
                    .to_string();

                // Read both input and output schemas
                for schema_type in ["input", "output"] {
                    let schema_path = path.join(format!("base_{}_schema.json", schema_type));
                    if schema_path.exists() {
                        let content = fs::read_to_string(&schema_path)?;
                        schema_files.insert(format!("{}_{}", name, schema_type), content);
                    }
                }
            }
        }
    }

    // Collect trigger schemas
    for schema_dir in ["src/triggers"] {
        if fs::metadata(schema_dir).is_err() {
            continue;
        }
        for entry in fs::read_dir(schema_dir)? {
            let path = entry?.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .ok_or_else(|| {
                        BuildError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid directory name",
                        ))
                    })?
                    .to_string_lossy()
                    .to_string();

                // Read both input and output schemas (triggers use input_schema.json and output_schema.json)
                for schema_type in ["input", "output"] {
                    let schema_path = path.join(format!("{}_schema.json", schema_type));
                    if schema_path.exists() {
                        let content = fs::read_to_string(&schema_path)?;
                        schema_files.insert(format!("{}_{}", name, schema_type), content);
                    }
                }
            }
        }
    }

    Ok(schema_files)
}
