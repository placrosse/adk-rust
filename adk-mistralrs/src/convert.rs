//! Conversion layer between ADK types and mistral.rs types.

use adk_core::{Content, Part};
use indexmap::IndexMap;
use serde_json::Value;

use crate::error::{MistralRsError, Result};

/// Convert ADK Content to mistral.rs message format
pub fn content_to_message(content: &Content) -> IndexMap<String, Value> {
    let mut message = IndexMap::new();

    // Convert role - map ADK roles to OpenAI-style roles
    let role = match content.role.as_str() {
        "user" => "user",
        "model" | "assistant" => "assistant",
        "system" => "system",
        "tool" | "function" => "tool",
        other => other, // Pass through unknown roles
    };
    message.insert("role".to_string(), Value::String(role.to_string()));

    // Convert content parts to text
    let text_parts: Vec<String> = content
        .parts
        .iter()
        .filter_map(|part| match part {
            Part::Text { text } => Some(text.clone()),
            _ => None,
        })
        .collect();

    if !text_parts.is_empty() {
        message.insert(
            "content".to_string(),
            Value::String(text_parts.join("\n")),
        );
    }

    // Handle function calls
    let tool_calls: Vec<Value> = content
        .parts
        .iter()
        .filter_map(|part| match part {
            Part::FunctionCall { id, name, args } => {
                let mut call = serde_json::Map::new();
                if let Some(id) = id {
                    call.insert("id".to_string(), Value::String(id.clone()));
                }
                call.insert("type".to_string(), Value::String("function".to_string()));

                let mut function = serde_json::Map::new();
                function.insert("name".to_string(), Value::String(name.clone()));
                function.insert(
                    "arguments".to_string(),
                    Value::String(serde_json::to_string(args).unwrap_or_default()),
                );
                call.insert("function".to_string(), Value::Object(function));

                Some(Value::Object(call))
            }
            _ => None,
        })
        .collect();

    if !tool_calls.is_empty() {
        message.insert("tool_calls".to_string(), Value::Array(tool_calls));
    }

    // Handle function responses
    for part in &content.parts {
        if let Part::FunctionResponse { id, name, response } = part {
            message.insert(
                "tool_call_id".to_string(),
                Value::String(id.clone().unwrap_or_default()),
            );
            message.insert("name".to_string(), Value::String(name.clone()));
            message.insert("content".to_string(), response.clone());
        }
    }

    message
}

/// Convert ADK tool declarations to mistral.rs tool format
pub fn tools_to_mistralrs(tools: &serde_json::Map<String, Value>) -> Result<Vec<Value>> {
    let mut mistral_tools = Vec::new();

    for (name, tool_def) in tools {
        let tool_obj = tool_def.as_object().ok_or_else(|| {
            MistralRsError::ToolConversion(format!("Tool '{}' is not an object", name))
        })?;

        let mut function = serde_json::Map::new();
        function.insert("name".to_string(), Value::String(name.clone()));

        if let Some(desc) = tool_obj.get("description") {
            function.insert("description".to_string(), desc.clone());
        }

        if let Some(params) = tool_obj.get("parameters") {
            function.insert("parameters".to_string(), params.clone());
        }

        let mut tool = serde_json::Map::new();
        tool.insert("type".to_string(), Value::String("function".to_string()));
        tool.insert("function".to_string(), Value::Object(function));

        mistral_tools.push(Value::Object(tool));
    }

    Ok(mistral_tools)
}

/// Extract tool name from mistral.rs tool format
pub fn extract_tool_name(tool: &Value) -> Option<String> {
    tool.get("function")
        .and_then(|f| f.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
}

/// Extract tool description from mistral.rs tool format
pub fn extract_tool_description(tool: &Value) -> Option<String> {
    tool.get("function")
        .and_then(|f| f.get("description"))
        .and_then(|d| d.as_str())
        .map(|s| s.to_string())
}

/// Extract tool parameters from mistral.rs tool format
pub fn extract_tool_parameters(tool: &Value) -> Option<Value> {
    tool.get("function")
        .and_then(|f| f.get("parameters"))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_content_to_message_user() {
        let content = Content {
            role: "user".to_string(),
            parts: vec![Part::Text { text: "Hello, world!".to_string() }],
        };

        let message = content_to_message(&content);
        assert_eq!(message.get("role").unwrap(), "user");
        assert_eq!(message.get("content").unwrap(), "Hello, world!");
    }

    #[test]
    fn test_content_to_message_assistant() {
        let content = Content {
            role: "model".to_string(),
            parts: vec![Part::Text { text: "Hi there!".to_string() }],
        };

        let message = content_to_message(&content);
        assert_eq!(message.get("role").unwrap(), "assistant");
    }

    #[test]
    fn test_content_to_message_with_function_call() {
        let content = Content {
            role: "model".to_string(),
            parts: vec![Part::FunctionCall {
                id: Some("call_123".to_string()),
                name: "get_weather".to_string(),
                args: json!({"location": "Tokyo"}),
            }],
        };

        let message = content_to_message(&content);
        assert_eq!(message.get("role").unwrap(), "assistant");
        let tool_calls = message.get("tool_calls").unwrap().as_array().unwrap();
        assert_eq!(tool_calls.len(), 1);
    }

    #[test]
    fn test_tools_to_mistralrs() {
        let mut tools = serde_json::Map::new();
        tools.insert(
            "get_weather".to_string(),
            json!({
                "description": "Get weather for a location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    },
                    "required": ["location"]
                }
            }),
        );

        let result = tools_to_mistralrs(&tools).unwrap();
        assert_eq!(result.len(), 1);

        let tool = &result[0];
        assert_eq!(extract_tool_name(tool), Some("get_weather".to_string()));
        assert_eq!(
            extract_tool_description(tool),
            Some("Get weather for a location".to_string())
        );
        assert!(extract_tool_parameters(tool).is_some());
    }

    #[test]
    fn test_tool_conversion_roundtrip() {
        let original_name = "test_function";
        let original_desc = "A test function";
        let original_params = json!({
            "type": "object",
            "properties": {
                "arg1": {"type": "string"},
                "arg2": {"type": "number"}
            }
        });

        let mut tools = serde_json::Map::new();
        tools.insert(
            original_name.to_string(),
            json!({
                "description": original_desc,
                "parameters": original_params.clone()
            }),
        );

        let converted = tools_to_mistralrs(&tools).unwrap();
        let tool = &converted[0];

        // Verify roundtrip preserves structure
        assert_eq!(extract_tool_name(tool), Some(original_name.to_string()));
        assert_eq!(extract_tool_description(tool), Some(original_desc.to_string()));
        assert_eq!(extract_tool_parameters(tool), Some(original_params));
    }
}
