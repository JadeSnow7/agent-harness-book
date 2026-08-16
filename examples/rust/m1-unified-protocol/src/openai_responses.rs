//! OpenAI Responses API 与统一协议之间的编解码。

use crate::protocol::{
    ContentBlock, Message, ModelRequest, ModelResponse, ProtocolError, Role, TextBlock,
    ToolResultBlock, ToolUseBlock,
};
use serde_json::{Value, json};

/// 将统一请求编码为 Responses API JSON Body。
pub fn encode_request(request: &ModelRequest) -> Result<Value, ProtocolError> {
    let mut items = Vec::new();

    if let Some(system) = request
        .system
        .as_ref()
        .map(|text| text.trim())
        .filter(|text| !text.is_empty())
    {
        items.push(json!({
            "type": "message",
            "role": "system",
            "content": [{"type": "input_text", "text": system}],
        }));
    }

    for message in &request.messages {
        items.extend(encode_message(message)?);
    }

    if items.is_empty() {
        return Err(ProtocolError::Encode(
            "encoded request input must not be empty".to_owned(),
        ));
    }

    let mut payload = json!({
        "model": request.model,
        "input": items,
    });
    if !request.tools.is_empty() {
        payload["tools"] = Value::Array(
            request
                .tools
                .iter()
                .map(|tool| {
                    json!({
                        "type": "function",
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": tool.input_schema,
                        "strict": tool.strict,
                    })
                })
                .collect(),
        );
    }
    Ok(payload)
}

/// 将 Responses API JSON 解码为统一响应。
pub fn decode_response(body: &Value) -> Result<ModelResponse, ProtocolError> {
    let output = body
        .get("output")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            ProtocolError::Decode("API response did not contain an output list".into())
        })?;

    let mut blocks = Vec::new();
    for item in output {
        let Some(item_type) = item.get("type").and_then(Value::as_str) else {
            continue;
        };
        match item_type {
            "message" => blocks.extend(decode_message_item(item)),
            "function_call" => {
                blocks.push(ContentBlock::ToolUse(decode_function_call(item)?));
            }
            _ => {}
        }
    }

    if blocks.is_empty() {
        return Err(ProtocolError::Decode(
            "API response contained no decodable content blocks".to_owned(),
        ));
    }

    let id = body.get("id").and_then(Value::as_str).map(str::to_owned);
    let model = body.get("model").and_then(Value::as_str).map(str::to_owned);
    let status = body
        .get("status")
        .and_then(Value::as_str)
        .map(str::to_owned);

    Ok(ModelResponse {
        id,
        model,
        message: Message::try_new(Role::Assistant, blocks)?,
        status,
    })
}

/// 解析 JSON 字符串并解码为统一响应。
pub fn decode_response_json(text: &str) -> Result<ModelResponse, ProtocolError> {
    let body: Value = serde_json::from_str(text)
        .map_err(|_| ProtocolError::Decode("API returned invalid JSON".to_owned()))?;
    if !body.is_object() {
        return Err(ProtocolError::Decode(
            "API response must be a JSON object".to_owned(),
        ));
    }
    decode_response(&body)
}

/// 检查 HTTP 状态后解码响应；错误不回显正文。
pub fn parse_http_response(status_code: u16, body: &str) -> Result<ModelResponse, ProtocolError> {
    if !(200..300).contains(&status_code) {
        return Err(ProtocolError::Api(format!(
            "API request failed with HTTP {status_code}"
        )));
    }
    decode_response_json(body)
}

fn encode_message(message: &Message) -> Result<Vec<Value>, ProtocolError> {
    match message.role {
        Role::Tool => message
            .content
            .iter()
            .map(encode_tool_result_item)
            .collect(),
        Role::Assistant => encode_assistant_items(message),
        Role::System | Role::User => {
            let mut content_items = Vec::new();
            for block in &message.content {
                match block {
                    ContentBlock::Text(text) => {
                        content_items.push(json!({
                            "type": "input_text",
                            "text": text.text,
                        }));
                    }
                    ContentBlock::ToolResult(_) => {
                        return Err(ProtocolError::Encode(
                            "tool_result blocks require role=tool".to_owned(),
                        ));
                    }
                    ContentBlock::ToolUse(_) => {
                        return Err(ProtocolError::Encode(
                            "tool_use blocks require role=assistant".to_owned(),
                        ));
                    }
                }
            }
            if content_items.is_empty() {
                return Err(ProtocolError::Encode(
                    "message produced no provider content items".to_owned(),
                ));
            }
            Ok(vec![json!({
                "type": "message",
                "role": message.role.as_str(),
                "content": content_items,
            })])
        }
    }
}

