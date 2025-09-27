use crate::psql_memory::{MessageResult, MessageType};
use async_openai::{
    Client,
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
        ChatCompletionResponseStream, ChatCompletionTool, ChatCompletionToolArgs,
        ChatCompletionToolChoiceOption, ChatCompletionToolType, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs, CreateChatCompletionStreamResponse, FinishReason,
        FunctionCall, FunctionObjectArgs,
    },
};
use futures::StreamExt;
use futures::TryFutureExt;
use futures::future::join_all;
use futures::{Stream, future::TryJoinAll};
use rocket::tokio::sync::mpsc::{self, Sender};
use rocket::tokio::{self, task::JoinError};
use rocket::{
    serde::json::{Value, json},
    tokio::task::JoinHandle,
};
use std::{
    collections::HashMap,
    ops::Deref,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::task;
pub fn get_llm(api_endpoint: &str) -> Client<OpenAIConfig> {
    Client::with_config(OpenAIConfig::default().with_api_base(format!("{}/v1", api_endpoint)))
}
pub fn get_bot_with_tools(
    model_name: &str,
    system_prompt: &str,
    tools: &Vec<Arc<dyn Tool + Send + Sync>>, //&Vec<impl Tool + 'static>,
) -> Result<CreateChatCompletionRequest, OpenAIError> {
    let chat_request = CreateChatCompletionRequestArgs::default()
        .model(model_name)
        .stream(true)
        .tools(
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
        )
        .tool_choice(ChatCompletionToolChoiceOption::Auto) //let llm choose whether to call a tool
        .messages([ChatCompletionRequestSystemMessageArgs::default()
            .content(system_prompt)
            .build()?
            .into()])
        .build()?;

    Ok(chat_request)
}

pub async fn chat(
    client: &Client<OpenAIConfig>,
    mut bot: CreateChatCompletionRequest, //this is cloned for each request
    previous_messages: &[MessageResult],
    new_message: &str,
) -> Result<(ChatCompletionResponseStream, CreateChatCompletionRequest), OpenAIError> {
    // bot.messages only has the values from get_bot.  Since bot is cloned,
    // new bot.messages created below are dropped after streaming and are
    // not persisted across requests

    for v in previous_messages.iter() {
        bot.messages.push(match v.message_type {
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
    bot.messages.push(
        ChatCompletionRequestUserMessageArgs::default()
            .content(new_message)
            .build()?
            .into(),
    );
    //cloned since create_stream consumes, and we need bot for the next call with tools
    let stream = client.chat().create_stream(bot.clone()).await?;
    Ok((stream, bot))
}

#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> Value;
    async fn invoke(&self, args: Value) -> anyhow::Result<Value>;
}

#[async_trait::async_trait]
impl<P> Tool for P
where
    P: Deref<Target = dyn Tool> + Send + Sync,
    // P must also satisfy other bounds you had on T, like Clone if needed
    // The inner dyn Tool must also satisfy Send + Sync for Arc to be safe
{
    fn name(&self) -> &'static str {
        self.deref().name()
    }
    fn description(&self) -> &'static str {
        self.deref().description()
    }
    fn parameters(&self) -> Value {
        self.deref().parameters()
    }
    async fn invoke(&self, args: Value) -> anyhow::Result<Value> {
        self.deref().invoke(args).await
    }
}

#[derive(Debug)]
struct ToolError {
    name: String,
}

impl std::error::Error for ToolError {}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Tool {} not found", self.name)
    }
}

pub struct ToolRegistry {
    map: std::collections::HashMap<&'static str, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            map: Default::default(),
        }
    }
    pub fn register<T: Tool + 'static>(&mut self, t: T) {
        self.map.insert(t.name(), Box::new(t));
    }
}

