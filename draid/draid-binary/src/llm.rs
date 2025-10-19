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
use futures::{SinkExt, StreamExt, future};
use poem::web::websocket::{Message, WebSocketStream};
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

fn get_end_of_thinking(item: &Result<CreateChatCompletionStreamResponse, OpenAIError>) -> bool {
    match item {
        Ok(item) => contains_stop_word(&get_final_tokens_from_stream(item)),
        Err(_e) => false,
    }
}

pub async fn chat_with_tools(
    bot: &Bot,
    tx: &mut WebSocketStream,
    previous_messages: &[MessageResult],
    new_message: &str,
    span_id: String,
) -> anyhow::Result<String> {
    info!(
        tool_use = false,
        endpoint = "query",
        span_id,
        "Initiated chat"
    );
    //create storage for tool calls
    let mut registry = ToolRegistry::new();
    let mut tool_results: std::collections::HashMap<(u32, u32), ChatCompletionMessageToolCall> =
        std::collections::HashMap::new();
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

    let mut finish_reason: Option<FinishReason> = None;
    let mut stream = bot
        .llm
        .chat()
        .create_stream(req)
        .await?
        .skip_while(|item| future::ready(!get_end_of_thinking(&item)));
    let mut full_message_no_tools = String::new();
    while let Some(result) = stream.next().await {
        let result = result?;
        let tokens = get_final_tokens_from_stream(&result);

        if tokens.trim().is_empty() {
            finish_reason = result
                .choices
                .iter()
                .filter_map(|choice| choice.finish_reason)
                .next();

            result
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
                        let tool_call = tool_results.entry(key).or_insert_with(|| {
                            ChatCompletionMessageToolCall {
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
                            }
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
        } else if contains_stop_word(&tokens) {
            info!(tool_use = false, endpoint = "query", span_id, "First token");
        } else {
            full_message_no_tools.push_str(&tokens);
            //no tool call, just send results
            match tx.send(Message::Text(tokens)).await {
                Ok(_) => {} // Success
                Err(_) => {
                    info!(
                        tool_use = false,
                        endpoint = "query",
                        span_id,
                        "Client disconnected (channel closed) during tool response stream."
                    );
                    // This stops the function from processing the rest of the stream
                    return Ok(full_message_no_tools);
                }
            }
        }
    }
    match finish_reason {
        Some(FinishReason::ToolCalls) => {
            info!(
                tool_use = true,
                endpoint = "query",
                span_id,
                "Finished constructing tool calls"
            );
            let mut full_message_tools = String::new();
            //no tools since we don't want to call the tools a second time
            let req_no_tools =
                construct_messages(get_req(&bot, &None)?, previous_messages, new_message)?;
            let mut stream =
                tool_response(&bot.llm, registry, req_no_tools, tool_results, &span_id)
                    .await?
                    .skip_while(|item| future::ready(!get_end_of_thinking(&item)));
            while let Some(result) = stream.next().await {
                let result = result?;
                let tokens = get_final_tokens_from_stream(&result);
                if contains_stop_word(&tokens) {
                    info!(tool_use = true, endpoint = "query", span_id, "First token");
                } else {
                    full_message_tools.push_str(&tokens);
                    match tx.send(Message::Text(tokens)).await {
                        Ok(_) => {} // Success
                        Err(_) => {
                            info!(
                                tool_use = true,
                                endpoint = "query",
                                span_id,
                                "Client disconnected (channel closed) during tool response stream."
                            );
                            // This stops the function from processing the rest of the stream
                            return Ok(full_message_tools);
                        }
                    }
                }
            }
            info!(
                tool_use = true,
                endpoint = "query",
                span_id,
                "Completed response"
            );
            Ok(full_message_tools)
        }
        _ => {
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
    use super::contains_stop_word;

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
}
