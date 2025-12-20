use crate::psql_memory::{MessageResult, MessageType};
use crate::tools::{Tool, ToolError, ToolRegistry};
use async_openai::types::CreateChatCompletionStreamResponse;
use async_openai::{
    Client,
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
        ChatCompletionResponseStream, ChatCompletionToolArgs, ChatCompletionToolChoiceOption,
        ChatCompletionToolType, CreateChatCompletionRequest, FinishReason, FunctionCall,
        FunctionObjectArgs,
    },
};
use futures::future::join_all;
use futures::{SinkExt, StreamExt};
use poem::web::websocket::{Message, WebSocketStream};
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::task;
use tokio::task::JoinHandle;
use tracing::info;

fn get_llm(api_endpoint: &str) -> Client<OpenAIConfig> {
    Client::with_config(OpenAIConfig::default().with_api_base(format!("{}/v1", api_endpoint)))
}

#[derive(Clone)]
pub struct Bot {
    model_name: String,
    system_prompt: &'static str,
    llm: Client<OpenAIConfig>,
    tools: Option<Vec<Arc<dyn Tool + Send + Sync>>>,
    temperature: Option<f32>,
    presence_penalty: Option<f32>,
    top_p: Option<f32>,
}

impl Bot {
    pub fn new(
        model_name: String,
        system_prompt: &'static str,
        api_endpoint: &str,
        temperature: Option<f32>,
        presence_penalty: Option<f32>,
        top_p: Option<f32>,
        tools: Option<Vec<Arc<dyn Tool + Send + Sync>>>,
    ) -> Self {
        Self {
            model_name,
            system_prompt,
            llm: get_llm(api_endpoint),
            temperature,
            presence_penalty,
            top_p,
            tools,
        }
    }
}

fn get_req(
    bot: &Bot,
    tools: &Option<Vec<Arc<dyn Tool + Send + Sync>>>,
) -> Result<CreateChatCompletionRequest, OpenAIError> {
    let chat_request = CreateChatCompletionRequest {
        model: bot.model_name.clone(),
        stream: Some(true),
        temperature: bot.temperature,
        presence_penalty: bot.presence_penalty,
        top_p: bot.top_p,
        tool_choice: Some(ChatCompletionToolChoiceOption::Auto),
        tools: match &tools {
            Some(tools) => Some(
                tools
                    .iter()
                    .map(|tool| {
                        let chat_completion = ChatCompletionToolArgs::default()
                            .r#type(ChatCompletionToolType::Function)
                            .function(
                                FunctionObjectArgs::default()
                                    .name(tool.name())
                                    .description(tool.description())
                                    .parameters(tool.parameters())
                                    .build()?,
                            )
                            .build()?;
                        Ok(chat_completion)
                    })
                    .collect::<Result<Vec<_>, OpenAIError>>()?,
            ),
            None => None,
        },
        messages: vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(bot.system_prompt)
                .build()?
                .into(),
        ],
        ..Default::default()
    };

    Ok(chat_request)
}

fn construct_messages(
    mut req: CreateChatCompletionRequest,
    previous_messages: &[MessageResult],
    new_message: &str,
) -> Result<CreateChatCompletionRequest, OpenAIError> {
    for v in previous_messages.iter() {
        req.messages.push(match v.message_type {
            MessageType::SystemMessage => ChatCompletionRequestSystemMessageArgs::default()
                .content(v.content.as_str())
                .build()?
                .into(),
            MessageType::AIMessage => ChatCompletionRequestAssistantMessageArgs::default()
                .content(v.content.as_str())
                .build()?
                .into(),
            MessageType::HumanMessage => ChatCompletionRequestUserMessageArgs::default()
                .content(v.content.as_str())
                .build()?
                .into(),
            MessageType::ToolMessage => ChatCompletionRequestToolMessageArgs::default()
                .content(v.content.as_str())
                .build()?
                .into(),
        });
    }
    req.messages.push(
        ChatCompletionRequestUserMessageArgs::default()
            .content(new_message)
            .build()?
            .into(),
    );
    Ok(req)
}

