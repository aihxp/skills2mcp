use std::collections::HashMap;

use serde_json::Value;

use crate::client::commands::{CommandDef, ParamDef, ParamType};
use crate::error::{Result, SxmcError};

/// A GraphQL operation (query or mutation) extracted via introspection.
#[derive(Debug, Clone)]
pub struct GraphQLOperation {
    pub name: String,
    pub description: String,
    pub kind: GraphQLOpKind,
    pub args: Vec<GraphQLArg>,
    pub returns_composite: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GraphQLOpKind {
    Query,
    Mutation,
}

#[derive(Debug, Clone)]
pub struct GraphQLArg {
    pub name: String,
    pub description: String,
    pub type_name: String,
    pub required: bool,
}

/// A client for a GraphQL endpoint.
pub struct GraphQLClient {
    url: String,
    client: reqwest::Client,
    operations: Vec<GraphQLOperation>,
}

impl GraphQLClient {
    /// Connect to a GraphQL endpoint and introspect its schema.
    pub async fn connect(url: &str, auth_headers: &[(String, String)]) -> Result<Self> {
        let mut header_map = reqwest::header::HeaderMap::new();
        for (key, value) in auth_headers {
            if let (Ok(name), Ok(val)) = (
                key.parse::<reqwest::header::HeaderName>(),
                value.parse::<reqwest::header::HeaderValue>(),
            ) {
                header_map.insert(name, val);
            }
        }

        let client = reqwest::Client::builder()
            .default_headers(header_map)
            .build()
            .map_err(|e| SxmcError::Other(format!("Failed to build HTTP client: {}", e)))?;

        let operations = introspect(&client, url).await?;

        Ok(Self {
            url: url.to_string(),
            client,
            operations,
        })
    }

    /// Convert operations to CommandDef objects.
    pub fn commands(&self) -> Vec<CommandDef> {
        self.operations
            .iter()
            .map(|op| {
                let params = op
                    .args
                    .iter()
                    .map(|a| ParamDef {
                        name: a.name.clone(),
                        description: if a.description.is_empty() {
                            format!("{} ({})", a.name, a.type_name)
                        } else {
                            a.description.clone()
                        },
                        param_type: graphql_type_to_param(&a.type_name),
                        required: a.required,
                        default: None,
                    })
                    .collect();

                let prefix = match op.kind {
                    GraphQLOpKind::Query => "query",
                    GraphQLOpKind::Mutation => "mutation",
                };

                CommandDef {
                    name: op.name.clone(),
                    description: if op.description.is_empty() {
                        format!("{}: {}", prefix, op.name)
                    } else {
                        op.description.clone()
                    },
                    params,
                }
            })
            .collect()
    }

    /// Execute a GraphQL operation by name.
    pub async fn execute(
        &self,
        operation_name: &str,
        args: &HashMap<String, String>,
    ) -> Result<Value> {
        let op = self
            .operations
            .iter()
            .find(|o| o.name == operation_name)
            .ok_or_else(|| SxmcError::Other(format!("Operation not found: {}", operation_name)))?;

        // Build variables from args, attempting JSON parse for each value
        let mut variables = serde_json::Map::new();
        for arg in &op.args {
            if let Some(value) = args.get(&arg.name) {
                let val =
                    serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.clone()));
                variables.insert(arg.name.clone(), val);
            }
        }

        // Build the query string
        let query = build_query(op);

        let body = serde_json::json!({
            "query": query,
            "variables": variables,
        });

        let response = self
            .client
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .map_err(|e| SxmcError::Other(format!("GraphQL request failed: {}", e)))?;

        let result: Value = response
            .json()
            .await
            .map_err(|e| SxmcError::Other(format!("Failed to parse GraphQL response: {}", e)))?;

        // Return the data portion, or the full response if there are errors
        if result.get("errors").is_some() {
            Ok(result)
        } else {
            Ok(result.get("data").cloned().unwrap_or(result))
        }
    }

    /// List operations, optionally filtered.
    pub fn list_operations(&self, search: Option<&str>) -> Vec<&GraphQLOperation> {
        self.operations
            .iter()
            .filter(|op| {
                if let Some(pattern) = search {
                    let p = pattern.to_lowercase();
                    op.name.to_lowercase().contains(&p)
                        || op.description.to_lowercase().contains(&p)
                } else {
                    true
                }
            })
            .collect()
    }
}

