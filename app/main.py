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
from fastapi.staticfiles import StaticFiles
from llama_index.observability.otel import LlamaIndexOpenTelemetry

from system_prompt import get_system_prompt
import os
from models import (
    Message,
    MessageInDB,
    Sessions,
    User,
    UserCreate,
    Token,
    CurrentUser,
    Base,
    GenericSuccess,
    UserUpdate,
    SessionInDB,
)
from user import (
    authenticate_user,
    get_current_admin_user,
    get_user_from_db,
    get_current_user,
    create_access_token,
    create_user_in_db_func,
    delete_user_in_db_func,
    get_current_user_by_roles,
    update_user_in_db_func,
    get_user_from_db_by_id,
    get_db,
    engine,
)
from datetime import timedelta


ACCESS_TOKEN_EXPIRE_MINUTES = 30  # Token valid for 30 minutes
LM_STUDIO_ENDPOINT = os.getenv("LM_STUDIO_ENDPOINT", "http://localhost:1234")
OLLAMA_ENDPOINT = os.getenv("OLLAMA_ENDPOINT", "http://localhost:11434")
LOG_LEVEL = os.getenv("LOG_LEVEL", "INFO").upper()
## Additional env variables to set:
# VECTOR_DATABASE_URL (defaults to postgresql://postgres:yourpassword@localhost:5432)
# USER_DATABASE_URL (defaults to sqlite://)

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


def get_session_memory(db: Session, username_id: int, session_id: str) -> Memory:
    if session_id in session_cache:
        return session_cache[session_id]
    else:
        db_messages = (
            db.query(Message)
            .filter(Message.session_id == session_id)
            .filter(
                Message.username_id == username_id
            )  # technically unnecessary, but does prevent bad actors for "stealing" a session id
            .filter(Message.role == "me")
            .order_by(Message.timestamp.desc())
            .limit(100)
            .all()
        )
        memory = get_memory(
            ollama_embedding,
            vector_store,
            str(
                username_id
            ),  # long term memory queries ANY interction with user...to keep entire context available per user
            [msg.content for msg in db_messages],
        )
        session_cache[session_id] = memory

    return memory


logger = logging.getLogger("household-helper")
logger.setLevel(LOG_LEVEL)


def create_init_admin(db, engine):
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


class Chat(BaseModel):
    text: str


async def yield_streams(stream_events, db: Session, session_id: str, username_id: int):
    full_response = []  # Store the full response in psql
    async for event in stream_events:
        if isinstance(event, AgentStream):
            full_response.append(event.delta)
            yield event.delta
    db_message = Message(
        session_id=session_id,
        username_id=username_id,
        role="it",
        content="".join(full_response),
    )
    db.add(db_message)
    db.commit()


