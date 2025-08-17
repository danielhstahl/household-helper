import uuid
from llama_index.core.memory import Memory
from sqlalchemy.orm import Session
from typing import Optional
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

from system_prompt import get_system_prompt
import os
from models import Message, Sessions, User, UserCreate, Token, CurrentUser, Base
from user import (
    authenticate_user,
    get_current_admin_user,
    get_user_from_db,
    get_current_user,
    create_access_token,
    create_user_in_db_func,
    get_current_user_by_roles,
    update_user_in_db_func,
    get_db,
    engine,
)
from datetime import timedelta


ACCESS_TOKEN_EXPIRE_MINUTES = 30  # Token valid for 30 minutes
LM_STUDIO_ENDPOINT = os.getenv("LM_STUDIO_ENDPOINT", "http://localhost:1234")
OLLAMA_ENDPOINT = os.getenv("OLLAMA_ENDPOINT", "http://localhost:11434")
LOG_LEVEL = os.getenv("LOG_LEVEL", "INFO").upper()


conn_str, db_name = get_connection_information()
vector_store = get_vector_store(
    conn_str,
    db_name,
    "household_helper",
)

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
# defined once, re-used per session/user
helper_agent = FunctionAgent(
    name="HelperAgent",
    description="Household helper.",
    # tools=tools,
    llm=llm,
    system_prompt=get_system_prompt(),
    chat_history=True,
)

# defined once, re-used per session/user
tutor_agent = FunctionAgent(
    name="TutorAgent",
    description="Tutor to help with grade-school homework.",
    system_prompt=get_tutor_prompt(),
    llm=llm,
    chat_history=True,
)

# this seems hacky, see if there is a better method
session_cache = {}


def get_session_memory(
    db: Session, username: str, session_id: Optional[str] = None
) -> Memory:
    if session_id:
        if session_id in session_cache:
            return session_cache[session_id]
        else:
            db_messages = (
                db.query(Message)
                .filter(Message.session_id == session_id)
                .filter(
                    Message.username == username
                )  # technically unnecessary, but does prevent bad actors for "stealing" a session id
                .order_by(Message.timestamp)
                .limit(100)
                .all()
            )
            memory = get_memory(
                ollama_embedding,
                vector_store,
                username,  # long term memory queries ANY interction with user...to keep entire context available per user
                [msg.content for msg in db_messages],
            )
            session_cache[session_id] = memory

    else:
        session = Sessions(id=uuid.uuid4(), username=username)
        db.add(session)
        db.commit()  # Commit changes to the database
        memory = get_memory(
            ollama_embedding,
            vector_store,
            username,  # long term memory queries ANY interction with user...to keep entire context available per user
        )
        session_cache[session_id] = memory

    return memory


logger = logging.getLogger("household-helper")
logger.setLevel(LOG_LEVEL)


def create_init_admin(db):
    Base.metadata.create_all(bind=engine)
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

    db = next(get_db())
    create_init_admin(db)

    logger.debug("loaded database")

    yield

    session_cache.clear()


class Chat(BaseModel):
    text: str


app = FastAPI(lifespan=lifespan)


async def yield_streams(stream_events):
    async for event in stream_events:
        if isinstance(event, AgentStream):
            yield event.delta


@app.get("/session")
async def session(
    db: Session = Depends(get_db), current_user: CurrentUser = Depends(get_current_user)
) -> list[str]:
    return [
        msg.content
        for msg in db.query(Sessions)
        .filter(Sessions.username == current_user.username)
        .all()
    ]


@app.post("/query")
async def query(
    chat: Chat,
    session_id: str | None = None,
    db: Session = Depends(get_db),
    current_user: CurrentUser = Depends(get_current_user_by_roles("helper")),
):
    memory = get_session_memory(db, current_user.username, session_id)
    handler = helper_agent.run(chat.text, memory=memory)
    return StreamingResponse(yield_streams(handler.stream_events()))


@app.post("/tutor")
async def tutor(
    chat: Chat,
    session_id: str | None = None,
    db: Session = Depends(get_db),
    current_user: CurrentUser = Depends(get_current_user_by_roles("tutor")),
    # agent: FunctionAgent = Depends(get_tutor_agent),
):
    memory = get_session_memory(db, current_user.username, session_id)
    handler = tutor_agent.run(chat.text, memory=memory)
    return StreamingResponse(yield_streams(handler.stream_events()))


@app.post("/token", response_model=Token)
async def login_for_access_token(
    form_data: OAuth2PasswordRequestForm = Depends(), db: Session = Depends(get_db)
):
    """
    Endpoint for users to log in and receive an access token (JWT).
    Uses OAuth2PasswordRequestForm for standard username/password submission.
    """
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
    db: Session = Depends(get_db),
):
    """
    Endpoint for administrators to create new users.
    Requires an authenticated admin user.
    """
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
    db: Session = Depends(get_db),
):
    """
    Endpoint for administrators to create new users.
    Requires an authenticated admin user.
    """
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