fn get_final_tokens_from_stream(stream: &CreateChatCompletionStreamResponse) -> String {
    stream
        .choices
        .iter()
        .filter_map(|chat_choice| (&chat_choice.delta.content).as_ref())
        .map(|s| &**s)
        .collect::<Vec<&str>>()
        .join("")
}

fn contains_stop_word(token: &str) -> bool {
    token.contains("</think>")
}

/*
fn get_end_of_thinking(item: &Result<CreateChatCompletionStreamResponse, OpenAIError>) -> bool {
    match item {
        Ok(item) => contains_stop_word(&get_final_tokens_from_stream(item)),
        Err(_e) => false,
    }
}
*/
fn construct_tool_call(
    stream_chunk: CreateChatCompletionStreamResponse,
) -> std::collections::HashMap<(u32, u32), ChatCompletionMessageToolCall> {
    let mut tool_results: std::collections::HashMap<(u32, u32), ChatCompletionMessageToolCall> =
        std::collections::HashMap::new();
    stream_chunk
        .choices
        .into_iter()
        .filter_map(|chat_choice| match chat_choice.delta.tool_calls {
            Some(calls) => Some((chat_choice.index, calls)),
            None => None,
        })
        .for_each(|(chat_choice_index, tools)| {
            tools.into_iter().for_each(|tool_call_chunk| {
                // If tool_results.entry(key) exists already, id will be null.
                // But insert_with won't be called in that case
                // So there should never be an ID of "123"
                let id = tool_call_chunk.id.unwrap_or_else(|| "123".to_string());
                let key = (chat_choice_index, tool_call_chunk.index);
                let tool_call =
                    tool_results
                        .entry(key)
                        .or_insert_with(|| ChatCompletionMessageToolCall {
                            id: id,
                            r#type: ChatCompletionToolType::Function,
                            function: FunctionCall {
                                name: tool_call_chunk
                                    .function
                                    .as_ref()
                                    .and_then(|f| f.name.clone())
                                    .unwrap_or_default(),
                                arguments: "".to_string(),
                            },
                        });
                if let Some(arguments) = tool_call_chunk
                    .function
                    .as_ref()
                    .and_then(|f| f.arguments.as_ref())
                {
                    tool_call.function.arguments.push_str(arguments);
                }
            })
        });
    tool_results
}

pub enum ChatStreamResult {
    Message(String),
    ToolCalls(std::collections::HashMap<(u32, u32), ChatCompletionMessageToolCall>),
}

