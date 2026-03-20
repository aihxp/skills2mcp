use clap::ValueEnum;
use rmcp::model::{
    CallToolResult, GetPromptResult, PromptMessageContent, PromptMessageRole, ReadResourceResult,
    ResourceContents,
};
use serde_json::{Map, Value};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum StructuredOutputFormat {
    Json,
    JsonPretty,
    Toon,
}

pub fn resolve_structured_format(
    format: Option<StructuredOutputFormat>,
    pretty: bool,
) -> StructuredOutputFormat {
    format.unwrap_or(if pretty {
        StructuredOutputFormat::JsonPretty
    } else {
        StructuredOutputFormat::Json
    })
}

pub fn format_structured_value(value: &Value, format: StructuredOutputFormat) -> String {
    match format {
        StructuredOutputFormat::Json => value.to_string(),
        StructuredOutputFormat::JsonPretty => {
            serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
        }
        StructuredOutputFormat::Toon => encode_toon(value),
    }
}

/// Format a CallToolResult for display.
pub fn format_tool_result(result: &CallToolResult, pretty: bool) -> String {
    let texts: Vec<String> = result
        .content
        .iter()
        .filter_map(|c| c.raw.as_text().map(|t| t.text.clone()))
        .collect();

    let output = texts.join("\n");

    if pretty {
        // Try to parse as JSON and pretty-print
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&output) {
            if let Ok(pretty_str) = serde_json::to_string_pretty(&val) {
                return pretty_str;
            }
        }
    }

    output
}

/// Format a GetPromptResult for display.
pub fn format_prompt_result(result: &GetPromptResult, pretty: bool) -> String {
    if pretty {
        return serde_json::to_string_pretty(result)
            .unwrap_or_else(|_| serde_json::to_string(result).unwrap_or_default());
    }

    if result.messages.len() == 1 {
        if let Some(message) = result.messages.first() {
            if let PromptMessageContent::Text { text } = &message.content {
                return text.clone();
            }
        }
    }

    let messages: Vec<String> = result
        .messages
        .iter()
        .map(|message| {
            let role = match message.role {
                PromptMessageRole::User => "user",
                PromptMessageRole::Assistant => "assistant",
            };
            let content = match &message.content {
                PromptMessageContent::Text { text } => text.clone(),
                _ => serde_json::to_string_pretty(&message.content).unwrap_or_else(|_| {
                    serde_json::to_string(&message.content).unwrap_or_default()
                }),
            };
            format!("[{}]\n{}", role, content)
        })
        .collect();

    messages.join("\n\n")
}

/// Format a ReadResourceResult for display.
pub fn format_resource_result(result: &ReadResourceResult, pretty: bool) -> String {
    if pretty {
        return serde_json::to_string_pretty(result)
            .unwrap_or_else(|_| serde_json::to_string(result).unwrap_or_default());
    }

    let contents: Vec<String> = result
        .contents
        .iter()
        .map(|content| match content {
            ResourceContents::TextResourceContents { text, .. } => text.clone(),
            ResourceContents::BlobResourceContents { blob, .. } => blob.clone(),
        })
        .collect();

    contents.join("\n\n")
}

/// Format MCP tools as a list for display.
pub fn format_tool_list(tools: &[rmcp::model::Tool], search: Option<&str>) -> String {
    let mut lines = Vec::new();

    for tool in tools {
        let name = tool.name.as_ref();
        let desc = tool.description.as_deref().unwrap_or("");

        if let Some(pattern) = search {
            let pattern_lower = pattern.to_lowercase();
            if !name.to_lowercase().contains(&pattern_lower)
                && !desc.to_lowercase().contains(&pattern_lower)
            {
                continue;
            }
        }

        lines.push(format!("  {}", name));
        if !desc.is_empty() {
            lines.push(format!("    {}", desc));
        }
    }

    if lines.is_empty() {
        if search.is_some() {
            return "No matching tools found.".to_string();
        }
        return "No tools available.".to_string();
    }

    format!("Tools ({}):\n{}", tools.len(), lines.join("\n"))
}

/// Format MCP prompts as a list for display.
pub fn format_prompt_list(prompts: &[rmcp::model::Prompt]) -> String {
    let mut lines = Vec::new();

    for prompt in prompts {
        lines.push(format!("  {}", prompt.name));
        if let Some(ref desc) = prompt.description {
            lines.push(format!("    {}", desc));
        }
    }

    if lines.is_empty() {
        return "No prompts available.".to_string();
    }

    format!("Prompts ({}):\n{}", prompts.len(), lines.join("\n"))
}

/// Format MCP resources as a list for display.
pub fn format_resource_list(resources: &[rmcp::model::Resource]) -> String {
    let mut lines = Vec::new();

    for resource in resources {
        lines.push(format!("  {} ({})", resource.name, resource.uri));
        if let Some(ref desc) = resource.description {
            lines.push(format!("    {}", desc));
        }
    }

    if lines.is_empty() {
        return "No resources available.".to_string();
    }

    format!("Resources ({}):\n{}", resources.len(), lines.join("\n"))
}

