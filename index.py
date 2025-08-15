from llama_index.llms.openai import OpenAI
from llama_index.core.agent.workflow import FunctionAgent
from llama_index.core.agent.workflow import AgentStream
from llama_index.embeddings.ollama import OllamaEmbedding
from tutor.prompt import get_tutor_prompt
from fastapi import FastAPI
from fastapi.responses import StreamingResponse
from contextlib import asynccontextmanager
from pydantic import BaseModel
import logging
from uuid import uuid4
from chat import get_memory
from vector_store import (
    get_connection_information,
    get_vector_store,
)
from llama_index.observability.otel import LlamaIndexOpenTelemetry

from system_prompt import get_system_prompt

instrumentor = LlamaIndexOpenTelemetry()

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


# this seems hacky, see if there is a better method
agent_wrapper = {}
session_memory = {}

logger = logging.getLogger("uvicorn.error")
logger.setLevel(logging.DEBUG)


@asynccontextmanager
async def lifespan(app: FastAPI):
    instrumentor.start_registering()

    conn_str, db_name = get_connection_information()
    vector_store = get_vector_store(
        conn_str,
        db_name,
        "household_helper",
    )
    memory = get_memory(ollama_embedding, vector_store)

    # Enable logging
    # logging.basicConfig(level=logging.DEBUG)
    helper_agent = FunctionAgent(
        name="HelperAgent",
        description="Household helper.  Can hand off to TutorAgent.",
        # tools=tools,
        llm=llm,
        system_prompt=get_system_prompt(),
        chat_history=True,
    )

    tutor_agent = FunctionAgent(
        name="TutorAgent",
        description="Tutor to help with grade-school homework.",
        system_prompt=get_tutor_prompt(),
        llm=llm,
        chat_history=True,
    )

    agent_wrapper["helper_agent"] = helper_agent
    agent_wrapper["tutor_agent"] = tutor_agent

    # todo, remove once sessions are fully working
    agent_wrapper["memory"] = memory
    agent_wrapper["vector_store"] = vector_store

    logger.debug("loaded agents")

    yield
    agent_wrapper.clear()
    session_memory.clear()


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


@app.post("/session")
async def session(chat: Chat) -> str:
    session_id = str(uuid4())
    vector_store = agent_wrapper["vector_store"]
    # TODO: define session ID
    session_memory[session_id] = get_memory(ollama_embedding, vector_store)
    return session_id


@app.post("/query")
async def query(chat: Chat):
    agent = agent_wrapper["helper_agent"]
    memory = agent_wrapper["memory"]
    handler = agent.run(chat.text, memory=memory)
    return StreamingResponse(yield_streams(handler.stream_events()))


@app.post("/tutor")
async def tutor(chat: Chat):
    agent = agent_wrapper["tutor_agent"]
    memory = agent_wrapper["memory"]
    handler = agent.run(chat.text, memory=memory)
    return StreamingResponse(yield_streams(handler.stream_events()))
