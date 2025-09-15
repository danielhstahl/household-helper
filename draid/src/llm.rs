use crate::psql_memory::{Message, MessageType};
use async_openai::{
    Client,
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
        CreateChatCompletionStreamResponse,
    },
};
use std::pin::Pin;

// Create request using builder pattern
// Every request struct has companion builder struct with same name + Args suffix

pub fn get_llm(api_endpoint: &str) -> Client<OpenAIConfig> {
    Client::with_config(OpenAIConfig::default().with_api_base(format!("{}/v1", api_endpoint)))
}
pub fn get_bot(
    model_name: &str,
    system_prompt: &str,
) -> Result<CreateChatCompletionRequest, OpenAIError> {
    CreateChatCompletionRequestArgs::default()
        .model(model_name)
        .max_tokens(40_u32)
        .stream(true)
        .messages([ChatCompletionRequestSystemMessageArgs::default()
            .content(system_prompt)
            .build()?
            .into()])
        .build()
}

//unforutnately I think I need to clone the bot each time
pub async fn chat(
    client: &Client<OpenAIConfig>,
    mut bot: CreateChatCompletionRequest,
    previous_messages: &[Message],
    new_message: &str,
) -> Result<
    Pin<
        Box<
            dyn futures::Stream<
                    Item = Result<
                        CreateChatCompletionStreamResponse,
                        async_openai::error::OpenAIError,
                    >,
                > + std::marker::Send,
        >,
    >,
    OpenAIError,
> {
    bot.messages.truncate(1); //only keep system prompt

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
    let stream = client.chat().create_stream(bot).await?;
    Ok(stream)
}