def create_fastapi(engine) -> FastAPI:
    @asynccontextmanager
    async def lifespan(app: FastAPI):
        instrumentor.start_registering()

        with Session(engine) as db:
            create_init_admin(db, engine)
            logger.debug("Loaded database")
            yield

        session_cache.clear()

    app = FastAPI(lifespan=lifespan)

    @app.get("/session")
    async def session(
        db: Session = Depends(get_db),
        current_user: CurrentUser = Depends(get_current_user),
    ) -> list[SessionInDB]:
        return [
            SessionInDB(id=session.id, session_start=session.session_start)
            for session in db.query(Sessions)
            .filter(Sessions.username_id == current_user.id)
            .order_by(Sessions.session_start.desc())
            .limit(100)
            .all()
        ]

    @app.get("/session/recent")
    async def recent_session(
        db: Session = Depends(get_db),
        current_user: CurrentUser = Depends(get_current_user),
    ) -> Optional[SessionInDB]:
        session = (
            db.query(Sessions)
            .filter(Sessions.username_id == current_user.id)
            .order_by(Sessions.session_start.desc())
            .first()
        )
        if session:
            return SessionInDB(id=session.id, session_start=session.session_start)
        else:
            return

    @app.post("/session")
    async def create_session(
        db: Session = Depends(get_db),
        current_user: CurrentUser = Depends(get_current_user),
    ) -> SessionInDB:
        db_session = Sessions(id=str(uuid.uuid4()), username_id=current_user.id)
        db.add(db_session)
        db.commit()
        db.refresh(db_session)
        return SessionInDB(id=db_session.id, session_start=db_session.session_start)

    @app.delete(
        "/session/{session_id}",
    )
    async def delete_session(
        session_id: str,
        db: Session = Depends(get_db),
        current_user: CurrentUser = Depends(get_current_user),
    ) -> GenericSuccess:
        session = db.get(Sessions, session_id)
        if not session:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Session does not exist",
            )
        db.delete(session)
        db.commit()
        return GenericSuccess(status="success")

    # consider pagination
    @app.get("/messages/{session_id}")
    async def messages(
        session_id: str,
        db: Session = Depends(get_db),
        current_user: CurrentUser = Depends(get_current_user),
    ) -> list[MessageInDB]:
        return [
            MessageInDB(content=msg.content, role=msg.role, timestamp=msg.timestamp)
            for msg in db.query(Message)
            .filter(Message.username_id == current_user.id)
            .filter(Message.session_id == session_id)
            .order_by(Message.timestamp.desc())  # get most recent
            .limit(100)
            # .order_by(Message.timestamp.asc())  # most recnet needs to be add end
            .all()
        ]

    @app.post("/query")
    async def query(
        chat: Chat,
        session_id: str,
        db: Session = Depends(get_db),
        current_user: CurrentUser = Depends(get_current_user_by_roles("helper")),
    ):
        memory = get_session_memory(db, current_user.id, session_id)
        handler = helper_agent.run(chat.text, memory=memory)
        db_message = Message(
            session_id=session_id,
            username_id=current_user.id,
            role="me",
            content="".join(chat.text),
        )
        db.add(db_message)
        db.commit()
        return StreamingResponse(
            yield_streams(handler.stream_events(), db, session_id, current_user.id)
        )

    @app.post("/tutor")
    async def tutor(
        chat: Chat,
        session_id: str,
        db: Session = Depends(get_db),
        current_user: CurrentUser = Depends(get_current_user_by_roles("tutor")),
    ):
        memory = get_session_memory(db, current_user.id, session_id)
        handler = tutor_agent.run(chat.text, memory=memory)
        db_message = Message(
            session_id=session_id,
            username_id=current_user.id,
            role="me",
            content="".join(chat.text),
        )
        db.add(db_message)
        db.commit()
        return StreamingResponse(
            yield_streams(handler.stream_events(), db, session_id, current_user.id)
        )

    @app.post("/token")
    async def login_for_access_token(
        form_data: OAuth2PasswordRequestForm = Depends(), db: Session = Depends(get_db)
    ) -> Token:
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

        return Token(access_token=access_token, token_type="bearer")

    @app.post("/users")
    async def create_user(
        user: UserCreate,
        current_admin: CurrentUser = Depends(
            get_current_admin_user
        ),  # Requires admin role
        db: Session = Depends(get_db),
    ) -> CurrentUser:
        """
        Endpoint for administrators to create new users.
        Requires an authenticated admin user.
        """
        if get_user_from_db(db, user.username):
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Username already registered",
            )

        db_user = create_user_in_db_func(db, user)

        return CurrentUser(
            id=db_user.id,
            username=db_user.username,
            disabled=db_user.disabled,
            roles=[role.role for role in db_user.roles],
        )

    @app.delete("/users/{id}")
    async def delete_user(
        id: int,
        current_admin: CurrentUser = Depends(
            get_current_admin_user
        ),  # Requires admin role
        db: Session = Depends(get_db),
    ) -> GenericSuccess:
        """
        Endpoint for administrators to create new users.
        Requires an authenticated admin user.
        """
        db_user = get_user_from_db_by_id(db, id)
        if not db_user:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Username does not exist",
            )
        delete_user_in_db_func(db, db_user)

        return GenericSuccess(status="success")

    @app.patch("/users/{id}")
    async def update_user(
        id: int,
        user: UserUpdate,
        current_admin: CurrentUser = Depends(
            get_current_admin_user
        ),  # Requires admin role
        db: Session = Depends(get_db),
    ) -> CurrentUser:
        """
        Endpoint for administrators to update new users.
        Requires an authenticated admin user.
        """
        db_user = get_user_from_db_by_id(db, id)
        if not db_user:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="User does not exist",
            )
        updated_user = update_user_in_db_func(db, db_user, user)
        return CurrentUser(
            id=updated_user.id,
            username=updated_user.username,
            disabled=updated_user.disabled,
            roles=[role.role for role in updated_user.roles],
        )

    @app.get("/users/me", response_model=CurrentUser)
    async def read_users_me(current_user: CurrentUser = Depends(get_current_user)):
        """
        Endpoint to get information about the currently authenticated user.
        Requires any authenticated user.
        """
        return current_user

    @app.get("/users")
    async def read_users(
        current_user: CurrentUser = Depends(get_current_admin_user),
        db: Session = Depends(get_db),
    ) -> list[CurrentUser]:
        """
        Endpoint to get all users information
        """
        return [
            CurrentUser(
                id=user.id,
                username=user.username,
                disabled=user.disabled,
                roles=[role.role for role in user.roles],
            )
            for user in db.query(User).all()
        ]

    @app.get("/users/admin_info")
    async def read_admin_info(
        current_admin: CurrentUser = Depends(get_current_admin_user),
    ) -> CurrentUser:
        """
        Endpoint accessible only by administrators to get their own info (demonstrates admin access).
        """
        return current_admin

    if os.getenv("HOST_STATIC"):
        app.mount("/", StaticFiles(directory="static"), name="static")
    return app


app = create_fastapi(engine)
