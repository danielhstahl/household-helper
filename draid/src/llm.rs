use crate::psql_memory::{MessageResult, MessageType};
use async_openai::{
    Client,
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
        ChatCompletionResponseStream, ChatCompletionToolArgs, ChatCompletionToolChoiceOption,
        ChatCompletionToolType, CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
        FinishReason, FunctionCall, FunctionObjectArgs,
    },
};
use futures::StreamExt;
use futures::future::join_all;
use reqwest::Client as HttpClient;

use rocket::tokio::{self};
use rocket::{
    serde,
    serde::{Deserialize, Serialize},
    tokio::sync::mpsc::Sender,
};

use rocket::{
    serde::json::{Value, json},
    tokio::task::JoinHandle,
};
use std::{ops::Deref, sync::Arc};
use tokio::task;

//TODO make many things PRIVATE

fn get_llm(api_endpoint: &str) -> Client<OpenAIConfig> {
    Client::with_config(OpenAIConfig::default().with_api_base(format!("{}/v1", api_endpoint)))
}

#[derive(Clone)]
pub struct Bot {
    model_name: String,
    system_prompt: &'static str,
    llm: Client<OpenAIConfig>,
    tools: Option<Vec<Arc<dyn Tool + Send + Sync>>>,
}

impl Bot {
    pub fn new(
        model_name: String,
        system_prompt: &'static str,
        api_endpoint: &str,
        tools: Option<Vec<Arc<dyn Tool + Send + Sync>>>,
    ) -> Self {
        Self {
            model_name,
            system_prompt,
            llm: get_llm(api_endpoint),
            tools,
        }
    }
}

fn get_req_with_tools(
    model_name: &str,
    system_prompt: &str,
    tools: &Vec<Arc<dyn Tool + Send + Sync>>,
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

fn get_req_without_tools(
    model_name: &str,
    system_prompt: &str,
) -> Result<CreateChatCompletionRequest, OpenAIError> {
    let chat_request = CreateChatCompletionRequestArgs::default()
        .model(model_name)
        .stream(true)
        .messages([ChatCompletionRequestSystemMessageArgs::default()
            .content(system_prompt)
            .build()?
            .into()])
        .build()?;

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

/*
pub async fn chat(
    client: &Client<OpenAIConfig>,
    bot: &Bot,
    previous_messages: &[MessageResult],
    new_message: &str,
) -> Result<ChatCompletionResponseStream, OpenAIError> {
    let req = construct_messages(
        get_req_without_tools(&bot.model_name, &bot.system_prompt)?,
        previous_messages,
        new_message,
    )?;
    let stream = client.chat().create_stream(req).await?;
    Ok(stream)
}*/

#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> Value;
    async fn invoke(&self, args: String) -> anyhow::Result<Value>;
}

#[async_trait::async_trait]
impl<P> Tool for P
where
    P: Deref<Target = dyn Tool + Send + Sync> + Send + Sync,
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
    async fn invoke(&self, args: String) -> anyhow::Result<Value> {
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
    map: std::collections::HashMap<&'static str, Arc<dyn Tool + Send + Sync>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            map: Default::default(),
        }
    }
    pub fn register<T: Tool + 'static + Send + Sync>(&mut self, t: T) {
        self.map.insert(t.name(), Arc::new(t));
    }
}

