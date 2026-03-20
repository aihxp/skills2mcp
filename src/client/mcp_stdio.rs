use rmcp::model::*;
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};
use tokio::process::Command;

use crate::error::{Result, SxmcError};

/// A client connected to an MCP server over stdio.
pub struct StdioClient {
    service: RunningService<RoleClient, ()>,
}

impl StdioClient {
    /// Connect to an MCP server by spawning a subprocess.
    pub async fn connect(command: &str, env_vars: &[(String, String)]) -> Result<Self> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(SxmcError::Other("Empty command".into()));
        }

        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        let transport = TokioChildProcess::new(cmd)
            .map_err(|e| SxmcError::McpError(format!("Failed to spawn: {}", e)))?;

        let service = ()
            .serve(transport)
            .await
            .map_err(|e| SxmcError::McpError(format!("Failed to initialize MCP session: {}", e)))?;

        Ok(Self { service })
    }

    /// List all available tools.
    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        let result = self
            .service
            .list_tools(None)
            .await
            .map_err(|e| SxmcError::McpError(format!("list_tools failed: {}", e)))?;
        Ok(result.tools)
    }

    /// Call a tool by name with JSON arguments.
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Map<String, serde_json::Value>,
    ) -> Result<CallToolResult> {
        let mut params = CallToolRequestParams::new(name.to_string());
        if !arguments.is_empty() {
            params.arguments = Some(arguments);
        }

        let result = self
            .service
            .call_tool(params)
            .await
            .map_err(|e| SxmcError::McpError(format!("call_tool failed: {}", e)))?;
        Ok(result)
    }

    /// List all available prompts.
    pub async fn list_prompts(&self) -> Result<Vec<Prompt>> {
        let result = self
            .service
            .list_prompts(None)
            .await
            .map_err(|e| SxmcError::McpError(format!("list_prompts failed: {}", e)))?;
        Ok(result.prompts)
    }

    /// Get a prompt by name.
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<GetPromptResult> {
        let mut params = GetPromptRequestParams::new(name);
        params.arguments = arguments;

        let result = self
            .service
            .get_prompt(params)
            .await
            .map_err(|e| SxmcError::McpError(format!("get_prompt failed: {}", e)))?;
        Ok(result)
    }

    /// List all available resources.
    pub async fn list_resources(&self) -> Result<Vec<Resource>> {
        let result = self
            .service
            .list_resources(None)
            .await
            .map_err(|e| SxmcError::McpError(format!("list_resources failed: {}", e)))?;
        Ok(result.resources)
    }

    /// Read a resource by URI.
    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult> {
        let params = ReadResourceRequestParams::new(uri);
        let result = self
            .service
            .read_resource(params)
            .await
            .map_err(|e| SxmcError::McpError(format!("read_resource failed: {}", e)))?;
        Ok(result)
    }

    /// Shut down the connection.
    pub async fn close(self) -> Result<()> {
        self.service
            .cancel()
            .await
            .map_err(|e| SxmcError::McpError(format!("Failed to close: {}", e)))?;
        Ok(())
    }
}