fn encode_assistant_items(message: &Message) -> Result<Vec<Value>, ProtocolError> {
    let mut items = Vec::new();
    let mut text_parts = Vec::new();

    let flush_text = |text_parts: &mut Vec<Value>, items: &mut Vec<Value>| {
        if !text_parts.is_empty() {
            items.push(json!({
                "type": "message",
                "role": "assistant",
                "content": text_parts.clone(),
            }));
            text_parts.clear();
        }
    };

    for block in &message.content {
        match block {
            ContentBlock::Text(text) => {
                text_parts.push(json!({
                    "type": "output_text",
                    "text": text.text,
                }));
            }
            ContentBlock::ToolUse(tool_use) => {
                flush_text(&mut text_parts, &mut items);
                let arguments = serde_json::to_string(&tool_use.input).map_err(|_| {
                    ProtocolError::Encode("tool_use input is not JSON-serializable".to_owned())
                })?;
                items.push(json!({
                    "type": "function_call",
                    "call_id": tool_use.id,
                    "name": tool_use.name,
                    "arguments": arguments,
                }));
            }
            ContentBlock::ToolResult(_) => {
                return Err(ProtocolError::Encode(
                    "assistant message cannot contain tool_result".to_owned(),
                ));
            }
        }
    }
    flush_text(&mut text_parts, &mut items);

    if items.is_empty() {
        return Err(ProtocolError::Encode(
            "assistant message produced no provider items".to_owned(),
        ));
    }
    Ok(items)
}

fn encode_tool_result_item(block: &ContentBlock) -> Result<Value, ProtocolError> {
    let ContentBlock::ToolResult(result) = block else {
        return Err(ProtocolError::Encode(
            "role=tool messages may only contain tool_result blocks".to_owned(),
        ));
    };
    Ok(json!({
        "type": "function_call_output",
        "call_id": result.tool_use_id,
        "output": result.content,
    }))
}

fn decode_message_item(item: &Value) -> Vec<ContentBlock> {
    let Some(content) = item.get("content").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut blocks = Vec::new();
    for content_item in content {
        let Some(content_type) = content_item.get("type").and_then(Value::as_str) else {
            continue;
        };
        match content_type {
            "output_text" | "text" | "input_text" => {
                if let Some(text) = content_item
                    .get("text")
                    .and_then(Value::as_str)
                    .filter(|text| !text.trim().is_empty())
                {
                    blocks.push(ContentBlock::Text(TextBlock {
                        text: text.to_owned(),
                    }));
                }
            }
            "refusal" => {
                if let Some(text) = content_item
                    .get("refusal")
                    .and_then(Value::as_str)
                    .filter(|text| !text.trim().is_empty())
                {
                    blocks.push(ContentBlock::Text(TextBlock {
                        text: text.to_owned(),
                    }));
                }
            }
            _ => {}
        }
    }
    blocks
}

fn decode_function_call(item: &Value) -> Result<ToolUseBlock, ProtocolError> {
    let call_id = item
        .get("call_id")
        .or_else(|| item.get("id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ProtocolError::Decode("function_call is missing call_id".into()))?;
    let name = item
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ProtocolError::Decode("function_call is missing name".into()))?;
    let input = parse_arguments(item.get("arguments").unwrap_or(&Value::Null))?;
    Ok(ToolUseBlock {
        id: call_id.to_owned(),
        name: name.to_owned(),
        input,
    })
}

fn parse_arguments(arguments: &Value) -> Result<Value, ProtocolError> {
    match arguments {
        Value::Object(_) => Ok(arguments.clone()),
        Value::Null => Ok(json!({})),
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(json!({}));
            }
            let value: Value = serde_json::from_str(trimmed).map_err(|_| {
                ProtocolError::Decode("function_call arguments are not valid JSON".to_owned())
            })?;
            if value.is_null() {
                return Ok(json!({}));
            }
            if !value.is_object() {
                return Err(ProtocolError::Decode(
                    "function_call arguments JSON must be an object".to_owned(),
                ));
            }
            Ok(value)
        }
        _ => Err(ProtocolError::Decode(
            "function_call arguments must be an object or JSON string".to_owned(),
        )),
    }
}

/// 构造只含一个 tool_result 的 tool 角色消息。
pub fn tool_result_message(
    tool_use_id: impl Into<String>,
    content: impl Into<String>,
) -> Result<Message, ProtocolError> {
    Message::try_new(
        Role::Tool,
        vec![ContentBlock::ToolResult(ToolResultBlock {
            tool_use_id: tool_use_id.into(),
            content: content.into(),
            is_error: false,
        })],
    )
}
