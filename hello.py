from llama_index.llms.ollama import Ollama
from llama_index.core.agent.workflow import FunctionAgent
from llama_index.core.workflow import Context
from llama_index.core.agent.workflow import AgentStream
import logging
from llama_index.embeddings.ollama import OllamaEmbedding
from chat import get_response, get_tools
from vector_store import (
    get_connection_information,
    get_vector_store_index,
    add_to_store,
)
from helper.prompt import helper_template
from system_prompt import get_system_prompt
import asyncio


ollama_embedding = OllamaEmbedding(
    model_name="bge-m3:567m",
    base_url="http://localhost:11434",  # what is this??
    ollama_additional_kwargs={"mirostat": 0},  # what is this??
)
llm = Ollama(
    model="qwen3:8b",
    request_timeout=120.0,
    thinking=True,
    # Manually set the context window to limit memory usage
    # context_window=8000,
)


async def main():
    conn_str, db_name = get_connection_information()
    vector_index = get_vector_store_index(
        conn_str, db_name, "household_helper", ollama_embedding
    )

    # Enable logging
    # logging.basicConfig(level=logging.DEBUG)
    past_converation_tool = "vector_index_past_conversation"
    tools = get_tools(llm, vector_index, helper_template, past_converation_tool)
    agent = FunctionAgent(
        tools=tools,
        llm=llm,
        system_prompt=get_system_prompt(past_converation_tool),
        chat_history=True,
    )
    # ctx = Context(agent)
    while True:
        text_input = input("User: ")
        if text_input == "exit":
            break
        handler = agent.run(text_input)
        async for event in handler.stream_events():
            if isinstance(event, AgentStream):
                print(event.delta, end="", flush=True)
        add_to_store(vector_index, text_input)


if __name__ == "__main__":
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main())