/// Run introspection query against a GraphQL endpoint.
async fn introspect(client: &reqwest::Client, url: &str) -> Result<Vec<GraphQLOperation>> {
    let query = r#"
    {
        __schema {
            queryType { name }
            mutationType { name }
            types {
                name
                kind
                fields {
                    name
                    description
                    type {
                        name
                        kind
                        ofType { name kind ofType { name kind ofType { name kind } } }
                    }
                    args {
                        name
                        description
                        type {
                            name
                            kind
                            ofType { name kind ofType { name kind ofType { name kind } } }
                        }
                    }
                }
            }
        }
    }
    "#;

    let body = serde_json::json!({ "query": query });

    let response = client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| SxmcError::Other(format!("Introspection request failed: {}", e)))?;

    let result: Value = response
        .json()
        .await
        .map_err(|e| SxmcError::Other(format!("Failed to parse introspection response: {}", e)))?;

    let schema = result
        .pointer("/data/__schema")
        .ok_or_else(|| SxmcError::Other("Invalid introspection response".into()))?;

    let query_type_name = schema
        .pointer("/queryType/name")
        .and_then(|v| v.as_str())
        .unwrap_or("Query");

    let mutation_type_name = schema
        .pointer("/mutationType/name")
        .and_then(|v| v.as_str());

    let types = schema
        .get("types")
        .and_then(|v| v.as_array())
        .ok_or_else(|| SxmcError::Other("No types in introspection".into()))?;

    let mut operations = Vec::new();

    for type_def in types {
        let type_name = type_def.get("name").and_then(|v| v.as_str()).unwrap_or("");

        let kind = if type_name == query_type_name {
            GraphQLOpKind::Query
        } else if mutation_type_name == Some(type_name) {
            GraphQLOpKind::Mutation
        } else {
            continue;
        };

        if let Some(fields) = type_def.get("fields").and_then(|v| v.as_array()) {
            for field in fields {
                let name = field
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Skip introspection fields
                if name.starts_with("__") {
                    continue;
                }

                let description = field
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let args = field
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| extract_args(arr))
                    .unwrap_or_default();

                operations.push(GraphQLOperation {
                    name,
                    description,
                    kind: kind.clone(),
                    args,
                    returns_composite: field
                        .get("type")
                        .map(is_composite_output_type)
                        .unwrap_or(false),
                });
            }
        }
    }

    operations.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(operations)
}

fn extract_args(args: &[Value]) -> Vec<GraphQLArg> {
    args.iter()
        .filter_map(|a| {
            let name = a.get("name")?.as_str()?.to_string();
            let description = a
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let type_info = a.get("type")?;
            let (type_name, required) = resolve_graphql_type(type_info);

            Some(GraphQLArg {
                name,
                description,
                type_name,
                required,
            })
        })
        .collect()
}

/// Resolve a GraphQL type, unwrapping NON_NULL and LIST wrappers.
fn resolve_graphql_type(type_val: &Value) -> (String, bool) {
    let kind = type_val.get("kind").and_then(|v| v.as_str()).unwrap_or("");

    if kind == "NON_NULL" {
        if let Some(of_type) = type_val.get("ofType") {
            let (inner, _) = resolve_graphql_type(of_type);
            return (inner, true);
        }
    }

    if kind == "LIST" {
        if let Some(of_type) = type_val.get("ofType") {
            let (inner, _) = resolve_graphql_type(of_type);
            return (format!("[{}]", inner), false);
        }
    }

    let name = type_val
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("String")
        .to_string();

    (name, false)
}

fn is_composite_output_type(type_val: &Value) -> bool {
    let kind = type_val.get("kind").and_then(|v| v.as_str()).unwrap_or("");

    match kind {
        "NON_NULL" | "LIST" => type_val
            .get("ofType")
            .map(is_composite_output_type)
            .unwrap_or(false),
        "OBJECT" | "INTERFACE" | "UNION" => true,
        _ => false,
    }
}

/// Map GraphQL type names to ParamType.
fn graphql_type_to_param(type_name: &str) -> ParamType {
    let clean = type_name.trim_start_matches('[').trim_end_matches(']');
    match clean {
        "Int" => ParamType::Integer,
        "Float" => ParamType::Number,
        "Boolean" => ParamType::Boolean,
        _ => ParamType::String,
    }
}