fn encode_toon(value: &Value) -> String {
    match value {
        Value::Object(map) => render_object(map, 0),
        Value::Array(items) => render_array(items, 0, None),
        _ => render_scalar(value),
    }
}

fn render_object(map: &Map<String, Value>, indent: usize) -> String {
    if map.is_empty() {
        return "{}".to_string();
    }

    let mut lines = Vec::new();
    for (key, value) in map {
        match value {
            Value::Object(child) => {
                lines.push(format!("{}{}:", indent_str(indent), key));
                lines.push(render_object(child, indent + 2));
            }
            Value::Array(items) => {
                if let Some(table) = render_tabular_array(items, indent, Some(key)) {
                    lines.push(table);
                } else if items.iter().all(is_primitive) {
                    let joined = items
                        .iter()
                        .map(render_scalar)
                        .collect::<Vec<_>>()
                        .join(", ");
                    lines.push(format!(
                        "{}{}[{}]: {}",
                        indent_str(indent),
                        key,
                        items.len(),
                        joined
                    ));
                } else {
                    lines.push(format!(
                        "{}{}: {}",
                        indent_str(indent),
                        key,
                        Value::Array(items.clone())
                    ));
                }
            }
            _ => lines.push(format!(
                "{}{}: {}",
                indent_str(indent),
                key,
                render_scalar(value)
            )),
        }
    }

    lines.join("\n")
}

fn render_array(items: &[Value], indent: usize, name: Option<&str>) -> String {
    if let Some(table) = render_tabular_array(items, indent, name) {
        return table;
    }

    if items.is_empty() {
        return "[]".to_string();
    }

    if items.iter().all(is_primitive) {
        let joined = items
            .iter()
            .map(render_scalar)
            .collect::<Vec<_>>()
            .join(", ");
        return match name {
            Some(name) => format!(
                "{}{}[{}]: {}",
                indent_str(indent),
                name,
                items.len(),
                joined
            ),
            None => format!("[{}]: {}", items.len(), joined),
        };
    }

    Value::Array(items.to_vec()).to_string()
}

fn render_tabular_array(items: &[Value], indent: usize, name: Option<&str>) -> Option<String> {
    let headers = tabular_headers(items)?;
    let header_prefix = match name {
        Some(name) => format!(
            "{}{}[{}]{{{}}}:",
            indent_str(indent),
            name,
            items.len(),
            headers.join(",")
        ),
        None => format!("[{}]{{{}}}:", items.len(), headers.join(",")),
    };

    let mut lines = vec![header_prefix];
    for item in items {
        let object = item.as_object()?;
        let row = headers
            .iter()
            .map(|key| {
                object
                    .get(key)
                    .map(render_scalar)
                    .unwrap_or_else(|| "null".to_string())
            })
            .collect::<Vec<_>>()
            .join(",");
        lines.push(format!("{}{}", indent_str(indent + 2), row));
    }

    Some(lines.join("\n"))
}

fn tabular_headers(items: &[Value]) -> Option<Vec<String>> {
    if items.is_empty() {
        return None;
    }

    let first = items.first()?.as_object()?;
    if first.is_empty() {
        return None;
    }

    let headers: Vec<String> = first.keys().cloned().collect();

    for item in items {
        let object = item.as_object()?;
        if object.len() != headers.len() {
            return None;
        }
        if !headers.iter().all(|key| object.contains_key(key)) {
            return None;
        }
        if !object.values().all(is_primitive) {
            return None;
        }
    }

    Some(headers)
}

fn render_scalar(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => serde_json::to_string(string).unwrap_or_else(|_| "\"\"".into()),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

fn is_primitive(value: &Value) -> bool {
    matches!(
        value,
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
    )
}

fn indent_str(indent: usize) -> &'static str {
    const SPACES: &str = "                                ";
    &SPACES[..indent.min(SPACES.len())]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_structured_value_json_compact() {
        let value = serde_json::json!({"name": "Ada", "count": 2});
        let rendered = format_structured_value(&value, StructuredOutputFormat::Json);
        let reparsed: Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(reparsed, value);
    }

    #[test]
    fn test_format_structured_value_toon_object() {
        let value = serde_json::json!({
            "name": "Ada",
            "active": true,
            "stats": {
                "count": 2
            }
        });

        let output = format_structured_value(&value, StructuredOutputFormat::Toon);
        assert!(output.contains(r#"name: "Ada""#));
        assert!(output.contains("active: true"));
        assert!(output.contains("stats:"));
        assert!(output.contains("  count: 2"));
    }

    #[test]
    fn test_format_structured_value_toon_tabular_array() {
        let value = serde_json::json!({
            "pets": [
                {"id": 1, "name": "Mochi"},
                {"id": 2, "name": "Pixel"}
            ]
        });

        let output = format_structured_value(&value, StructuredOutputFormat::Toon);
        assert!(output.contains("pets[2]{id,name}:"));
        assert!(output.contains(r#"  1,"Mochi""#));
        assert!(output.contains(r#"  2,"Pixel""#));
    }
}
