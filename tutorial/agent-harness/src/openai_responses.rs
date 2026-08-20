use crate::protocol::{
    ContentBlock, Message, ModelRequest, ModelResponse, ProtocolError, Role, TextBlock,
    ToolResultBlock, ToolUseBlock,
};
use serde_json::{Value, json};

/// Encodes a unified request into an OpenAI Responses API JSON body.
pub fn encode_request(request: &ModelRequest) -> Result<Value, ProtocolError> {
    let mut input = Vec::new();
    if let Some(system) = request
        .system
        .as_deref()
        .filter(|text| !text.trim().is_empty())
    {
        input.push(json!({"type":"message","role":"system","content":[{"type":"input_text","text":system}]}));
    }
    for message in &request.messages {
        input.extend(encode_message(message)?);
    }
    if input.is_empty() {
        return Err(ProtocolError::Encode(
            "encoded request input must not be empty".into(),
        ));
    }
    let mut body = json!({"model":request.model,"input":input});
    if !request.tools.is_empty() {
        body["tools"] = Value::Array(request.tools.iter().map(|tool| json!({"type":"function","name":tool.name,"description":tool.description,"parameters":tool.input_schema,"strict":tool.strict})).collect());
    }
    Ok(body)
}

fn encode_message(message: &Message) -> Result<Vec<Value>, ProtocolError> {
    match message.role {
        Role::Tool => message.content.iter().map(encode_tool_result).collect(),
        Role::Assistant => encode_assistant(message),
        Role::System | Role::User => encode_text_message(message),
    }
}

fn encode_text_message(message: &Message) -> Result<Vec<Value>, ProtocolError> {
    let mut content = Vec::new();
    for block in &message.content {
        match block {
            ContentBlock::Text(text) => content.push(json!({"type":"input_text","text":text.text})),
            ContentBlock::ToolUse(_) => {
                return Err(ProtocolError::Encode(
                    "tool_use blocks require role=assistant".into(),
                ));
            }
            ContentBlock::ToolResult(_) => {
                return Err(ProtocolError::Encode(
                    "tool_result blocks require role=tool".into(),
                ));
            }
        }
    }
    Ok(vec![
        json!({"type":"message","role":message.role.as_str(),"content":content}),
    ])
}

fn encode_assistant(message: &Message) -> Result<Vec<Value>, ProtocolError> {
    let mut items = Vec::new();
    let mut text = Vec::new();
    for block in &message.content {
        match block {
            ContentBlock::Text(value) => text.push(json!({"type":"output_text","text":value.text})),
            ContentBlock::ToolUse(tool) => {
                if !text.is_empty() {
                    items.push(json!({"type":"message","role":"assistant","content":text}));
                    text = Vec::new();
                }
                let arguments = serde_json::to_string(&tool.input).map_err(|_| {
                    ProtocolError::Encode("tool_use input is not JSON-serializable".into())
                })?;
                items.push(json!({"type":"function_call","call_id":tool.id,"name":tool.name,"arguments":arguments}));
            }
            ContentBlock::ToolResult(_) => {
                return Err(ProtocolError::Encode(
                    "assistant message cannot contain tool_result".into(),
                ));
            }
        }
    }
    if !text.is_empty() {
        items.push(json!({"type":"message","role":"assistant","content":text}));
    }
    if items.is_empty() {
        return Err(ProtocolError::Encode(
            "assistant message produced no provider items".into(),
        ));
    }
    Ok(items)
}

fn encode_tool_result(block: &ContentBlock) -> Result<Value, ProtocolError> {
    let ContentBlock::ToolResult(result) = block else {
        return Err(ProtocolError::Encode(
            "role=tool messages may only contain tool_result blocks".into(),
        ));
    };
    Ok(json!({"type":"function_call_output","call_id":result.tool_use_id,"output":result.content}))
}

/// Parses and decodes a JSON response, requiring an object at the top level.
pub fn decode_response_json(text: &str) -> Result<ModelResponse, ProtocolError> {
    let body: Value = serde_json::from_str(text)
        .map_err(|_| ProtocolError::Decode("API returned invalid JSON".into()))?;
    if !body.is_object() {
        return Err(ProtocolError::Decode(
            "API response must be a JSON object".into(),
        ));
    }
    decode_response(&body)
}