pub async fn chat_with_tools(
    client: &Client<OpenAIConfig>,
    bot: CreateChatCompletionRequest,
    registry: ToolRegistry,
    tx: Sender<String>,
    mut current_stream: ChatCompletionResponseStream,
) -> anyhow::Result<()> {
    //create storage for tool calls
    let mut tool_results: std::collections::HashMap<String, ChatCompletionMessageToolCall> =
        std::collections::HashMap::new();

    let mut finish_reason: Option<FinishReason> = None;
    while let Some(result) = current_stream.next().await {
        let result = result?;
        let tokens = result
            .choices
            .iter()
            .filter_map(|chat_choice| (&chat_choice.delta.content).as_ref())
            .map(|s| &**s)
            .collect::<Vec<&str>>()
            .join("");
        if tokens.is_empty() {
            //no need to move
            finish_reason = result
                .choices
                .iter()
                .filter_map(|choice| choice.finish_reason)
                .next();

            result
                .choices
                .into_iter()
                .filter_map(|chat_choice| chat_choice.delta.tool_calls)
                .for_each(|tools| {
                    tools.into_iter().for_each(|tool_call_chunk| {
                        let id = tool_call_chunk.id.unwrap_or_else(|| "123".to_string());
                        let tool_call = tool_results.entry(id.clone()).or_insert_with(|| {
                            ChatCompletionMessageToolCall {
                                id: id,
                                r#type: ChatCompletionToolType::Function,
                                function: FunctionCall {
                                    name: tool_call_chunk
                                        .function
                                        .as_ref()
                                        .and_then(|f| f.name.clone())
                                        .unwrap_or_default(),
                                    arguments: "".to_string(), /*tool_call_chunk
                                                               .function
                                                               .as_ref()
                                                               .and_then(|f| f.arguments.clone()) //arguments may stream across multiple stream.next()!!
                                                               .unwrap_or_default(),*/
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
        } else {
            println!("no tool call, just sending results");
            tx.send(tokens).await?;
        }
    }
    match finish_reason {
        Some(FinishReason::ToolCalls) => {
            println!("got to finish with tool calls 242");
            let mut stream = tool_response(&client, registry, bot, tool_results).await?;
            while let Some(result) = stream.next().await {
                println!("inside final stream line 245");
                let result = result?;
                let tokens = result
                    .choices
                    .iter()
                    .filter_map(|chat_choice| (&chat_choice.delta.content).as_ref())
                    .map(|s| &**s)
                    .collect::<Vec<&str>>()
                    .join("");
                tx.send(tokens).await?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

async fn tool_response(
    client: &Client<OpenAIConfig>,
    mut registry: ToolRegistry, //consumes registry
    mut bot: CreateChatCompletionRequest,
    tools: std::collections::HashMap<String, ChatCompletionMessageToolCall>,
) -> anyhow::Result<ChatCompletionResponseStream> {
    let handles: Vec<JoinHandle<(String, Result<Value, anyhow::Error>)>> = tools
        .clone() //only cloned because of assitant_message below.  I'm not sure if assitant_message is even needed
        .into_iter()
        .map(|(id, tool_call)| {
            let func = registry
                .map
                .remove(tool_call.function.name.as_str())
                .ok_or_else(|| ToolError {
                    name: tool_call.function.name,
                })?;
            let result: task::JoinHandle<(String, Result<Value, anyhow::Error>)> =
                tokio::spawn(async move {
                    (id, func.invoke(json!(tool_call.function.arguments)).await)
                });
            Ok(result)
        })
        .collect::<Result<Vec<_>, ToolError>>()?;
    let results = join_all(handles).await;

    let tool_messages: Vec<ChatCompletionRequestMessage> = results
        .into_iter()
        .map(|v| {
            let v = v?;
            let id = v.0;
            let content = v.1?;
            println!("function call argument: {}", content);
            let message: ChatCompletionRequestMessage =
                ChatCompletionRequestToolMessageArgs::default()
                    .content(content.to_string()) //result of tool call, stringified Json
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
    bot.messages.push(assistant_message);
    bot.messages.extend(tool_messages);
    Ok(client.chat().create_stream(bot).await?)
}

#[derive(Clone)]
struct EchoTool;

#[async_trait::async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &'static str {
        "knowledge_base"
    }
    fn description(&self) -> &'static str {
        "Get data from a knowledge base"
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "The message content required to search the knowledge base",
                },
            },
            "required": ["content"],
        })
    }
    async fn invoke(&self, args: Value) -> anyhow::Result<Value> {
        Ok(json!({"echo":args}))
    }
}

/*
async fn call_fn(name: &str, args: &str) -> Result<Value, String> {
    let mut available_functions: HashMap<&str, fn(&str, &str) -> Value> = HashMap::new();
    available_functions.insert("get_current_weather", get_current_weather);

    let function_args: Value = args.parse().unwrap();

    let location = function_args["location"].as_str().unwrap();
    let unit = function_args["unit"].as_str().unwrap_or("fahrenheit");
    let function = available_functions.get(name).unwrap();
    let function_response = function(location, unit);
    Ok(function_response)
}*/
/*
fn get_current_weather(location: &str, unit: &str) -> Value {
    //let mut rng = thread_rng();

    //let temperature: i32 = rng.gen_range(20..=55);

    let forecast = "sunny";

    let weather_info = json!({
        "location": location,
        "temperature": 30.to_string(),
        "unit": unit,
        "forecast": forecast
    });

    weather_info
}*/
