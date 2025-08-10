# from llama_index.llms.ollama import Ollama
# from llama_index.llms.lmstudio import LMStudio
from llama_index.llms.openai import OpenAI
from llama_index.core.agent.workflow import FunctionAgent
from llama_index.core.agent.workflow import AgentStream, AgentInput
import logging
from llama_index.embeddings.ollama import OllamaEmbedding
import nest_asyncio


from chat import get_memory
from vector_store import (
    get_connection_information,
    get_vector_store,
)

# from helper.prompt import helper_template
from system_prompt import get_system_prompt
import asyncio


ollama_embedding = OllamaEmbedding(
    model_name="bge-m3:567m",
    base_url="http://localhost:11434",
    ollama_additional_kwargs={"mirostat": 0},  # what is this??
)
llm = OpenAI(
    model_name="qwen3-8b",
    api_base="http://localhost:1234/v1",
    is_function_calling_model=True,
    context_window=4096,
)

# Ollama(
#    base_url="http://localhost:11434",
#    model="qwen3:8b",
#    request_timeout=120.0,
#    thinking=True,
#    stream=True,
# )


async def main():
    conn_str, db_name = get_connection_information()
    vector_store = get_vector_store(
        conn_str,
        db_name,
        "household_helper",
    )
    memory = get_memory(ollama_embedding, vector_store)

    # Enable logging
    # logging.basicConfig(level=logging.DEBUG)
    agent = FunctionAgent(
        # tools=tools,
        llm=llm,
        system_prompt=get_system_prompt(),
        chat_history=True,
    )
    while True:
        text_input = input("User: ")
        if text_input == "exit":
            break
        handler = agent.run(text_input, memory=memory)
        async for event in handler.stream_events():
            if isinstance(event, AgentStream):
                print(event.delta, end="", flush=True)
            elif isinstance(event, AgentInput):
                for message in event.input:
                    if "thinking" in message.additional_kwargs:
                        print(message.additional_kwargs["thinking"], flush=True)


if __name__ == "__main__":
    nest_asyncio.apply()
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main())