/// Decodes provider output items into ordered unified content blocks.
pub fn decode_response(body: &Value) -> Result<ModelResponse, ProtocolError> {
    let output = body
        .get("output")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            ProtocolError::Decode("API response did not contain an output list".into())
        })?;
    let mut blocks = Vec::new();
    for item in output {
        match item.get("type").and_then(Value::as_str) {
            Some("message") => blocks.extend(decode_message(item)),
            Some("function_call") => blocks.push(ContentBlock::ToolUse(decode_call(item)?)),
            _ => {}
        }
    }
    if blocks.is_empty() {
        return Err(ProtocolError::Decode(
            "API response contained no decodable content blocks".into(),
        ));
    }
    Ok(ModelResponse {
        id: body.get("id").and_then(Value::as_str).map(str::to_owned),
        model: body.get("model").and_then(Value::as_str).map(str::to_owned),
        message: Message::try_new(Role::Assistant, blocks)?,
        status: body
            .get("status")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

/// Checks HTTP status and decodes the body without exposing response content.
pub fn parse_http_response(status: u16, body: &str) -> Result<ModelResponse, ProtocolError> {
    if !(200..300).contains(&status) {
        return Err(ProtocolError::Api(format!(
            "API request failed with HTTP {status}"
        )));
    }
    decode_response_json(body)
}

fn decode_message(item: &Value) -> Vec<ContentBlock> {
    item.get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|part| match part.get("type").and_then(Value::as_str) {
            Some("output_text") | Some("text") | Some("input_text") => part
                .get("text")
                .and_then(Value::as_str)
                .filter(|text| !text.trim().is_empty())
                .map(|text| {
                    ContentBlock::Text(TextBlock {
                        text: text.to_owned(),
                    })
                }),
            Some("refusal") => part
                .get("refusal")
                .and_then(Value::as_str)
                .filter(|text| !text.trim().is_empty())
                .map(|text| {
                    ContentBlock::Text(TextBlock {
                        text: text.to_owned(),
                    })
                }),
            _ => None,
        })
        .collect()
}

fn decode_call(item: &Value) -> Result<ToolUseBlock, ProtocolError> {
    let id = item
        .get("call_id")
        .or_else(|| item.get("id"))
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| ProtocolError::Decode("function_call is missing call_id".into()))?;
    let name = item
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| ProtocolError::Decode("function_call is missing name".into()))?;
    let input = parse_arguments(item.get("arguments").unwrap_or(&Value::Null))?;
    Ok(ToolUseBlock {
        id: id.to_owned(),
        name: name.to_owned(),
        input,
    })
}

fn parse_arguments(arguments: &Value) -> Result<Value, ProtocolError> {
    match arguments {
        Value::Object(_) => Ok(arguments.clone()),
        Value::Null => Ok(json!({})),
        Value::String(text) if text.trim().is_empty() => Ok(json!({})),
        Value::String(text) => {
            let parsed: Value = serde_json::from_str(text).map_err(|_| {
                ProtocolError::Decode("function_call arguments are not valid JSON".into())
            })?;
            if parsed.is_null() {
                Ok(json!({}))
            } else if parsed.is_object() {
                Ok(parsed)
            } else {
                Err(ProtocolError::Decode(
                    "function_call arguments JSON must be an object".into(),
                ))
            }
        }
        _ => Err(ProtocolError::Decode(
            "function_call arguments must be an object or JSON string".into(),
        )),
    }
}

/// Builds one tool observation while preserving its original call id.
pub fn tool_result_message(
    id: impl Into<String>,
    content: impl Into<String>,
) -> Result<Message, ProtocolError> {
    Message::try_new(
        Role::Tool,
        vec![ContentBlock::ToolResult(ToolResultBlock {
            tool_use_id: id.into(),
            content: content.into(),
            is_error: false,
        })],
    )
}
