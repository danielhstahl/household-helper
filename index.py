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
from chat import get_memory
from vector_store import (
    get_connection_information,
    get_vector_store,
)
from fastapi.security import OAuth2PasswordRequestForm
from fastapi import HTTPException, Depends, status
from llama_index.observability.otel import LlamaIndexOpenTelemetry
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from system_prompt import get_system_prompt
import os
from models import (
    User,
    UserCreate,
    Token,
    CurrentUser,
)
from user import (
    authenticate_user,
    get_current_admin_user,
    get_user_from_db,
    get_current_user,
    create_access_token,
    create_user_in_db_func,
    get_current_user_by_roles,
    update_user_in_db_func,
)
from datetime import timedelta

ACCESS_TOKEN_EXPIRE_MINUTES = 30  # Token valid for 30 minutes
LM_STUDIO_ENDPOINT = os.getenv("LM_STUDIO_ENDPOINT", "http://localhost:1234")
OLLAMA_ENDPOINT = os.getenv("OLLAMA_ENDPOINT", "http://localhost:11434")
LOG_LEVEL = os.getenv("LOG_LEVEL", "INFO").upper()

instrumentor = LlamaIndexOpenTelemetry()

ollama_embedding = OllamaEmbedding(
    model_name="bge-m3:567m",
    base_url=OLLAMA_ENDPOINT,
    ollama_additional_kwargs={"mirostat": 0},
)
llm = OpenAI(
    model_name="qwen3-8b",
    api_base=f"{LM_STUDIO_ENDPOINT}/v1",  ##LMStudio
    is_function_calling_model=True,
    context_window=4096,
)


# this seems hacky, see if there is a better method
agent_wrapper = {}
session_memory = {}

logger = logging.getLogger("household-helper")
logger.setLevel(LOG_LEVEL)


def create_init_admin(db):
    if db.query(User).count() == 0:
        print("No users found. Creating initial admin user.")
        admin_username = "admin"
        admin_password = os.getenv("INIT_ADMIN_PASSWORD", "")
        if admin_password != "":
            admin_user_data = UserCreate(
                username=admin_username, password=admin_password, roles=["admin"]
            )
            create_user_in_db_func(db, admin_user_data)
            print(f"Initial admin user '{admin_username}' created.")
        else:
            print("Must set INIT_ADMIN_PASSWORD env variable!")
    else:
        print("Users already exist. Skipping initial admin creation.")


@asynccontextmanager
async def lifespan(app: FastAPI):
    instrumentor.start_registering()

    conn_str, db_name = get_connection_information()
    vector_store = get_vector_store(
        conn_str,
        db_name,
        "household_helper",
    )

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

    engine = create_engine(f"{conn_str}/user")
    SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
    db = SessionLocal()
    create_init_admin(db)
    # todo, remove once sessions are fully working
    # agent_wrapper["memory"] = memory
    agent_wrapper["vector_store"] = vector_store

    agent_wrapper["db"] = db

    logger.debug("loaded agents")

    yield
    db.close()
    agent_wrapper.clear()
    session_memory.clear()


class Chat(BaseModel):
    text: str


app = FastAPI(lifespan=lifespan)


async def yield_streams(stream_events):
    async for event in stream_events:
        if isinstance(event, AgentStream):
            yield event.delta


@app.post("/session")
async def session(current_user: CurrentUser = Depends(get_current_user)) -> str:
    vector_store = agent_wrapper["vector_store"]
    session_memory[current_user.username] = get_memory(
        ollama_embedding, vector_store, current_user.username
    )
    return current_user.username


@app.post("/query")
async def query(
    chat: Chat, current_user: CurrentUser = Depends(get_current_user_by_roles("helper"))
):
    memory = session_memory[current_user.username]
    agent = agent_wrapper["helper_agent"]
    # memory = agent_wrapper["memory"]
    handler = agent.run(chat.text, memory=memory)
    return StreamingResponse(yield_streams(handler.stream_events()))


@app.post("/tutor")
async def tutor(
    chat: Chat, current_user: CurrentUser = Depends(get_current_user_by_roles("tutor"))
):
    agent = agent_wrapper["tutor_agent"]
    memory = agent_wrapper["memory"]
    handler = agent.run(chat.text, memory=memory)
    return StreamingResponse(yield_streams(handler.stream_events()))


@app.post("/token", response_model=Token)
async def login_for_access_token(form_data: OAuth2PasswordRequestForm = Depends()):
    """
    Endpoint for users to log in and receive an access token (JWT).
    Uses OAuth2PasswordRequestForm for standard username/password submission.
    """
    db = agent_wrapper["db"]
    user = await authenticate_user(db, form_data.username, form_data.password)
    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Incorrect username or password",
            headers={"WWW-Authenticate": "Bearer"},
        )

    # Generate JWT
    access_token_expires = timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    access_token = create_access_token(
        data={"sub": user.username}, expires_delta=access_token_expires
    )

    return {"access_token": access_token, "token_type": "bearer"}


@app.post("/users", response_model=CurrentUser, status_code=status.HTTP_201_CREATED)
async def create_user(
    user_data: UserCreate,
    current_admin: CurrentUser = Depends(get_current_admin_user),  # Requires admin role
):
    """
    Endpoint for administrators to create new users.
    Requires an authenticated admin user.
    """
    db = agent_wrapper["db"]
    if get_user_from_db(db, user_data.username):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Username already registered",
        )

    new_user = create_user_in_db_func(db, user_data)

    return CurrentUser(
        username=new_user.username,
        disabled=new_user.disabled,
        roles=new_user.roles,
    )


@app.patch("/users", response_model=CurrentUser, status_code=status.HTTP_201_CREATED)
async def update_user(
    user_data: UserCreate,
    current_admin: CurrentUser = Depends(get_current_admin_user),  # Requires admin role
):
    """
    Endpoint for administrators to create new users.
    Requires an authenticated admin user.
    """
    db = agent_wrapper["db"]
    if not get_user_from_db(db, user_data.username):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Username does not exist registered",
        )
    user = update_user_in_db_func(db, user_data)
    return CurrentUser(
        username=user.username,
        disabled=user.disabled,
        roles=user.roles,
    )


@app.get("/users/me", response_model=CurrentUser)
async def read_users_me(current_user: CurrentUser = Depends(get_current_user)):
    """
    Endpoint to get information about the currently authenticated user.
    Requires any authenticated user.
    """
    return current_user


@app.get("/users/admin_info", response_model=CurrentUser)
async def read_admin_info(current_admin: CurrentUser = Depends(get_current_admin_user)):
    """
    Endpoint accessible only by administrators to get their own info (demonstrates admin access).
    """
    return current_admin
