use std::path::Path;

use crate::client::openapi::OpenApiSpec;
use crate::error::{Result, SxmcError};

/// Generate a SKILL.md from an OpenAPI spec.
pub async fn generate_from_openapi(
    source: &str,
    output_dir: &Path,
    auth_headers: &[(String, String)],
) -> Result<std::path::PathBuf> {
    let spec = OpenApiSpec::load(source, auth_headers).await?;
    let commands = spec.commands();

    let skill_name = sanitize_name(&spec.title);
    let skill_dir = output_dir.join(&skill_name);
    std::fs::create_dir_all(&skill_dir)
        .map_err(|e| SxmcError::Other(format!("Failed to create skill dir: {}", e)))?;

    let mut body = String::new();
    body.push_str(&format!("# {}\n\n", spec.title));
    body.push_str("This skill was auto-generated from an OpenAPI spec.\n\n");
    body.push_str("## Available Operations\n\n");

    for cmd in &commands {
        body.push_str(&format!("### {}\n", cmd.name));
        if !cmd.description.is_empty() {
            body.push_str(&format!("{}\n", cmd.description));
        }
        if !cmd.params.is_empty() {
            body.push_str("\nParameters:\n");
            for param in &cmd.params {
                let req = if param.required { " (required)" } else { "" };
                body.push_str(&format!(
                    "- `{}`{}: {}\n",
                    param.name, req, param.description
                ));
            }
        }
        body.push('\n');
    }

    // Build argument hint from all unique parameter names
    let mut all_params: Vec<String> = commands
        .iter()
        .flat_map(|c| {
            c.params.iter().map(|p| {
                if p.required {
                    format!("<{}>", p.name)
                } else {
                    format!("[{}]", p.name)
                }
            })
        })
        .collect();
    all_params.sort();
    all_params.dedup();

    let skill_md = format!(
        "---\nname: {}\ndescription: \"API client for {}\"\nargument-hint: \"{}\"\n---\n\n{}",
        skill_name,
        spec.title,
        all_params.join(" "),
        body
    );

    let skill_path = skill_dir.join("SKILL.md");
    std::fs::write(&skill_path, skill_md)
        .map_err(|e| SxmcError::Other(format!("Failed to write SKILL.md: {}", e)))?;

    Ok(skill_dir)
}

fn sanitize_name(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Petstore API"), "petstore-api");
        assert_eq!(sanitize_name("My Cool API v2"), "my-cool-api-v2");
        assert_eq!(sanitize_name("  Spaces  "), "spaces");
    }
}
