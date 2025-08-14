# from llama_index.llms.ollama import Ollama
# from llama_index.llms.lmstudio import LMStudio
from llama_index.llms.openai import OpenAI
from llama_index.core.agent.workflow import AgentWorkflow, FunctionAgent
from llama_index.core.agent.workflow import AgentStream
from llama_index.embeddings.ollama import OllamaEmbedding
from tutor.prompt import get_tutor_prompt
from fastapi import FastAPI
from fastapi.responses import StreamingResponse
from contextlib import asynccontextmanager
from pydantic import BaseModel
import logging
from chat import get_memory
from vector_store import (
    get_connection_information,
    get_vector_store,
)

# from helper.prompt import helper_template
from system_prompt import get_system_prompt


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


# this seems hacky, see if there is a better method
agent_wrapper = {}


logger = logging.getLogger("uvicorn.error")
logger.setLevel(logging.DEBUG)


@asynccontextmanager
async def lifespan(app: FastAPI):
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
        name="HelperAgent",
        description="Household helper.  Can hand off to TutorAgent.",
        # tools=tools,
        llm=llm,
        system_prompt=get_system_prompt()
        + "  You can hand off to the TutorAgent if a question appears to be related to grade-school homework.",
        chat_history=True,
        can_handoff_to=["TutorAgent"],
    )

    tutor_agent = FunctionAgent(
        name="TutorAgent",
        description="Tutor to help with grade-school homework.",
        system_prompt=get_tutor_prompt(),
        llm=llm,
        chat_history=True,
    )
    agent_workflow = AgentWorkflow(
        agents=[agent, tutor_agent],
        root_agent=agent.name,
    )

    agent_wrapper["agent"] = agent_workflow
    agent_wrapper["memory"] = memory

    logger.debug("got past lifespan")

    yield
    agent_wrapper.clear()


class Chat(BaseModel):
    text: str


app = FastAPI(lifespan=lifespan)


@app.get("/state")
async def state():
    # print(agent_wrapper)

    return agent_wrapper


async def yield_streams(stream_events):
    async for event in stream_events:
        if isinstance(event, AgentStream):
            yield event.delta


@app.post("/query")
async def query(chat: Chat):
    # print(agent_wrapper)
    logger.debug(agent_wrapper)
    agent = agent_wrapper["agent"]
    memory = agent_wrapper["memory"]
    handler = agent.run(chat.text, memory=memory)

    return StreamingResponse(yield_streams(handler.stream_events()))


# if __name__ == "__main__":
#    nest_asyncio.apply()
#    loop = asyncio.get_event_loop()
#    loop.run_until_complete(main())