/// Build a simple query string for an operation.
fn build_query(op: &GraphQLOperation) -> String {
    let prefix = match op.kind {
        GraphQLOpKind::Query => "query",
        GraphQLOpKind::Mutation => "mutation",
    };

    let selection = if op.returns_composite {
        " { __typename }"
    } else {
        ""
    };

    if op.args.is_empty() {
        return format!("{} {{ {}{} }}", prefix, op.name, selection);
    }

    // Build variable declarations
    let var_decls: Vec<String> = op
        .args
        .iter()
        .map(|a| {
            let gql_type = if a.required {
                format!("{}!", a.type_name)
            } else {
                a.type_name.clone()
            };
            format!("${}: {}", a.name, gql_type)
        })
        .collect();

    // Build argument passing
    let arg_pass: Vec<String> = op
        .args
        .iter()
        .map(|a| format!("{}: ${}", a.name, a.name))
        .collect();

    format!(
        "{} Op({}) {{ {}({}){} }}",
        prefix,
        var_decls.join(", "),
        op.name,
        arg_pass.join(", "),
        selection
    )
}

/// Format GraphQL operations for display.
pub fn format_graphql_list(ops: &[&GraphQLOperation], search: Option<&str>) -> String {
    let filtered: Vec<&&GraphQLOperation> = if let Some(pattern) = search {
        let p = pattern.to_lowercase();
        ops.iter()
            .filter(|op| {
                op.name.to_lowercase().contains(&p) || op.description.to_lowercase().contains(&p)
            })
            .collect()
    } else {
        ops.iter().collect()
    };

    if filtered.is_empty() {
        if search.is_some() {
            return "No matching operations found.".to_string();
        }
        return "No operations available.".to_string();
    }

    let mut lines = Vec::new();
    for op in &filtered {
        let kind_str = match op.kind {
            GraphQLOpKind::Query => "Q",
            GraphQLOpKind::Mutation => "M",
        };
        lines.push(format!("  {} [{}]", op.name, kind_str));
        if !op.description.is_empty() {
            lines.push(format!("    {}", op.description));
        }
    }

    format!("Operations ({}):\n{}", filtered.len(), lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_graphql_type_simple() {
        let t: Value = serde_json::json!({"name": "String", "kind": "SCALAR"});
        assert_eq!(resolve_graphql_type(&t), ("String".to_string(), false));
    }

    #[test]
    fn test_resolve_graphql_type_non_null() {
        let t: Value = serde_json::json!({
            "kind": "NON_NULL",
            "ofType": {"name": "Int", "kind": "SCALAR"}
        });
        assert_eq!(resolve_graphql_type(&t), ("Int".to_string(), true));
    }

    #[test]
    fn test_resolve_graphql_type_list() {
        let t: Value = serde_json::json!({
            "kind": "LIST",
            "ofType": {"name": "String", "kind": "SCALAR"}
        });
        assert_eq!(resolve_graphql_type(&t), ("[String]".to_string(), false));
    }

    #[test]
    fn test_build_query_no_args() {
        let op = GraphQLOperation {
            name: "users".to_string(),
            description: "".to_string(),
            kind: GraphQLOpKind::Query,
            args: vec![],
            returns_composite: false,
        };
        assert_eq!(build_query(&op), "query { users }");
    }

    #[test]
    fn test_build_query_with_args() {
        let op = GraphQLOperation {
            name: "user".to_string(),
            description: "".to_string(),
            kind: GraphQLOpKind::Query,
            args: vec![GraphQLArg {
                name: "id".to_string(),
                description: "".to_string(),
                type_name: "ID".to_string(),
                required: true,
            }],
            returns_composite: false,
        };
        assert_eq!(build_query(&op), "query Op($id: ID!) { user(id: $id) }");
    }

    #[test]
    fn test_build_query_with_composite_return_type() {
        let op = GraphQLOperation {
            name: "user".to_string(),
            description: "".to_string(),
            kind: GraphQLOpKind::Query,
            args: vec![GraphQLArg {
                name: "id".to_string(),
                description: "".to_string(),
                type_name: "ID".to_string(),
                required: true,
            }],
            returns_composite: true,
        };
        assert_eq!(
            build_query(&op),
            "query Op($id: ID!) { user(id: $id) { __typename } }"
        );
    }
}