#[derive(Serialize)]
enum TokenCategory {
    Message,
    ChainOfThought,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketToken {
    token_type: TokenCategory,
    tokens: String,
}

async fn process_chat_stream(
    tx: &mut WebSocketStream,
    mut stream: ChatCompletionResponseStream,
) -> anyhow::Result<ChatStreamResult> {
    // chain of thought
    while let Some(result) = stream.next().await {
        let result = result?;
        let tokens = get_final_tokens_from_stream(&result);

        if contains_stop_word(&tokens) {
            // Break out of this loop to switch processing modes.
            break;
        }

        let ws_token = WebSocketToken {
            token_type: TokenCategory::ChainOfThought,
            tokens,
        };
        tx.send(Message::Text(serde_json::to_string(&ws_token)?))
            .await?;
    }
    let mut full_message_no_tools = String::new();
    let mut tool_call_result: Option<ChatStreamResult> = None;
    // real response
    while let Some(result) = stream.next().await {
        let result = result?;
        let tokens = get_final_tokens_from_stream(&result);

        let finish_reason = result
            .choices
            .iter()
            .filter_map(|choice| choice.finish_reason)
            .next();
        let has_tool_calls = result
            .choices
            .iter()
            .any(|choice| choice.delta.tool_calls.is_some());
        if has_tool_calls {
            tool_call_result = Some(ChatStreamResult::ToolCalls(construct_tool_call(result)));
        }
        match finish_reason {
            None => {
                full_message_no_tools.push_str(&tokens);
                let ws_token = WebSocketToken {
                    token_type: TokenCategory::Message,
                    tokens,
                };
                tx.send(Message::Text(serde_json::to_string(&ws_token)?))
                    .await?;
            }
            Some(FinishReason::ToolCalls) => {
                return Ok(match tool_call_result {
                    Some(tool_calls) => Ok(tool_calls),
                    None => Err(OpenAIError::StreamError(
                        "Finish reason is ToolCalls, but no tools to call!".to_string(),
                    )),
                }?);
            }
            _ => return Ok(ChatStreamResult::Message(full_message_no_tools)),
        }
    }
    //shouldn't get here, but if it does just return the message
    Ok(ChatStreamResult::Message(full_message_no_tools))
}

pub async fn chat_with_tools(
    bot: &Bot,
    tx: &mut WebSocketStream,
    previous_messages: &[MessageResult],
    new_message: &str,
    span_id: &String,
) -> anyhow::Result<String> {
    info!(
        tool_use = false,
        endpoint = "query",
        span_id,
        "Initiated chat"
    );
    //create storage for tool calls
    let mut registry = ToolRegistry::new();
    let req = construct_messages(get_req(&bot, &bot.tools)?, previous_messages, new_message)?;

    match &bot.tools {
        Some(tools) => {
            for tool in tools {
                //clone arc, cheap
                registry.register(tool.clone());
            }
        }
        None => (),
    };

    let stream = bot.llm.chat().create_stream(req).await?;
    match process_chat_stream(tx, stream).await? {
        ChatStreamResult::ToolCalls(tool_calls) => {
            info!(
                tool_use = true,
                endpoint = "query",
                span_id,
                "Finished constructing tool calls"
            );
            //no tools since we don't want to call the tools a second time
            let req_no_tools =
                construct_messages(get_req(&bot, &None)?, previous_messages, new_message)?;
            let stream =
                tool_response(&bot.llm, registry, req_no_tools, tool_calls, &span_id).await?;

            let stream_result = process_chat_stream(tx, stream).await?;
            Ok(match stream_result {
                ChatStreamResult::Message(full_message_with_tools) => {
                    info!(
                        tool_use = true,
                        endpoint = "query",
                        span_id,
                        "Completed response"
                    );
                    Ok(full_message_with_tools)
                }
                _ => Err(OpenAIError::StreamError(
                    "Message required from tool call result".to_string(),
                )),
            }?)
        }
        ChatStreamResult::Message(full_message_no_tools) => {
            info!(
                tool_use = false,
                endpoint = "query",
                span_id,
                "Completed response"
            );
            Ok(full_message_no_tools)
        }
    }
}

fn get_truncation_index(content: &String) -> usize {
    std::cmp::min(50, content.len())
}
async fn tool_response<T: Clone>(
    client: &Client<OpenAIConfig>,
    mut registry: ToolRegistry, //consumes registry
    mut req: CreateChatCompletionRequest,
    tools: std::collections::HashMap<T, ChatCompletionMessageToolCall>,
    span_id: &str,
) -> anyhow::Result<ChatCompletionResponseStream> {
    let handles: Vec<JoinHandle<(String, Result<Value, anyhow::Error>)>> = tools
        .iter()
        .map(|(_id, tool_call)| {
            let tool_call_func_name = tool_call.function.name.clone();
            let tool_call_func_args = tool_call.function.arguments.clone();
            let id = tool_call.id.clone();
            let func = registry
                .map
                .remove(tool_call_func_name.as_str())
                .ok_or_else(|| ToolError {
                    name: tool_call_func_name,
                })?;
            let result: task::JoinHandle<(String, Result<Value, anyhow::Error>)> =
                tokio::spawn(async move { (id, func.invoke(tool_call_func_args).await) });
            Ok(result)
        })
        .collect::<Result<Vec<_>, ToolError>>()?;
    let results = join_all(handles).await;

    let tool_messages: Vec<ChatCompletionRequestMessage> = results
        .into_iter()
        .map(|v| {
            let v = v?;
            let id = v.0;
            let content = v.1?.to_string();

            let truncate_content_for_log: usize = get_truncation_index(&content);
            info!(
                tool_use = true,
                endpoint = "query",
                span_id,
                message = format!("tool call result: {}", &content[..truncate_content_for_log])
            );
            let message: ChatCompletionRequestMessage =
                ChatCompletionRequestToolMessageArgs::default()
                    .content(content) //result of tool call, stringified Json
                    .tool_call_id(id)
                    .build()?
                    .into();
            Ok(message)
        })
        .collect::<Result<Vec<ChatCompletionRequestMessage>, anyhow::Error>>()?;
    let assistant_message: ChatCompletionRequestMessage =
        ChatCompletionRequestAssistantMessageArgs::default()
            .tool_calls(
                tools
                    .into_iter()
                    .map(|(_id, tool_call)| tool_call)
                    .collect::<Vec<ChatCompletionMessageToolCall>>(),
            )
            .build()?
            .into();
    req.messages.push(assistant_message);
    req.messages.extend(tool_messages);
    Ok(client.chat().create_stream(req).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::psql_memory::MessageType;
    use async_openai::types::ChatCompletionRequestMessage;

    #[test]
    fn it_returns_true_if_contains_stop_word() {
        let result = contains_stop_word("hello </think>");
        assert!(result);
    }
    #[test]
    fn it_returns_false_if_does_not_contain_stop_word() {
        let result = contains_stop_word("hello");
        assert!(!result);
    }

    #[test]
    fn it_constructs_messages_correctly() {
        let req = CreateChatCompletionRequest::default();
        let previous_messages = vec![
            MessageResult {
                message_type: MessageType::SystemMessage,
                content: "System message".to_string(),
                timestamp: chrono::Utc::now(),
            },
            MessageResult {
                message_type: MessageType::HumanMessage,
                content: "User message".to_string(),
                timestamp: chrono::Utc::now(),
            },
            MessageResult {
                message_type: MessageType::AIMessage,
                content: "AI message".to_string(),
                timestamp: chrono::Utc::now(),
            },
        ];
        let new_message = "New user message";

        let result = construct_messages(req, &previous_messages, new_message).unwrap();

        assert_eq!(result.messages.len(), 4);

        match &result.messages[0] {
            ChatCompletionRequestMessage::System(msg) => match &msg.content {
                async_openai::types::ChatCompletionRequestSystemMessageContent::Text(text) => {
                    assert_eq!(text, "System message")
                }
                _ => panic!("Expected Text content"),
            },
            _ => panic!("Expected System message"),
        }
        match &result.messages[1] {
            ChatCompletionRequestMessage::User(msg) => match &msg.content {
                async_openai::types::ChatCompletionRequestUserMessageContent::Text(text) => {
                    assert_eq!(text, "User message")
                }
                _ => panic!("Expected Text content"),
            },
            _ => panic!("Expected User message"),
        }
        match &result.messages[2] {
            ChatCompletionRequestMessage::Assistant(msg) => match &msg.content {
                Some(async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(
                    text,
                )) => assert_eq!(text, "AI message"),
                _ => panic!("Expected Text content"),
            },
            _ => panic!("Expected Assistant message"),
        }
        match &result.messages[3] {
            ChatCompletionRequestMessage::User(msg) => match &msg.content {
                async_openai::types::ChatCompletionRequestUserMessageContent::Text(text) => {
                    assert_eq!(text, "New user message")
                }
                _ => panic!("Expected Text content"),
            },
            _ => panic!("Expected User message"),
        }
    }

    #[test]
    fn it_gets_req_correctly() {
        let bot = Bot::new(
            "model".to_string(),
            "system prompt",
            "http://localhost:11434",
            Some(0.5),
            Some(0.6),
            Some(0.7),
            None,
        );

        let result = get_req(&bot, &None).unwrap();

        assert_eq!(result.model, "model");
        assert_eq!(result.temperature, Some(0.5));
        assert_eq!(result.presence_penalty, Some(0.6));
        assert_eq!(result.top_p, Some(0.7));
        assert_eq!(result.messages.len(), 1);
        match &result.messages[0] {
            ChatCompletionRequestMessage::System(msg) => match &msg.content {
                async_openai::types::ChatCompletionRequestSystemMessageContent::Text(text) => {
                    assert_eq!(text, "system prompt")
                }
                _ => panic!("Expected Text content"),
            },
            _ => panic!("Expected System message"),
        }
    }
}