pub async fn chat_with_tools(
    bot: Bot,
    tx: Sender<String>,
    previous_messages: &[MessageResult],
    new_message: String,
) -> anyhow::Result<()> {
    //create storage for tool calls
    let mut registry = ToolRegistry::new();
    let mut tool_results: std::collections::HashMap<(u32, u32), ChatCompletionMessageToolCall> =
        std::collections::HashMap::new();
    let req = match &bot.tools {
        Some(tools) => {
            let req_with_tools = construct_messages(
                get_req_with_tools(&bot.model_name, &bot.system_prompt, &tools)?,
                previous_messages,
                new_message.as_str(),
            )?;
            //clone arc, cheap
            for tool in tools.clone() {
                registry.register(tool);
            }
            req_with_tools
        }
        None => construct_messages(
            get_req_without_tools(&bot.model_name, &bot.system_prompt)?,
            previous_messages,
            new_message.as_str(),
        )?,
    };

    let mut finish_reason: Option<FinishReason> = None;
    let mut stream = bot.llm.chat().create_stream(req).await?;
    while let Some(result) = stream.next().await {
        let result = result?;
        let tokens = result
            .choices
            .iter()
            .filter_map(|chat_choice| (&chat_choice.delta.content).as_ref())
            .map(|s| &**s)
            .collect::<Vec<&str>>()
            .join("");
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
                    println!("this is raw tools {:?}", tools);
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
        } else {
            println!("no tool call, just sending results");
            println!("tokens: {}", tokens);
            tx.send(tokens).await?;
        }
    }
    match finish_reason {
        Some(FinishReason::ToolCalls) => {
            //no tools since we don't want to call the tools AGAIN
            let req_no_tools = construct_messages(
                get_req_without_tools(&bot.model_name, &bot.system_prompt)?,
                previous_messages,
                new_message.as_str(),
            )?;
            let mut stream = tool_response(&bot.llm, registry, req_no_tools, tool_results).await?;
            while let Some(result) = stream.next().await {
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

async fn tool_response<T: Clone>(
    client: &Client<OpenAIConfig>,
    mut registry: ToolRegistry, //consumes registry
    mut req: CreateChatCompletionRequest,
    tools: std::collections::HashMap<T, ChatCompletionMessageToolCall>,
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
            let content = v.1?;
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
    req.messages.push(assistant_message);
    req.messages.extend(tool_messages);
    Ok(client.chat().create_stream(req).await?)
}

/*
#[derive(Clone)]
pub struct EchoTool;

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
    async fn invoke(&self, args: String) -> anyhow::Result<Value> {
        let args: Value = serde::json::from_str(&args)?;
        //args should have content key, see parameters
        Ok(json!({"echo":args}))
    }
}
*/
#[derive(Clone)]
pub struct AddTool;

#[async_trait::async_trait]
impl Tool for AddTool {
    fn name(&self) -> &'static str {
        "calculator"
    }
    fn description(&self) -> &'static str {
        "Add some numbers"
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "a": {
                    "type": "number",
                    "description": "First number",
                },
                "b": {
                    "type": "number",
                    "description": "Second number",
                },
            },
            "required": ["a", "b"],
        })
    }
    async fn invoke(&self, args: String) -> anyhow::Result<Value> {
        let args: Value = serde::json::from_str(&args)?;
        let result = args["a"].as_number().unwrap().as_f64().unwrap()
            + args["b"].as_number().unwrap().as_f64().unwrap();
        Ok(json!({"result":result}))
    }
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Content {
    content: String,
}

#[derive(Clone)]
pub struct KnowledgeBase;

#[async_trait::async_trait]
impl Tool for KnowledgeBase {
    fn name(&self) -> &'static str {
        "knowledge_base"
    }
    fn description(&self) -> &'static str {
        "Knowledge base containing information on Paul Graham"
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Search term to send to knowledge base",
                },
            },
            "required": ["content"],
        })
    }
    async fn invoke(&self, args: String) -> anyhow::Result<Value> {
        let client = HttpClient::new();
        let kb_url = "http://127.0.0.1:8001/content/similar";
        let args: Value = serde::json::from_str(&args)?;
        let content = args["content"].as_str().unwrap();
        let body = json!({"text":content, "num_results":3});
        let response = client.post(kb_url).json(&body).send().await?;
        let result = response.json::<Vec<Content>>().await?;
        println!("response from kb: {:?}", result);
        Ok(json!(result))
    }
}
