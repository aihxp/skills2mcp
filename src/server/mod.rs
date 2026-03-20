pub mod handler;

use std::path::PathBuf;

use crate::error::Result;
use crate::skills::discovery;
use crate::skills::parser;

use self::handler::SkillsServer;

/// Build a SkillsServer from skill search paths.
pub fn build_server(paths: &[PathBuf]) -> Result<SkillsServer> {
    let skill_dirs = discovery::discover_skills(paths)?;
    let mut skills = Vec::new();

    for dir in &skill_dirs {
        let source = dir.parent().and_then(|p| p.to_str()).unwrap_or("unknown");
        match parser::parse_skill(dir, source) {
            Ok(skill) => {
                eprintln!("[sxmc] Loaded skill: {}", skill.name);
                skills.push(skill);
            }
            Err(e) => {
                eprintln!("[sxmc] Warning: failed to parse {}: {}", dir.display(), e);
            }
        }
    }

    eprintln!(
        "[sxmc] Loaded {} skills with {} tools and {} resources",
        skills.len(),
        skills.iter().map(|s| s.scripts.len()).sum::<usize>(),
        skills.iter().map(|s| s.references.len()).sum::<usize>(),
    );

    Ok(SkillsServer::new(skills))
}

/// Run the MCP server over stdio.
pub async fn serve_stdio(paths: &[PathBuf]) -> Result<()> {
    let server = build_server(paths)?;
    let transport = rmcp::transport::stdio();

    let service = rmcp::ServiceExt::serve(server, transport)
        .await
        .map_err(|e| crate::error::SxmcError::McpError(e.to_string()))?;

    service
        .waiting()
        .await
        .map_err(|e| crate::error::SxmcError::McpError(e.to_string()))?;

    Ok(())
}
